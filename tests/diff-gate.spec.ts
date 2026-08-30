import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const signedOut = { authenticated: false, entra_sign_in_configured: false, github_app_configured: false };
async function mockSignedOut(page: import('@playwright/test').Page) {
  await page.route('**/api/auth/status', route => route.fulfill({ json: signedOut }));
}
async function waitForFinalStyles(page: import('@playwright/test').Page) {
  await expect.poll(() => page.locator('.small-button').first().evaluate(element => {
    const style = getComputedStyle(element);
    return `${style.color}/${style.backgroundColor}`;
  })).toMatch(/rgb\(23, 33, 43\)\/rgb\(247, 201, 72\)/);
}

test('@claim:sample-sandbox keeps sample data isolated and discards it when demo mode ends', async ({ page }) => {
  const requests:string[]=[]; page.on('request', r=>requests.push(r.url()));
  await page.goto('/demo');
  await expect(page.getByRole('heading',{name:'See an agent-authored change under review'})).toBeVisible();
  await expect(page.getByRole('heading',{name:'Add organization-level retention controls'})).toBeVisible();
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await page.getByRole('button',{name:'Mark reviewed'}).first().click();
  await page.getByRole('link',{name:'Privacy'}).first().click();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:diff-gate'))).toBeNull();
  await page.getByRole('link',{name:'Demo'}).click();
  await expect(page.getByText('2 required owner checks')).toBeVisible();
  expect(requests.every(url=>new URL(url).origin==='http://127.0.0.1:4173')).toBeTruthy();
});
test('@claim:demo-query-path opens an isolated sample with banner controls and reset', async ({ page }) => {
  await page.goto('/?demo=1');
  await expect(page).toHaveTitle('Demo — Diff Gate');
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Reset demo' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Start for real' })).toBeVisible();
  await page.getByRole('button', { name: 'Mark reviewed' }).first().click();
  await expect(page.getByText('1 required owner check')).toBeVisible();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.getByText('2 required owner checks')).toBeVisible();
});
test('@claim:mobile-first-action keeps the sample action inside the first phone screen', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockSignedOut(page);
  await page.goto('/');
  const action = page.getByRole('button', { name: 'Try it with sample data' });
  const box = await action.boundingBox();
  expect(box).not.toBeNull();
  expect((box?.y || 0) + (box?.height || 0)).toBeLessThanOrEqual(844);
  await action.click();
  await expect(page).toHaveURL(/\?demo=1$/);
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
});
test('regression: an unsafe deployed topology returns a clean read-only readiness response on cold landing', async ({ page }) => {
  const responses: Array<{ path: string; status: number }> = [];
  const errors: string[] = [];
  page.on('response', response => {
    const url = new URL(response.url());
    if (url.origin === 'http://127.0.0.1:4173') responses.push({ path: url.pathname, status: response.status() });
  });
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  await page.route('**/api/auth/status', route => route.fulfill({
    status: 200,
    json: {
      service_ready: false,
      authenticated: false,
      entra_sign_in_configured: false,
      github_app_configured: false,
    },
  }));

  await page.goto('/', { waitUntil: 'networkidle' });

  await expect(page.getByRole('heading', { name: 'Team workspace is temporarily unavailable' })).toBeVisible();
  expect(responses.find(response => response.path === '/api/auth/status')).toEqual({ path: '/api/auth/status', status: 200 });
  expect(responses.every(response => response.status < 400)).toBeTruthy();
  expect(errors).toEqual([]);
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
  await expect(page.getByRole('heading', { name: 'Add organization-level retention controls' })).toBeVisible();
  await expect(page.getByText('pnpm test: 214 passed · migration check: passed')).toBeVisible();
  await expect(page.getByText('db/migrations requires database owner sign-off.')).toBeVisible();
  const download=page.waitForEvent('download'); await page.getByRole('button',{name:'Export packet'}).click();
  const file=await download; expect(file.suggestedFilename()).toBe('diff-gate-packet.json');
  const stream = await file.createReadStream();
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(Buffer.from(chunk));
  const packet = JSON.parse(Buffer.concat(chunks).toString());
  expect(packet.title).toBe('Add organization-level retention controls');
  expect(packet.changed).toEqual([
    'src/policy/retention.ts',
    'db/migrations/20260828_retention.sql',
    'src/api/export.ts',
  ]);
  expect(packet.checks).toHaveLength(4);
  expect(packet.checks.map((check: { label: string }) => check.label)).toEqual([
    'Contract changed', 'Migration found', 'Test evidence', 'Risky path',
  ]);
});
test('@claim:no-merge-action records a demo decision without calling a code-hosting service', async ({ page }) => {
  const requests: string[] = [];
  page.on('request', request => requests.push(request.url()));
  await page.goto('/?demo=1');
  await page.getByRole('button', { name: 'Mark reviewed' }).first().click();
  await page.getByRole('button', { name: 'Mark reviewed' }).first().click();
  await page.getByRole('button', { name: 'Approve for merge' }).click();
  await expect(page.getByRole('button', { name: 'Approved' })).toBeDisabled();
  expect(requests.every(url => new URL(url).origin === 'http://127.0.0.1:4173')).toBeTruthy();
  await expect(page.getByRole('button', { name: /merge/i })).toHaveCount(0);
});
test('keyboard review can resolve every flagged check', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button',{name:'Mark reviewed'}).first().focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('button',{name:'Mark reviewed'})).toHaveCount(1);
  await page.getByRole('button',{name:'Mark reviewed'}).focus();
  await page.keyboard.press('Enter');
  const approval = page.getByRole('button',{name:'Approve for merge'});
  await expect(approval).toBeEnabled();
  await approval.focus();
  await expect(approval).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('button', { name: 'Approved' })).toBeDisabled();
});

