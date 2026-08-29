# Diff Gate handoff — 404 repair

## What changed

Repair commit replaces the static fallback's incorrect `200 OK` with `404 Not Found` while retaining the complete styled recovery page, its accessible recovery link, and its existing CSP-compatible assets. Known SPA routes (`/`, `/demo`, `/privacy`, and `/terms`), `/health` build identity, the demo sandbox, the Sociobot Entra boundary, and the 40-request-per-client-per-second API allowance are unchanged.

Regression coverage now exists at three levels:

- Rust route test `unknown_routes_return_the_designed_recovery_page_with_404_status` asserts an arbitrary unknown path responds with `404`, HTML content type, and the designed “This review desk is empty” page.
- `scripts/live-browser-smoke.mjs` asserts the browser navigation response for an unknown route is exactly `404`, while treating the expected navigation status separately from application console failures.
- `scripts/verify-live-deployment.sh` performs a black-box `curl` assertion that an unknown production path is exactly `404` before declaring a deployment valid.

## Local verification

The clean build sequence completed on 2026-08-29 UTC:

```sh
npm ci
npx tsc --noEmit
npm run build
cargo build --release
```

Vite produced `dist/` with 21.41 kB JavaScript (6.99 kB gzip) and 12.23 kB CSS (3.62 kB gzip). The compiled release server was run with an isolated SQLite database; `GET /does-not-exist` returned **HTTP 404** and the expected recovery-page heading.

The following all passed:

```sh
npm test                 # 21 Playwright tests: browser, mobile, keyboard, Axe, privacy, and offline demo
cargo fmt --check
cargo test               # 19 Rust tests, including the 404 and 40-request rate-limit assertions
cargo clippy -- -D warnings
./scripts/verify-runtime-contract.sh
node --check scripts/live-browser-smoke.mjs
sh -n scripts/verify-live-deployment.sh
```

Every exact command listed in `.factory/claims.json` was also rerun successfully. Offline update/service-worker checks do not apply: this backend-served web product makes no PWA offline-reload/update claim; the shipped offline demo interaction is covered by Playwright.

## Deploy and live verification

Deploy with:

```sh
./scripts/deploy-production.sh
node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
```

The deployment script uses the container configuration, preserves the single Azure Files-backed `/data` replica, checks the live Sociobot Entra redirect and durable storage identity, and now enforces the exact unknown-route status `404`.

## Known gaps

None. Docker is not installed in this worker image, so the local Docker command could not be run; the release binary build completed locally and the configured remote container build runs during deployment.
