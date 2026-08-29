import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const signedOut = { authenticated: false, entra_sign_in_configured: false, github_app_configured: false };
async function mockSignedOut(page: import('@playwright/test').Page) {
  await page.route('**/api/auth/status', route => route.fulfill({ json: signedOut }));
}

test('@claim:sample-sandbox keeps sample data isolated and discards it when demo mode ends', async ({ page }) => {
  const requests:string[]=[]; page.on('request', r=>requests.push(r.url()));
  await page.goto('/demo');
  await expect(page.getByRole('heading',{name:'See an agent change under review'})).toBeVisible();
  await expect(page.getByRole('heading',{name:'Add organization-level retention controls'})).toBeVisible();
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await page.getByRole('button',{name:'Mark reviewed'}).first().click();
  await page.getByRole('link',{name:'Privacy'}).first().click();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:diff-gate'))).toBeNull();
  await page.getByRole('link',{name:'Demo'}).click();
  await expect(page.getByText('2 owner checks')).toBeVisible();
  expect(requests.every(url=>new URL(url).origin==='http://127.0.0.1:4173')).toBeTruthy();
});
test('@claim:no-third-party-runtime sends no demo data off-origin', async ({ page }) => {
  const requests:string[]=[]; page.on('request', request=>requests.push(request.url()));
  await mockSignedOut(page);
  await page.goto('/');
  await page.getByRole('button',{name:'Try it with sample data'}).click();
  await page.getByRole('button',{name:'Mark reviewed'}).first().click();
  await page.getByRole('button',{name:'Export packet'}).click();
  expect(requests.length).toBeGreaterThan(0);
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

test('390px mobile header reflows without clipping at 200% text size', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockSignedOut(page);
  await page.goto('/');
  await page.addStyleTag({ content: 'html { font-size: 200%; }' });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBeTruthy();
  for (const link of await page.locator('.site-head nav a').all()) {
    const box = await link.boundingBox();
    expect(box).not.toBeNull();
    expect((box?.x || 0) + (box?.width || 0)).toBeLessThanOrEqual(390);
  }
  await expect(page.getByRole('link', { name: 'Privacy' }).first()).toBeVisible();
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

test('dark mode has no serious or critical axe findings on every public route', async ({ browser }) => {
  const context = await browser.newContext({ colorScheme: 'dark', viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await mockSignedOut(page);
  for (const path of ['/', '/demo', '/privacy', '/terms']) {
    await page.goto(path);
    const results = await new AxeBuilder({ page }).analyze();
    const blocking = results.violations.filter(({ impact }) => impact === 'serious' || impact === 'critical');
    expect(blocking, `${path}: ${JSON.stringify(blocking)}`).toEqual([]);
  }
  await context.close();
});

test('light mode has no serious or critical axe findings on every public route', async ({ browser }) => {
  const context = await browser.newContext({ colorScheme: 'light', viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await mockSignedOut(page);
  for (const path of ['/', '/demo', '/privacy', '/terms']) {
    await page.goto(path);
    const results = await new AxeBuilder({ page }).analyze();
    const blocking = results.violations.filter(({ impact }) => impact === 'serious' || impact === 'critical');
    expect(blocking, `${path}: ${JSON.stringify(blocking)}`).toEqual([]);
  }
  await context.close();
});

test('@claim:audit-export signed-in packet history exposes retention, audit export, and confirmed deletion', async ({ page }) => {
  let deleted = false;
  let retention = 90;
  const packet = {
    id: 'packet-1', title: 'Update account contract', owner: 'owner@example.com',
    status: 'approved', created_at: '2026-08-28T10:00:00Z', approved_by: 'owner@example.com',
    approved_at: '2026-08-28T10:05:00Z', source_url: null,
    data: JSON.stringify({ source: 'Fixture', changed: ['src/api/account.ts'], checks: [{ label: 'Contract changed', detail: 'Reviewed.', state: 'done' }] }),
  };
  await page.route('**/api/**', async route => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    if (path === '/api/auth/status') return route.fulfill({ json: { authenticated: true, entra_sign_in_configured: true, github_app_configured: true, user: 'owner@example.com', team: 'Quality' } });
    if (path === '/api/packets' && request.method() === 'GET') return route.fulfill({ json: deleted ? [] : [{ id: packet.id, title: packet.title, status: packet.status }] });
    if (path === '/api/settings' && request.method() === 'GET') return route.fulfill({ json: { retention_days: retention } });
    if (path === '/api/repository-policies' && request.method() === 'GET') return route.fulfill({ json: [] });
    if (path === '/api/settings' && request.method() === 'PUT') {
      retention = JSON.parse(request.postData() || '{}').retention_days;
      return route.fulfill({ json: { retention_days: retention } });
    }
    if (path === `/api/packets/${packet.id}/audit`) return route.fulfill({ json: [{ id: 'audit-1', actor: 'owner@example.com', action: 'approved', detail: 'Owner approved this packet.', created_at: packet.approved_at }] });
    if (path === `/api/packets/${packet.id}` && request.method() === 'GET') return route.fulfill({ json: packet });
    if (path === `/api/packets/${packet.id}` && request.method() === 'DELETE') { deleted = true; return route.fulfill({ status: 204 }); }
    return route.fulfill({ status: 404, json: { error: 'Unexpected fixture request.' } });
  });
  await page.goto('/');
  await page.getByRole('button', { name: /Update account contract/ }).click();
  await expect(page.getByRole('heading', { name: 'Audit history' })).toBeVisible();
  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export packet and history' }).click();
  const exportedFile = await download;
  expect(exportedFile.suggestedFilename()).toBe('diff-gate-packet.json');
  const stream = await exportedFile.createReadStream();
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(Buffer.from(chunk));
  expect(JSON.parse(Buffer.concat(chunks).toString()).audit_history).toHaveLength(1);
  page.once('dialog', dialog => dialog.accept());
  await page.getByRole('button', { name: 'Delete packet' }).click();
  await expect(page.getByText('No saved review packets yet.')).toBeVisible();
  await page.locator('#retention-days').fill('30');
  await page.getByRole('button', { name: 'Save retention' }).click();
  await expect(page.getByText('Packets will be kept for 30 days.')).toBeVisible();
});

test('signed-in teams can start the real GitHub App setup when no installation exists', async ({ page }) => {
  await page.route('**/api/**', async route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/api/auth/status') return route.fulfill({ json: { authenticated: true, entra_sign_in_configured: true, github_app_configured: false, install_url: null, user: 'owner@example.com', team: 'Quality' } });
    if (path === '/api/packets') return route.fulfill({ json: [] });
    if (path === '/api/settings') return route.fulfill({ json: { retention_days: 90 } });
    if (path === '/api/repository-policies') return route.fulfill({ json: [] });
    return route.fulfill({ status: 404, json: { error: 'Unexpected fixture request.' } });
  });
  await page.goto('/');
  await expect(page.getByRole('link', { name: 'Create a read-only team GitHub App' })).toHaveAttribute('href', '/auth/github/new');
  await expect(page.getByText('A GitHub App installation must be bound to this Sociobot team before importing.')).toBeVisible();
});

test('header demo route and browser Back always restore the demo sandbox', async ({ page }) => {
  await mockSignedOut(page);
  await page.goto('/');
  await page.getByRole('link', { name: 'Demo' }).click();
  await expect(page).toHaveURL(/\/demo$/);
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await page.getByRole('button', { name: 'Start for real' }).click();
  await expect(page).toHaveURL(/\/$/);
  await page.goBack();
  await expect(page).toHaveURL(/\/demo$/);
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
});

test('demo approval is retained after reload once every check is resolved', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Mark reviewed' }).first().click();
  await page.getByRole('button', { name: 'Mark reviewed' }).first().click();
  await page.getByRole('button', { name: 'Approve for merge' }).click();
  await expect(page.getByText('Approved by')).toBeVisible();
  await page.reload();
  await expect(page.getByText('Approved by')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Approved' })).toBeDisabled();
});

test('navigation and footer links meet 44px touch targets at 390px', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/demo');
  for (const link of [page.getByRole('link', { name: 'Skip to content' }), page.getByRole('link', { name: 'Demo' }), page.getByRole('link', { name: 'Privacy' }).last(), page.getByRole('link', { name: 'Terms' })]) {
    const box = await link.boundingBox();
    expect(box?.width).toBeGreaterThanOrEqual(44);
    expect(box?.height).toBeGreaterThanOrEqual(44);
  }
});

test('every rendered demo control has a 44px minimum target at 390px', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/demo');
  for (const control of await page.locator('a, button, input, textarea, summary').all()) {
    const box = await control.boundingBox();
    expect(box, await control.evaluate((element) => element.outerHTML)).not.toBeNull();
    expect(box?.height).toBeGreaterThanOrEqual(44);
    expect(box?.width).toBeGreaterThanOrEqual(44);
  }
});

test('@claim:sociobot-billing shows the documented monthly plans and verifies a restored Sociobot license', async ({ page }) => {
  let verified = false;
  await mockSignedOut(page);
  await page.route('https://api.sociobot.in/api/v1/products/agent-diff-gate/verify?license=fixture-license', route => {
    verified = true;
    return route.fulfill({ json: { valid: true, reason: 'ok' } });
  });
  await page.goto('/');
  await expect(page.getByText('$12 per developer each month or $99 per team each month.')).toBeVisible();
  await expect(page.getByRole('link', { name: /Choose a Sociobot plan/ })).toHaveAttribute('href', 'https://api.sociobot.in/api/v1/products/agent-diff-gate/checkout');
  await page.getByText('Restore a paid plan').click();
  await page.locator('#license-token').fill('fixture-license');
  await page.getByRole('button', { name: 'Restore plan' }).click();
  await expect.poll(() => verified).toBeTruthy();
  await expect(page.getByText('Your Diff Gate plan is active on this device.')).toBeVisible();
});

test('invalid restored license is cleared and leaves a usable recovery path', async ({ page }) => {
  await mockSignedOut(page);
  await page.route('https://api.sociobot.in/api/v1/products/agent-diff-gate/verify?license=invalid-license', route =>
    route.fulfill({ json: { valid: false, reason: 'invalid' } }),
  );
  await page.goto('/');
  await page.getByText('Restore a paid plan').click();
  await page.locator('#license-token').fill('invalid-license');
  await page.getByRole('button', { name: 'Restore plan' }).click();
  await expect(page.getByText('This license is no longer active. Restore another license or choose a Sociobot plan.')).toBeVisible();
  await expect(page.getByRole('link', { name: /Choose a Sociobot plan/ })).toBeVisible();
  await expect(page.locator('#license-token')).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('sb_license:agent-diff-gate'))).toBeNull();
});
