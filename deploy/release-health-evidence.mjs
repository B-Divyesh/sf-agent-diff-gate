#!/usr/bin/env node

import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

import { assertHealthIdentityResults } from './live-health-identity.mjs';

/**
 * Prove that a replacement kept one healthy, durable SQLite store. The live
 * verifier calls this exact function after collecting two independent sets of
 * 100 health responses, so the fixture test exercises the same acceptance
 * rule as a release.
 */
export function assertReplacementHealthEvidence({
  expectedBuild,
  beforeRevision,
  afterRevision,
  beforeResults,
  afterResults,
}) {
  const before = assertHealthIdentityResults(beforeResults, { expectedBuild });
  const after = assertHealthIdentityResults(afterResults, { expectedBuild });
  const errors = [];

  if (!beforeRevision || !afterRevision || beforeRevision === afterRevision) {
    errors.push('replacement did not create a new app revision');
  }
  if (before.storageId !== after.storageId) {
    errors.push('replacement changed the database identity');
  }
  if (errors.length) throw new Error(`Unsafe replacement verification:\n- ${errors.join('\n- ')}`);

  return {
    beforeRevision,
    afterRevision,
    storageId: before.storageId,
    responsesBefore: before.responses,
    responsesAfter: after.responses,
  };
}

function requiredOption(name) {
  const index = process.argv.indexOf(name);
  if (index === -1 || !process.argv[index + 1]) throw new Error(`Missing ${name}.`);
  return process.argv[index + 1];
}

function fileArguments(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) throw new Error(`Missing ${name}.`);
  const files = [];
  for (const value of process.argv.slice(index + 1)) {
    if (value.startsWith('--')) break;
    files.push(value);
  }
  if (!files.length) throw new Error(`Missing health responses after ${name}.`);
  return files;
}

async function readResults(paths) {
  return Promise.all(paths.map(async path => JSON.parse(await readFile(path, 'utf8'))));
}

async function main() {
  const summary = assertReplacementHealthEvidence({
    expectedBuild: requiredOption('--expected-build'),
    beforeRevision: requiredOption('--before-revision'),
    afterRevision: requiredOption('--after-revision'),
    beforeResults: await readResults(fileArguments('--before')),
    afterResults: await readResults(fileArguments('--after')),
  });
  process.stdout.write(`${summary.storageId}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
