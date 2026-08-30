import { chromium } from '@playwright/test';
import { writeFile } from 'node:fs/promises';

const base = 'https://agent-diff-gate.sociobot.in';
const browser = await chromium.launch({ headless: true });
const result = { base, desktop: {}, mobile: {}, requests: [], responses: [], consoleErrors: [], pageErrors: [] };

const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, acceptDownloads: true });
const page = await context.newPage();
page.on('request', request => result.requests.push({ method: request.method(), url: request.url() }));
page.on('response', response => result.responses.push({ url: response.url(), status: response.status(), headers: response.headers() }));
page.on('console', message => { if (message.type() === 'error') result.consoleErrors.push(message.text()); });
page.on('pageerror', error => result.pageErrors.push(error.message));

const home = await page.goto(`${base}/`, { waitUntil: 'networkidle' });
const action = page.getByRole('button', { name: 'Try it with sample data' });
result.desktop.firstRead = {
  status: home?.status(),
  title: await page.title(),
  h1: await page.getByRole('heading', { level: 1 }).innerText(),
  audience: await page.locator('.lede').innerText(),
  action: await action.innerText(),
  actionOutcome: await page.locator('.after-action').innerText(),
};
await action.click();
await page.waitForURL(/demo=1/);
result.desktop.demo = {
  banner: await page.locator('.demo-bar').innerText(),
  heading: await page.getByRole('heading', { level: 1 }).innerText(),
  changedFiles: await page.locator('.file-list code').allInnerTexts(),
  checks: await page.locator('.checks strong').allInnerTexts(),
  approveInitiallyDisabled: await page.getByRole('button', { name: 'Approve for merge' }).isDisabled(),
  storageBefore: await page.evaluate(() => ({ keys: Object.keys(sessionStorage), demo: sessionStorage.getItem('demo:diff-gate') })),
};

while (await page.getByRole('button', { name: 'Mark reviewed' }).count()) {
  const button = page.getByRole('button', { name: 'Mark reviewed' }).first();
  await button.focus();
  const focus = await button.evaluate(el => ({
    visible: el.matches(':focus-visible'),
    outlineStyle: getComputedStyle(el).outlineStyle,
    outlineWidth: getComputedStyle(el).outlineWidth,
    outlineColor: getComputedStyle(el).outlineColor,
  }));
  result.desktop.lastReviewFocus = focus;
  await page.keyboard.press('Enter');
}
result.desktop.approveEnabledAfterChecks = await page.getByRole('button', { name: 'Approve for merge' }).isEnabled();
const downloadPromise = page.waitForEvent('download');
await page.getByRole('button', { name: 'Export packet' }).click();
const download = await downloadPromise;
const downloadPath = await download.path();
const exported = JSON.parse(await (await import('node:fs/promises')).readFile(downloadPath, 'utf8'));
result.desktop.export = {
  filename: download.suggestedFilename(),
  title: exported.title,
  owner: exported.owner,
  changedCount: exported.changed.length,
  checkCount: exported.checks.length,
  states: exported.checks.map(check => check.state),
};
await page.getByRole('button', { name: 'Approve for merge' }).click();
result.desktop.approved = {
  stamp: await page.locator('.stamp').innerText(),
  storage: await page.evaluate(() => sessionStorage.getItem('demo:diff-gate')),
};
await page.getByRole('button', { name: 'Reset demo' }).click();
result.desktop.reset = {
  stamp: await page.locator('.stamp').innerText(),
  markReviewedCount: await page.getByRole('button', { name: 'Mark reviewed' }).count(),
};
await page.getByRole('button', { name: 'Start for real' }).click();
await page.waitForURL(`${base}/`);
result.desktop.startReal = {
  demoKey: await page.evaluate(() => sessionStorage.getItem('demo:diff-gate')),
  signInVisible: await page.getByRole('link', { name: 'Sign in with Sociobot' }).isVisible(),
};
await page.screenshot({ path: '.factory/verification-25-artifacts/live-desktop-flow.png', fullPage: true });
await context.close();

const mobileContext = await browser.newContext({ viewport: { width: 390, height: 844 }, reducedMotion: 'reduce' });
const mobile = await mobileContext.newPage();
await mobile.goto(`${base}/`, { waitUntil: 'networkidle' });
const box = await mobile.getByRole('button', { name: 'Try it with sample data' }).boundingBox();
result.mobile.firstAction = { box, fullyVisible: !!box && box.x >= 0 && box.y >= 0 && box.x + box.width <= 390 && box.y + box.height <= 844 };
result.mobile.homeOverflow = await mobile.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
await mobile.getByRole('button', { name: 'Try it with sample data' }).click();
await mobile.waitForURL(/demo=1/);
result.mobile.demoOverflow = await mobile.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
result.mobile.minInteractiveTarget = await mobile.locator('a,button,input,textarea,select').evaluateAll(elements => Math.min(...elements.filter(el => getComputedStyle(el).display !== 'none').map(el => Math.min(el.getBoundingClientRect().width, el.getBoundingClientRect().height))));
result.mobile.runningAnimationsReducedMotion = await mobile.evaluate(() => document.getAnimations().filter(animation => animation.playState === 'running').length);
await mobile.evaluate(() => { document.documentElement.style.fontSize = '200%'; });
result.mobile.overflowAt200PercentText = await mobile.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
await mobile.screenshot({ path: '.factory/verification-25-artifacts/live-mobile-200.png', fullPage: true });
await mobileContext.close();

result.origins = [...new Set(result.requests.map(request => new URL(request.url).origin))];
result.sameOriginOnly = result.origins.every(origin => origin === new URL(base).origin);
await browser.close();
await writeFile('.factory/verification-25-artifacts/live-e2e.json', JSON.stringify(result, null, 2));
console.log(JSON.stringify(result, null, 2));
