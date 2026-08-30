import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests',
  use: { baseURL: 'http://127.0.0.1:4173' },
  webServer: [
    { command: 'npm run dev -- --port 4173', port: 4173, reuseExistingServer: true },
    {
      command: 'env PORT=4174 DATABASE_URL="sqlite:target/diff-gate-playwright.db?mode=rwc" PUBLIC_BASE_URL="http://127.0.0.1:4174" cargo run --quiet',
      url: 'http://127.0.0.1:4174/health',
      reuseExistingServer: true,
      timeout: 120_000,
    },
  ],
  reporter: 'line',
});
