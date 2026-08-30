#!/usr/bin/env node

import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

export const HEALTH_PROBE_SIZE = 100;

export function assertHealthIdentityResults(
  results,
  { expectedBuild, total = HEALTH_PROBE_SIZE } = {},
) {
  const errors = [];
  const statuses = [...new Set(results.map(result => result.status))];
  const builds = [...new Set(results.map(result => result.build))];
  const storageIds = [...new Set(results.map(result => result.storage_id).filter(Boolean))];

  if (results.length !== total) {
    errors.push(`health probe returned ${results.length} responses; expected ${total}`);
  }
  if (statuses.length !== 1 || statuses[0] !== 'ok') {
    errors.push(`health statuses were ${JSON.stringify(statuses)}; expected ["ok"]`);
  }
  if (expectedBuild && (builds.length !== 1 || builds[0] !== expectedBuild)) {
    errors.push(`health builds were ${JSON.stringify(builds)}; expected ["${expectedBuild}"]`);
  }
  if (storageIds.length !== 1) {
    errors.push(`health probe exposed ${storageIds.length} storage identities; expected exactly one`);
  }
  if (errors.length) throw new Error(`Unsafe live health probe:\n- ${errors.join('\n- ')}`);

  return { storageId: storageIds[0], responses: results.length };
}

async function main() {
  const expectedBuild = process.argv[2];
  const paths = process.argv.slice(3);
  if (!expectedBuild || paths.length === 0) {
    throw new Error('Usage: live-health-identity.mjs <expected-build> <health-json>...');
  }
  const results = await Promise.all(paths.map(async path => JSON.parse(await readFile(path, 'utf8'))));
  const summary = assertHealthIdentityResults(results, { expectedBuild });
  process.stdout.write(`${summary.storageId}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
