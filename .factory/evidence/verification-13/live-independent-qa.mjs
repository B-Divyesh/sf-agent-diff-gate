import assert from 'node:assert/strict';
import { writeFile } from 'node:fs/promises';
import { chromium } from '@playwright/test';

const base = 'https://agent-diff-gate.sociobot.in';
const browser = await chromium.launch({ headless: true });
const evidence = { base, checked_at: new Date().toISOString() };

const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await context.newPage();
const requests = [];
const errors = [];
page.on('request', request => requests.push(request.url()));
page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
page.on('pageerror', error => errors.push(error.message));

const homeResponse = await page.goto(base, { waitUntil: 'networkidle' });
const headline = await page.getByRole('heading', { level: 1 }).textContent();
const audience = await page.locator('.lede').textContent();
const action = page.getByRole('button', { name: 'Try it with sample data' });
const actionBox = await action.boundingBox();
assert.equal(headline, 'Review agent-authored changes before merge');
assert.match(audience ?? '', /small software teams.*required owner.*test evidence/i);
assert(actionBox && actionBox.y + actionBox.height <= 900);
await action.focus();
await page.keyboard.press('Enter');
await page.getByText('Demo — sample data, nothing is saved').waitFor();

assert.equal(await page.getByRole('heading', { name: 'Add organization-level retention controls' }).count(), 1);
for (const path of ['src/policy/retention.ts', 'db/migrations/20260828_retention.sql', 'src/api/export.ts']) {
  assert.equal(await page.getByText(path, { exact: true }).count(), 1);
}
assert.equal(await page.getByText('2 required owner checks').count(), 1);
assert.equal(await page.getByRole('button', { name: 'Approve for merge' }).isDisabled(), true);

for (let index = 0; index < 2; index += 1) {
  const resolve = page.getByRole('button', { name: 'Mark reviewed' }).first();
  await resolve.focus();
  await resolve.press('Enter');
  if (index === 0) await page.getByText('1 required owner check').waitFor();
}
assert.equal(await page.getByRole('button', { name: 'Approve for merge' }).isEnabled(), true);
await page.getByRole('button', { name: 'Approve for merge' }).click();
assert.equal(await page.getByRole('button', { name: 'Approved' }).isDisabled(), true);
await page.reload({ waitUntil: 'networkidle' });
assert.equal(await page.getByRole('button', { name: 'Approved' }).isDisabled(), true);

const downloadPromise = page.waitForEvent('download');
await page.getByRole('button', { name: 'Export packet' }).click();
const download = await downloadPromise;
const stream = await download.createReadStream();
const chunks = [];
for await (const chunk of stream) chunks.push(Buffer.from(chunk));
const packet = JSON.parse(Buffer.concat(chunks).toString());
assert.equal(packet.title, 'Add organization-level retention controls');
assert.equal(packet.changed.length, 3);
assert.equal(packet.checks.length, 4);
assert.equal(packet.status, 'approved');

await page.getByRole('button', { name: 'Reset demo' }).click();
assert.equal(await page.getByText('2 required owner checks').count(), 1);
await page.getByRole('button', { name: 'Start for real' }).click();
assert.equal(await page.evaluate(() => sessionStorage.getItem('demo:diff-gate')), null);
assert.equal(await page.getByText('Demo — sample data, nothing is saved').count(), 0);
assert.equal(errors.length, 0);
assert(requests.every(url => new URL(url).origin === new URL(base).origin));
evidence.desktop_demo = {
  headline, audience, action_box: actionBox, keyboard_launch: true,
  changed_files: packet.changed.length, checks: packet.checks.length,
  approval_persisted_after_reload: true, reset_restored_sample: true,
  start_for_real_cleared_demo_storage: true, request_count: requests.length,
  request_origins: [...new Set(requests.map(url => new URL(url).origin))], errors,
  browser_document_status: homeResponse?.status(), browser_document_headers: await homeResponse?.allHeaders(),
};
await context.close();

const mobile = await browser.newContext({ viewport: { width: 390, height: 844 }, reducedMotion: 'reduce' });
const mobilePage = await mobile.newPage();
const demoResponse = await mobilePage.goto(`${base}/demo`, { waitUntil: 'networkidle' });
const controls = await mobilePage.locator('a,button,input,textarea,summary').evaluateAll(elements => elements
  .filter(element => {
    const rect = element.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  })
  .map(element => {
    const rect = element.getBoundingClientRect();
    return { tag: element.tagName, text: element.textContent?.trim(), width: rect.width, height: rect.height };
  }));
