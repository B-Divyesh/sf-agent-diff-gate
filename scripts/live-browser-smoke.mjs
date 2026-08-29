import { chromium } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { mkdir } from 'node:fs/promises';

const base = process.argv[2] || 'https://agent-diff-gate.sociobot.in';
const artifacts = '.factory/repair-6-artifacts';
await mkdir(artifacts, { recursive: true });
const browser = await chromium.launch({ headless: true });

for (const profile of [
  { name: 'desktop', viewport: { width: 1440, height: 1000 }, colorScheme: 'light' },
  { name: 'mobile', viewport: { width: 390, height: 844 }, colorScheme: 'dark', reducedMotion: 'reduce' },
]) {
  const context = await browser.newContext(profile);
  const page = await context.newPage();
  const errors = [];
  page.on('console', message => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', error => errors.push(error.message));
  for (const path of ['/', '/demo', '/privacy', '/terms']) {
    await page.goto(`${base}${path}`, { waitUntil: 'networkidle' });
    const semantics = await page.evaluate(() => ({
      lang: document.documentElement.lang,
      title: document.title,
      mains: document.querySelectorAll('main').length,
      h1s: document.querySelectorAll('h1').length,
      overflow: document.documentElement.scrollWidth > document.documentElement.clientWidth,
    }));
    if (semantics.lang !== 'en' || !semantics.title || semantics.mains !== 1 || semantics.h1s !== 1 || semantics.overflow) {
      throw new Error(`${profile.name} ${path} semantic/layout failure: ${JSON.stringify(semantics)}`);
    }
    const axe = await new AxeBuilder({ page }).analyze();
    const blocking = axe.violations.filter(({ impact }) => impact === 'serious' || impact === 'critical');
    if (blocking.length) throw new Error(`${profile.name} ${path} axe: ${JSON.stringify(blocking)}`);
  }
  await page.goto(`${base}/demo`, { waitUntil: 'networkidle' });
  await page.keyboard.press('Tab');
  if (!(await page.locator(':focus').isVisible())) throw new Error(`${profile.name}: keyboard focus is not visible`);
  await page.screenshot({ path: `${artifacts}/live-${profile.name}.png`, fullPage: true });
  const missingResponse = await page.goto(`${base}/missing-release-check`, { waitUntil: 'networkidle' });
  if (missingResponse?.status() !== 200 || missingResponse.headers()['x-diff-gate-route'] !== 'not-found') {
    throw new Error(`${profile.name}: unknown route did not return the explicit recovery contract`);
  }
  if (await page.getByRole('heading', { name: 'Page not found' }).count() !== 1) throw new Error(`${profile.name}: missing recovery heading`);
  const missingAxe = await new AxeBuilder({ page }).analyze();
  if (missingAxe.violations.some(({ impact }) => impact === 'serious' || impact === 'critical')) {
    throw new Error(`${profile.name}: inaccessible 404 page`);
  }
  if (errors.length) throw new Error(`${profile.name} console errors: ${errors.join(' | ')}`);
  await context.close();
}

const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
const page = await context.newPage();
const requests = [];
page.on('request', request => requests.push(request.url()));
await page.goto(`${base}/demo`, { waitUntil: 'networkidle' });
await context.setOffline(true);
await page.getByRole('button', { name: 'Mark reviewed' }).first().click();
await page.getByRole('button', { name: 'Export packet' }).click();
if (!(await page.getByText('1 required owner check').isVisible())) throw new Error('offline demo did not remain usable');
if (!requests.every(url => new URL(url).origin === new URL(base).origin)) throw new Error('demo sent a third-party request');
await context.close();
await browser.close();
console.log('Live desktop, 390px mobile, keyboard, accessibility, privacy, and offline demo checks passed.');
