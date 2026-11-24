// End-to-end tests of the browser app (the same app as on the desktop, in WordPress
// and on MyRoots Server). Run: npm run build:app && npm run test:e2e
// A local Chrome can be used instead of the downloaded browser: PW_CHROME=/path/to/chrome
import { defineConfig, devices } from '@playwright/test';

const chrome = process.env.PW_CHROME;

export default defineConfig({
  testDir: 'tests/e2e',
  timeout: 120_000,
  expect: { timeout: 15_000 },
  fullyParallel: true,
  workers: process.env.CI ? 2 : undefined,
  retries: process.env.CI ? 1 : 0,
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'target/e2e-report' }]],
  outputDir: 'target/e2e-results',
  use: {
    baseURL: 'http://localhost:4174/',
    locale: 'en-US',
    trace: 'retain-on-failure',
    acceptDownloads: true,
    launchOptions: chrome ? { executablePath: chrome } : {},
  },
  projects: [
    { name: 'desktop', use: { ...devices['Desktop Chrome'], viewport: { width: 1360, height: 860 } } },
    { name: 'phone', use: { ...devices['Pixel 7'], browserName: 'chromium' }, testMatch: /smoke\.spec/ },
  ],
  webServer: {
    command: 'npx vite preview --port 4174 --strictPort',
    url: 'http://localhost:4174/',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
