import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('@claim:sample-sandbox opens a complete packet and keeps its state in demo mode', async ({ page }) => {
  const requests:string[]=[]; page.on('request', r=>requests.push(r.url()));
  await page.goto('/demo');
  await expect(page.getByRole('heading',{name:'See an agent change under review'})).toBeVisible();
  await expect(page.getByRole('heading',{name:'Add organization-level retention controls'})).toBeVisible();
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  expect(requests.every(url=>new URL(url).origin==='http://127.0.0.1:4173')).toBeTruthy();
});
test('@claim:packet-export exports the review packet as JSON', async ({ page }) => {
  await page.goto('/demo');
  const download=page.waitForEvent('download'); await page.getByRole('button',{name:'Export packet'}).click();
  const file=await download; expect(file.suggestedFilename()).toBe('diff-gate-packet.json');
  expect(await file.createReadStream()).toBeTruthy();
});
test('keyboard review can resolve every flagged check', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button',{name:'Mark reviewed'}).first().focus(); await page.keyboard.press('Enter');
  await page.getByRole('button',{name:'Mark reviewed'}).first().focus(); await page.keyboard.press('Enter');
  await expect(page.getByRole('button',{name:'Approve for merge'})).toBeEnabled();
});

test('demo state is isolated in its namespace and reset restores the shipped packet', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button',{name:'Mark reviewed'}).first().click();
  await page.reload();
  await expect(page.getByText('1 owner check')).toBeVisible();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:diff-gate'))).toContain('Migration found');
  await page.getByRole('button',{name:'Reset demo'}).click();
  await expect(page.getByText('2 owner checks')).toBeVisible();
});

test('390px mobile view has no horizontal overflow and retains keyboard targets', async ({ page }) => {
  await page.setViewportSize({width:390,height:844});
  await page.goto('/demo');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBeTruthy();
  await page.keyboard.press('Tab');
  await expect(page.locator(':focus')).toBeVisible();
});

test('loaded demo remains reviewable when the browser goes offline', async ({ page, context }) => {
  await page.goto('/demo');
  await context.setOffline(true);
  await page.getByRole('button',{name:'Mark reviewed'}).first().click();
  await expect(page.getByText('1 owner check')).toBeVisible();
  await context.setOffline(false);
});

test('demo has no serious or critical axe findings', async ({ page }) => {
  await page.goto('/demo');
  const results = await new AxeBuilder({ page }).analyze();
  const blocking = results.violations.filter(({ impact }) => impact === 'serious' || impact === 'critical');
  expect(blocking).toEqual([]);
});