assert(controls.every(control => control.width >= 44 && control.height >= 44));
assert.equal(await mobilePage.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
await mobilePage.keyboard.press('Tab');
assert.equal(await mobilePage.locator(':focus').isVisible(), true);
const motion = await mobilePage.locator('*').evaluateAll(elements => elements.map(element => {
  const style = getComputedStyle(element);
  return { animation: style.animationDuration, transition: style.transitionDuration };
}).filter(value => value.animation !== '0s' || value.transition !== '0s'));
assert.deepEqual(motion, []);
evidence.mobile = { viewport: '390x844', control_count: controls.length, minimum_targets_44px: true, overflow: false, visible_keyboard_focus: true, reduced_motion_nonzero_styles: motion, browser_document_status: demoResponse?.status(), browser_document_headers: await demoResponse?.allHeaders() };
await mobile.close();

const fixture = await browser.newContext({ viewport: { width: 1280, height: 900 } });
const fixturePage = await fixture.newPage();
const calls = { settings: 0, imports: 0, packets: 0, policies: 0 };
await fixturePage.route('**/api/**', async route => {
  const request = route.request();
  const path = new URL(request.url()).pathname;
  if (path === '/api/auth/status') return route.fulfill({ json: { authenticated: true, entra_sign_in_configured: true, github_app_configured: false, user: 'owner@example.com', team: 'Quality' } });
  if (path === '/api/packets' && request.method() === 'GET') return route.fulfill({ json: [] });
  if (path === '/api/settings' && request.method() === 'GET') return route.fulfill({ json: { retention_days: 90 } });
  if (path === '/api/repository-policies' && request.method() === 'GET') return route.fulfill({ json: [] });
  if (path === '/api/settings' && request.method() === 'PUT') { calls.settings += 1; return route.fulfill({ json: { retention_days: 30 } }); }
  if (path === '/api/github/import') { calls.imports += 1; return route.fulfill({ status: 400, json: { error: 'Install the team GitHub App before importing.' } }); }
  if (path === '/api/packets' && request.method() === 'POST') { calls.packets += 1; return route.fulfill({ status: 400, json: { error: 'A change title is required.' } }); }
  if (path === '/api/repository-policies' && request.method() === 'PUT') {
    calls.policies += 1;
    const body = JSON.parse(request.postData() ?? '{}');
    if (!body.rules?.length) return route.fulfill({ status: 400, json: { error: 'Add at least one sensitive path and required owner.' } });
    return route.fulfill({ json: body });
  }
  return route.fulfill({ status: 404, json: { error: 'Unexpected fixture request.' } });
});
await fixturePage.goto(base, { waitUntil: 'networkidle' });
await fixturePage.locator('#retention-days').fill('0');
await fixturePage.getByRole('button', { name: 'Save retention' }).click();
assert.equal(calls.settings, 0);
assert.notEqual(await fixturePage.locator('#retention-days').evaluate(element => element.validationMessage), '');
await fixturePage.locator('#retention-days').fill('30');
await fixturePage.getByRole('button', { name: 'Save retention' }).click();
await fixturePage.getByText('Packets will be kept for 30 days.').waitFor();
assert.equal(calls.settings, 1);
await fixturePage.locator('#pr-url').fill('not-a-url');
await fixturePage.getByRole('button', { name: 'Import pull request' }).click();
assert.equal(calls.imports, 0);
await fixturePage.locator('#packet-title').fill('');
await fixturePage.getByRole('button', { name: 'Save review packet' }).click();
assert.equal(calls.packets, 0);
await fixturePage.locator('#policy-repository').fill('acme/service');
await fixturePage.locator('#policy-rules').fill('schema/**');
await fixturePage.getByRole('button', { name: 'Save repository policy' }).click();
await fixturePage.getByText('Add at least one sensitive path and required owner.').waitFor();
await fixturePage.locator('#policy-rules').fill('schema/** | database-owner@example.com');
await fixturePage.getByRole('button', { name: 'Save repository policy' }).click();
await fixturePage.getByText('schema/** → database-owner@example.com').waitFor();
assert.equal(calls.policies, 2);
evidence.invalid_recovery = { boundary_retention_zero_blocked: true, retention_30_saved: true, malformed_url_blocked: true, empty_title_blocked: true, incomplete_policy_error: true, corrected_policy_saved: true, calls };
await fixture.close();

await browser.close();
await writeFile('.factory/evidence/verification-13/live-independent-qa.json', `${JSON.stringify(evidence, null, 2)}\n`);
console.log(JSON.stringify(evidence, null, 2));
