# Diff Gate polish 2 handoff

**Repair commits:** `a34f12d3a5f41c6eb86458f89a691e8c620b3b17` (product) and `9b5b69bffc133f07c696cdec1dfb95a528c3f5df` (handoff)  
**Live URL:** <https://agent-diff-gate.sociobot.in>

## Delivered

- Closed every finding in `.factory/review-1.md` and `.factory/review-2.md`; the finding-by-finding record is in `.factory/polish-2.md`.
- Preserved the dithered change-control visual system while clarifying the first-screen action and README language.
- Kept `?demo=1` and `/demo` isolated with the persistent reset/start-for-real banner.
- Made JSON export proof parse the actual download and inspect sample content.
- Added complete absolute social metadata and a full standalone 404 document.
- Replaced missing-route console suppression with an explicit successful recovery navigation (`X-Diff-Gate-Route: not-found`, `X-Robots-Tag: noindex`) and an unfiltered console regression test.
- Bound GitHub-imported packets to their displayed head SHA. Refresh and approval both recheck GitHub; a changed revision clears prior evidence and blocks approval.

## Verification before deployment

- `npm ci && npm test` — 24 Playwright tests passed, including keyboard, mobile, offline, privacy, metadata, routing, and Axe checks.
- `npm run build` — passed; initial JavaScript gzip size: 7.19 kB.
- `cargo test` — 20 tests passed.
- `cargo fmt --check` and `cargo clippy -- -D warnings` — passed.
- `./scripts/verify-runtime-contract.sh` — passed with a PORT-only runtime.
- `node scripts/live-browser-smoke.mjs http://127.0.0.1:18082` — passed desktop/mobile, Axe, keyboard, offline-demo, privacy, and unfiltered missing-route console checks. Screenshots: `.factory/repair-6-artifacts/live-desktop.png` and `.factory/repair-6-artifacts/live-mobile.png`.
- Clean clone `/tmp/diff-gate-clean-3XtSqh`: `npm ci`, then all 20 commands from `.factory/claims.json`, passed. Transcript: `/tmp/diff-gate-clean-claims.log`.

## Deployment and live recheck

- Deployed through `scripts/deploy-production.sh`; live `/health` reports build `9b5b69bffc133f07c696cdec1dfb95a528c3f5df` and the durable storage identity.
- Cold live `node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in` passed desktop/mobile, Axe, keyboard, offline-demo, privacy, and missing-route console checks.
- `./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in` passed the public PKCE identity, one-replica, Azure Files `/data`, and durable-replacement checks.

## Known gaps

None.
