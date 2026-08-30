import { defineConfig } from '@playwright/test';

const cargoTarget = process.env.CARGO_TARGET_DIR ?? 'target';
const quoteForShell = (value: string) => `'${value.replaceAll("'", "'\\\"'\\\"'")}'`;

export default defineConfig({
  testDir: './tests',
  use: { baseURL: 'http://127.0.0.1:4173' },
  webServer: [
    {
      command: 'npm run dev -- --port 4173',
      port: 4173,
      reuseExistingServer: false,
      gracefulShutdown: { signal: 'SIGTERM', timeout: 10_000 },
    },
    {
      command: `env PORT=4174 DATABASE_URL=${quoteForShell(`sqlite:${cargoTarget}/diff-gate-playwright.db?mode=rwc`)} PUBLIC_BASE_URL=http://127.0.0.1:4174 ${quoteForShell(`${cargoTarget}/debug/diff-gate`)}`,
      url: 'http://127.0.0.1:4174/health',
      reuseExistingServer: false,
      // Rust is compiled by scripts/test-browser.sh. Thirty seconds is a
      // truthful bound for starting the prebuilt server and running migrations.
      timeout: 30_000,
      gracefulShutdown: { signal: 'SIGTERM', timeout: 10_000 },
    },
  ],
  reporter: 'line',
});
