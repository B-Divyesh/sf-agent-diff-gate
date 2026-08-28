# Diff Gate repair handoff

## Release decision

Repository-controlled findings from independent verification commit `b2e9237ffe837284861c11c402a9ab59dfae8d16` are repaired and covered. Production release remains blocked by factory-owned identity provisioning: the work order and Container App supply only `PORT`, with no Sociobot Entra application or team-bound GitHub App credentials. The repository contract forbids creating identity infrastructure from this product repo. The sample remains available; the real workflow continues to fail closed until those values are provisioned.

The expected `.factory/brief.json` is absent from the base commit and repository history. Scope was preserved from the independent report, existing README, design thesis, and passing candidate behavior.

## Repairs

- Restricted identity authorities to HTTPS on `sociobotcustomers.ciamlogin.com`, port 443, with a tenant path and no credentials, query, or fragment. Other Entra or OIDC hosts are rejected.
- Added tenant retention settings from 1 to 3,650 days with a 90-day default. Cleanup removes expired packets and audit rows at startup, hourly, and before team packet reads. Expired sessions and ten-minute OAuth state rows are also purged.
- Added confirmed packet deletion, team-scoped audit history, and audit-inclusive JSON export to the signed-in interface.
- Changed duplicate/concurrent approval recovery from a false `404` to truthful immutable-approval `409 Conflict`; exactly one approval audit row wins.
- Added a two-page, 102-file GitHub fixture proving later contract and migration paths are fetched and classified.
- Cleared `demo:diff-gate` when SPA navigation leaves demo mode; returning to `/demo` starts the shipped sample again.
- Fixed dark landing contrast at 390px and expanded dark Axe coverage across `/`, `/demo`, `/privacy`, and `/terms`.
- Limited immutable caching to hashed `/assets/*`. Stable WebP, PNG, and SVG paths use one-hour revalidation.
- Added HSTS, absolute sitemap URLs, current copy audit, retention/privacy copy, and exact claim registrations.

## Exact verification evidence

Run from a clean checkout:

```sh
npm ci
npx tsc --noEmit
npm test
npm run build
cargo fmt --check
cargo test
cargo clippy -- -D warnings
cargo build --release
```

Completed on 2026-08-28:

- `npm ci`: pass; 58 packages installed, 0 vulnerabilities.
- `npx tsc --noEmit`: pass.
- `npm test`: pass; 13/13 Playwright tests. Coverage includes every registered browser claim, desktop, 390px mobile, keyboard, offline-after-load, route history, touch targets, signed-in history/deletion/retention, light Axe, and dark Axe on all public routes.
- Every exact command in `.factory/claims.json`: pass individually.
- `npm run build`: pass; `dist/` produced. Initial JS 18,517 bytes (6.43 KB gzip), CSS 11,615 bytes (3.52 KB gzip), hero WebP 136,640 bytes.
- `cargo fmt --check`: pass.
- `cargo test`: pass; 11/11, including the simultaneous approval, retention/deletion, GitHub pagination/classification, response policy, authority restriction, team boundary, owner/evidence, rate limit, and build identity regressions.
- `cargo clippy -- -D warnings`: pass.
- `cargo build --release`: pass.
- Clean runtime: `env -i PORT=18080 target/release/diff-gate` started with generated default SQLite configuration and no other environment variables. `/health` returned `{"status":"ok","build":"dev"}`.
- Rate limit: 55 concurrent requests from one forwarded client returned 40×200 and 15×429; all 15 limited responses included `Retry-After: 1`.
- Response policy: stable `/change-control.webp` returned `public, max-age=3600, must-revalidate`; hashed JS returned `public, max-age=31536000, immutable`; unknown path returned HTTP 404; HSTS, CSP, `nosniff`, and referrer policy were present.
- `/opt/fleet/lib/verify-url.sh` passed `/` and `/demo` against the release binary with one h1, one main, title/lang/alt checks, and zero console/page errors. Evidence is under `.factory/repair-artifacts/verify-*`.
- Mobile Lighthouse: 99 performance, 100 accessibility, 100 best practices, 100 SEO; FCP 1.1s, LCP 2.0s, TBT 0ms, CLS 0, total transfer 168 KiB. JSON: `.factory/repair-artifacts/lighthouse-home.json`.
- The standalone Axe CLI could not locate its Selenium Chrome binary in this worker. The pinned Playwright 1.58.2 Axe integration ran the same axe-core serious/critical gate and passed every public route in both treatments.
- Docker is unavailable in the worker. The deployment ACR build is the container/package verification; the Dockerfile remains multi-stage, unpinned `rust:1-alpine`, non-root, and declares `BUILD_SHA` and port 8080.

## External blockers and scope

The live workflow needs these factory-provisioned settings:

```text
ENTRA_AUTHORITY=https://sociobotcustomers.ciamlogin.com/<tenant>
ENTRA_CLIENT_ID=<application id>
ENTRA_CLIENT_SECRET=<secret reference>
ENTRA_TEAM_CLAIM=extension_DiffGateTeam
GITHUB_APP_ID=<app id>
GITHUB_APP_PRIVATE_KEY=<secret reference>
GITHUB_TEAM_INSTALLATIONS={"entra:<team-id>":"<installation-id>"}
GITHUB_APP_SLUG=<app slug>
PUBLIC_BASE_URL=https://agent-diff-gate.sociobot.in
```

The injected container deployment configuration provides only `PORT=8080`; it will therefore continue to report `entra_sign_in_configured:false` and `github_app_configured:false`. Provisioning those applications would change external infrastructure and requires factory authority.

No paid UI was added. The required Sociobot checkout URL for `agent-diff-gate` returned HTTP 404 on 2026-08-28, so advertising the researched paid tier would be false and direct billing registration is prohibited from this repo. The free sample and repository code do not imply a paid entitlement.

Service-worker install/update testing and library/package-consumer testing do not apply: this remains a backend-served `web-with-backend` container with no service worker or published library.
