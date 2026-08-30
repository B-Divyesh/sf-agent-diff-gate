import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

test('@claim:sample-sandbox browser command prebuilds Rust before the health probe', async () => {
  const packageJson = JSON.parse(await readFile(new URL('../package.json', import.meta.url)));
  const command = await readFile(new URL('../scripts/test-browser.sh', import.meta.url), 'utf8');
  const config = await readFile(new URL('../playwright.config.ts', import.meta.url), 'utf8');

  assert.equal(packageJson.scripts['test:browser'], './scripts/test-browser.sh');
  assert.match(command, /^cargo build --quiet$/m);
  assert.match(command, /^exec \.\/node_modules\/.bin\/playwright test "\$@"$/m);
  assert.match(config, /\$\{cargoTarget\}\/debug\/diff-gate/);
  assert.match(config, /timeout: 30_000/);
  assert.match(config, /gracefulShutdown: \{ signal: 'SIGTERM', timeout: 10_000 \}/);
});
