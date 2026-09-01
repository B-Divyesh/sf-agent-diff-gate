import assert from 'node:assert/strict';
import { chmod, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const hook = fileURLToPath(new URL('../deploy/factory-container.sh', import.meta.url));

test('@claim:stateful-worker-deploy factory hook calls this product release script only for agent-diff-gate on port 8080', async () => {
  const fixture = await mkdtemp(join(tmpdir(), 'diff-gate-deploy-'));
  try {
    await mkdir(join(fixture, 'scripts'));
    const release = join(fixture, 'scripts', 'deploy-production.sh');
    await writeFile(release, '#!/bin/sh\nprintf "stateful-release-called\\n"\n');
    await chmod(release, 0o755);

    const result = spawnSync(
      hook,
      ['agent-diff-gate', fixture, 'Dockerfile', '8080'],
      { encoding: 'utf8' },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, 'stateful-release-called\n');

    const wrongProduct = spawnSync(hook, ['other-product', fixture, 'Dockerfile', '8080'], {
      encoding: 'utf8',
    });
    assert.equal(wrongProduct.status, 2);
    assert.match(wrongProduct.stderr, /unexpected product/);

    const wrongPort = spawnSync(hook, ['agent-diff-gate', fixture, 'Dockerfile', '3000'], {
      encoding: 'utf8',
    });
    assert.equal(wrongPort.status, 2);
    assert.match(wrongPort.stderr, /container port 8080/);
  } finally {
    await rm(fixture, { recursive: true, force: true });
  }
});