test('demo state is isolated in its namespace and reset restores the shipped packet', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button',{name:'Mark reviewed'}).first().click();
  await page.reload();
  await expect(page.getByText('1 required owner check')).toBeVisible();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:diff-gate'))).toContain('Migration found');
  await page.getByRole('button',{name:'Reset demo'}).click();
  await expect(page.getByText('2 required owner checks')).toBeVisible();
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
  await expect(page.getByText('1 required owner check')).toBeVisible();
  await context.setOffline(false);
});

test('demo has no serious or critical axe findings', async ({ page }) => {
  await page.goto('/demo');
  await waitForFinalStyles(page);
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
    if (path === '/demo') await waitForFinalStyles(page);
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
    if (path === '/demo') await waitForFinalStyles(page);
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
  await expect(page).toHaveURL(/\?demo=1$/);
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await page.getByRole('button', { name: 'Start for real' }).click();
  await expect(page).toHaveURL(/\/$/);
  await page.goBack();
  await expect(page).toHaveURL(/\?demo=1$/);
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

test('public routes set their own plain-language title, canonical URL, and focused h1', async ({ page }) => {
  await mockSignedOut(page);
  for (const expected of [
    ['/', 'Diff Gate — Review agent-authored changes before merge', '/'],
    ['/?demo=1', 'Demo — Diff Gate', '/demo'],
    ['/privacy', 'Privacy — Diff Gate', '/privacy'],
    ['/terms', 'Terms — Diff Gate', '/terms'],
  ] as const) {
    await page.goto(expected[0]);
    await expect(page).toHaveTitle(expected[1]);
    await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', `http://127.0.0.1:4173${expected[2]}`);
    await expect(page.locator('h1')).toBeFocused();
  }
});

test('public routes declare complete absolute social metadata', async ({ page }) => {
  await mockSignedOut(page);
  for (const path of ['/', '/?demo=1', '/privacy', '/terms']) {
    await page.goto(path);
    for (const selector of [
      'meta[property="og:title"]', 'meta[property="og:description"]',
      'meta[property="og:image"]', 'meta[name="twitter:title"]',
      'meta[name="twitter:description"]', 'meta[name="twitter:image"]',
    ]) await expect(page.locator(selector)).not.toHaveAttribute('content', '');
    await expect(page.locator('meta[property="og:image"]')).toHaveAttribute('content', 'http://127.0.0.1:4173/social.webp');
    await expect(page.locator('meta[name="twitter:image"]')).toHaveAttribute('content', 'http://127.0.0.1:4173/social.webp');
  }
});

test('unknown route renders the recovery view without console errors', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto('/missing-review');
  await expect(page.getByRole('heading', { name: 'Page not found' })).toBeVisible();
  expect(errors).toEqual([]);
});

test('dedicated 404 document keeps the public header and metadata', async ({ page }) => {
  await page.goto('/404.html');
  await expect(page.getByRole('heading', { name: 'Page not found' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'How it works' })).toBeVisible();
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://agent-diff-gate.sociobot.in/404');
  await expect(page.locator('link[rel="apple-touch-icon"]')).toHaveAttribute('href', '/apple-touch-icon.png');
  await expect(page.locator('meta[property="og:image"]')).toHaveAttribute('content', 'https://agent-diff-gate.sociobot.in/social.webp');
  await expect(page.locator('meta[name="twitter:image"]')).toHaveAttribute('content', 'https://agent-diff-gate.sociobot.in/social.webp');
});

test('regression: canceled Entra callback renders an accessible recovery screen', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const errors: string[] = [];
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });

  const response = await page.goto('http://127.0.0.1:4174/auth/callback?error=access_denied&error_description=User%20cancelled');

  expect(response?.status()).toBe(200);
  await expect(page).toHaveTitle('Sign-in did not complete — Diff Gate');
  await expect(page.getByRole('heading', { level: 1, name: 'Sign-in did not complete' })).toBeVisible();
  await expect(page.getByText('Sign-in was cancelled or your account did not grant access.')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Try sign-in again' })).toHaveAttribute('href', '/auth/entra');
  await expect(page.getByRole('link', { name: 'Return to Diff Gate' })).toHaveAttribute('href', '/');
  await expect(page.getByRole('link', { name: 'Try it with sample data' })).toHaveAttribute('href', '/?demo=1');
  await expect(page.getByText(/missing field `code`/)).toHaveCount(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBeTruthy();
  const results = await new AxeBuilder({ page }).analyze();
  const blocking = results.violations.filter(({ impact }) => impact === 'serious' || impact === 'critical');
  expect(blocking).toEqual([]);
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to content' })).toBeFocused();
  expect(errors).toEqual([]);
  await context.close();
});
