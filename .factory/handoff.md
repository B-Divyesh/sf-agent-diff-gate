# Diff Gate polish 3 handoff

- **Repair commit:** `6095165c7993ff7d3c63de894d8ad74ac16f37d4`
- **Built-routing follow-up:** `71bf3ad7a4a00d6cf753980cf478cb22620f3cc7`
- **Branch pushed:** `main` at `71bf3ad`
- **Review closure:** [`.factory/polish-3.md`](polish-3.md)

## What changed

- Direct missing-page browser navigation no longer emits a console resource
  error. It receives a designed, noindex recovery document with explicit
  `X-Diff-Gate-Route: not-found`; non-navigation requests still receive HTTP
  404.
- Removed the live-smoke console exception and added exact backend, browser,
  and header-contract regressions.
- Rewrote README deployment copy in plain language. Each deployment statement
  now has one exact claim and test for the rendered single-replica template,
  product-and-port hook boundary, or paired 100-response replacement check.
- Fixed the pre-existing duplicate `@claim:sample-sandbox` tag so each tagged
  claim has one test. Updated the catalog sentence to a verb-first 80-byte
  description.

## Verification

- Clean clone `/tmp/agent-diff-gate-clean-F4JRl4` at `71bf3ad`: `npm ci`, then
  every one of the 23 `claims.json` commands separately — **23/23 PASS**.
- `npm test`: **17 Node tests + 27 Playwright tests PASS**. This includes
  keyboard, mobile 390 px, 200% text, demo isolation/reset/export/offline,
  route metadata, privacy request boundaries, and light/dark Axe checks.
- `npm run build`: PASS; `dist/` produced 7.28 kB gzip JavaScript.
- `npx tsc --noEmit`, `cargo test --all-targets` (24 tests),
  `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and
  `./scripts/verify-runtime-contract.sh`: PASS.
- Local exact-server evidence: [mobile recovery](polish-3-artifacts/local-404-mobile.png)
  and [mobile demo](polish-3-artifacts/local-demo-mobile.png). The recovery
  browser response was 200/noindex with no console errors; a non-navigation
  request returned 404.

## Deployment and live check

The public service still reported prior build
`155e6c200f3cffa3a98f904337b695571f5ba78d` when checked after the push. The
work-order container configuration must deploy `71bf3ad`; this repair did not
invoke the legacy repository deploy script because it accesses shared factory
infrastructure outside the allowed product boundary. After the configured
deployment, run `scripts/live-browser-smoke.mjs` against
<https://agent-diff-gate.sociobot.in>, confirm the deployed build from
`/health`, and rerun the public 404, demo, accessibility, privacy, and offline
checks listed in `polish-3.md`.

Docker is unavailable in this worker (`docker: command not found`), so the
factory's container build is the remaining Docker verification step. There are
no known source or product-behavior gaps.
