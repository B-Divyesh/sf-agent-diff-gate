#!/usr/bin/env node

import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

export const DATABASE_URL = 'sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none';
export const DEPLOYMENT_CONFIG_VERSION = '4';

const managedEnvironmentNames = new Set([
  'PORT',
  'DATABASE_URL',
  'PUBLIC_BASE_URL',
  'ENTRA_AUTHORITY',
  'ENTRA_TENANT_ID',
  'ENTRA_CLIENT_ID',
  'ENTRA_TEAM_CLAIM',
  'DEPLOYMENT_CONFIG_VERSION',
]);

function copy(value) {
  return JSON.parse(JSON.stringify(value));
}

function requiredEnvironment(runtime) {
  return [
    { name: 'PORT', value: '8080' },
    { name: 'DATABASE_URL', value: DATABASE_URL },
    { name: 'PUBLIC_BASE_URL', value: runtime.PUBLIC_BASE_URL },
    { name: 'ENTRA_AUTHORITY', value: runtime.ENTRA_AUTHORITY },
    { name: 'ENTRA_TENANT_ID', value: runtime.ENTRA_TENANT_ID },
    { name: 'ENTRA_CLIENT_ID', value: runtime.ENTRA_CLIENT_ID },
    { name: 'ENTRA_TEAM_CLAIM', value: runtime.ENTRA_TEAM_CLAIM },
    { name: 'DEPLOYMENT_CONFIG_VERSION', value: DEPLOYMENT_CONFIG_VERSION },
  ];
}

export function renderProductionTemplate(app, { image, storageName, runtime }) {
  if (!app?.properties?.template || !Array.isArray(app.properties.template.containers)) {
    throw new Error('Container App response has no template containers.');
  }
  const template = copy(app.properties.template);
  const applicationContainers = template.containers.filter(({ name }) => name === 'app');
  if (applicationContainers.length !== 1) {
    throw new Error(`Expected one app container; found ${applicationContainers.length}.`);
  }

  template.containers = template.containers.map(container => {
    if (container.name !== 'app') return container;
    const retainedEnvironment = (container.env ?? []).filter(({ name }) => !managedEnvironmentNames.has(name));
    const retainedMounts = (container.volumeMounts ?? []).filter(
      ({ mountPath, volumeName }) => mountPath !== '/data' && volumeName !== 'data',
    );
    return {
      ...container,
      image,
      env: [...retainedEnvironment, ...requiredEnvironment(runtime)],
      volumeMounts: [...retainedMounts, { volumeName: 'data', mountPath: '/data' }],
    };
  });
  // Azure's read response includes cooldownPeriod and pollingInterval, but the
  // 2024-03-01 PATCH schema rejects those read-only fields.
  template.scale = { minReplicas: 1, maxReplicas: 1 };
  template.volumes = [
    ...(template.volumes ?? []).filter(({ name }) => name !== 'data'),
    { name: 'data', storageType: 'AzureFile', storageName },
  ];
  return template;
}

function matchingEnvironment(container, name, value) {
  const entries = (container.env ?? []).filter(entry => entry.name === name);
  return entries.length === 1 && entries[0].value === value;
}

export function productionContractErrors(app, { image, storageName, runtime }) {
  const errors = [];
  const configuration = app?.properties?.configuration;
  const template = app?.properties?.template;
  const applicationContainers = (template?.containers ?? []).filter(({ name }) => name === 'app');
  const container = applicationContainers[0];

  if (configuration?.activeRevisionsMode !== 'Single') errors.push('active revision mode must be Single');
  if (template?.scale?.minReplicas !== 1 || template?.scale?.maxReplicas !== 1) {
    errors.push('SQLite requires exactly one replica');
  }
  if (applicationContainers.length !== 1) errors.push('exactly one app container is required');
  if (container && image && container.image !== image) errors.push(`app image must be ${image}`);
  const dataVolumes = (template?.volumes ?? []).filter(volume => volume.name === 'data');
  if (
    dataVolumes.length !== 1 ||
    dataVolumes[0].storageType !== 'AzureFile' ||
    dataVolumes[0].storageName !== storageName
  ) {
    errors.push(`Azure Files volume data must use ${storageName}`);
  }
  const dataMounts = (container?.volumeMounts ?? []).filter(mount => mount.volumeName === 'data');
  if (dataMounts.length !== 1 || dataMounts[0].mountPath !== '/data') {
    errors.push('Azure Files volume data must be mounted at /data');
  }
  for (const entry of requiredEnvironment(runtime)) {
    if (!container || !matchingEnvironment(container, entry.name, entry.value)) {
      errors.push(`${entry.name} must match the production contract`);
    }
  }
  return errors;
}

export function assertProductionContract(app, options) {
  const errors = productionContractErrors(app, options);
  if (errors.length) throw new Error(`Unsafe production configuration:\n- ${errors.join('\n- ')}`);
}

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1 || !process.argv[index + 1]) throw new Error(`Missing ${name}.`);
  return process.argv[index + 1];
}

async function readStandardInput() {
  let body = '';
  process.stdin.setEncoding('utf8');
  for await (const chunk of process.stdin) body += chunk;
  return JSON.parse(body);
}

async function main() {
  const command = process.argv[2];
  const runtime = JSON.parse(await readFile(option('--config'), 'utf8'));
  const settings = {
    image: process.argv.includes('--image') ? option('--image') : undefined,
    storageName: option('--storage'),
    runtime,
  };
  const app = await readStandardInput();
  if (command === 'render') {
    const template = renderProductionTemplate(app, settings);
    process.stdout.write(`${JSON.stringify({ properties: { template } })}\n`);
    return;
  }
  if (command === 'assert') {
    assertProductionContract(app, settings);
    process.stdout.write('Production control-plane configuration is safe for SQLite.\n');
    return;
  }
  throw new Error('Usage: production-contract.mjs <render|assert> --config <file> --storage <name> [--image <ref>]');
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
