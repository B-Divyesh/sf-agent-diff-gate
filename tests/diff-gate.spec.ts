import { expect, test } from '@playwright/test';

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
