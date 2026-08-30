import { chromium } from '@playwright/test';

const base = 'https://agent-diff-gate.sociobot.in';
const browser = await chromium.launch({ headless: true });

try {
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    reducedMotion: 'reduce',
    acceptDownloads: true,
  });
  const page = await context.newPage();
  const requests = [];
  const responses = [];
  const errors = [];
  page.on('request', request => requests.push({
    method: request.method(),
    type: request.resourceType(),
    url: request.url(),
  }));
  page.on('response', response => responses.push({ status: response.status(), url: response.url() }));
  page.on('console', message => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', error => errors.push(error.message));

  await page.goto(`${base}/`, { waitUntil: 'networkidle' });
  const first = await page.evaluate(() => ({
    h1: document.querySelector('h1')?.textContent?.trim(),
    primary: [...document.querySelectorAll('a,button')]
      .find(element => element.textContent?.includes('Try it with sample data'))?.textContent?.trim(),
  }));
  await page.getByText('Try it with sample data', { exact: true }).click();
  await page.waitForTimeout(250);

  const banner = await page.getByText('Demo — sample data, nothing is saved').isVisible();
  const approval = page.getByRole('button', { name: 'Approve for merge' });
  const initialDisabled = await approval.isDisabled();
  const initialGuidance = await page
    .getByText('Resolve and save every required owner check before approval.')
    .isVisible();
  const reviewButtons = page.getByRole('button', { name: 'Mark reviewed' });
  const reviewChecks = await reviewButtons.count();
  for (let index = 0; index < reviewChecks; index += 1) {
    const button = reviewButtons.first();
    await button.focus();
    await page.keyboard.press('Enter');
  }
  const enabledAfterReview = await approval.isEnabled();
  await approval.focus();
  const focus = await approval.evaluate(element => {
    const style = getComputedStyle(element);
    return {
      outline: style.outline,
      outlineColor: style.outlineColor,
      outlineWidth: style.outlineWidth,
      boxShadow: style.boxShadow,
    };
  });
  await page.keyboard.press('Enter');
  const approved = await page.getByText('Approved', { exact: true }).isVisible();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  const resetRestored = await page.getByText('2 required owner checks').isVisible();
  const demoKeys = await page.evaluate(() => Object.keys(sessionStorage));
  await page.getByText('Start for real', { exact: true }).click();
  await page.waitForTimeout(250);
  const endKeys = await page.evaluate(() => Object.keys(sessionStorage));
  const offOrigin = requests.filter(request => new URL(request.url).origin !== new URL(base).origin);
  const failed = responses.filter(response => response.status >= 400);

  console.log(JSON.stringify({
    first,
    banner,
    initialDisabled,
    initialGuidance,
    reviewChecks,
    enabledAfterReview,
    focus,
    approved,
    resetRestored,
    demoKeys,
    endKeys,
    requestCount: requests.length,
    requestLog: requests.map(request => `${request.method} ${new URL(request.url).pathname} [${request.type}]`),
    offOrigin,
    failed,
    errors,
  }, null, 2));
  await context.close();
} finally {
  await browser.close();
}
