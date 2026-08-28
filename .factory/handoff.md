# Diff Gate repair handoff

## Repaired release blockers

- Replaced the sample-only real path with an authenticated GitHub workflow. GitHub OAuth (`read:user read:org`) establishes a server-side, secure, expiring session and organization boundary; the GitHub App uses a short-lived installation token to import an installed repository’s PR and changed paths.
- Added default policy evaluation for contract/API paths and migrations, editable manual packets, durable named approval, and a durable packet audit record.
- Closed the public packet API: every packet create/list/read/approve/import route now requires a session and every SQL packet lookup is constrained to its team id. Cross-team lookups intentionally return 404.
- Kept the `/demo` sample fully local. It does not call the packet API or require GitHub configuration.
- Removed the unimplemented paid-plan, checkout, and license UI rather than claiming an unlock lifecycle that does not exist.
- Kept per-client limiting ahead of all non-health endpoints; it keys on the first forwarded address (or `X-Real-IP`) and returns `429` with `Retry-After: 1` after 40 requests per second.
- Changed the image build stage to the required unpinned `rust:1-alpine`, added immutable caching for hashed/static assets, and made unknown server routes HTTP 404.

## Verification

Run from a clean install:

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

The backend test suite includes exact regressions for unauthenticated packet rejection, cross-team read/approval rejection plus durable approval audit, forwarded-IP rate limiting with `Retry-After`, strict GitHub PR URL validation, and the unpinned Docker Rust stage. Browser tests cover both claims, sample isolation/reset, 390px layout, keyboard review, offline use after load, and serious/critical Axe violations.

## Deployment and GitHub setup

ACR build `chm6` built and pushed `sociobotregistry.azurecr.io/sf-agent-diff-gate:43b25f8`. Container App `sf-agent-diff-gate` was updated to that image as revision `sf-agent-diff-gate--0000001`. Live `/health` reports `{"status":"ok","build":"43b25f8"}`. Live `/demo` passed `verify-url.sh`: 711 ms load, zero console errors, title/lang/main/one h1, and no image missing alternate text. Live checks also confirmed immutable WebP cache headers, real HTTP 404, unauthenticated API `401`, and 100 concurrent packet requests from one client gave 90 x `401` and 10 x `429`.

The container intentionally starts with only `PORT` and no GitHub configuration; `/demo` and `/health` work in that mode. To enable real teams, the factory must configure these deployment secrets/settings outside the repository: `GITHUB_OAUTH_CLIENT_ID`, `GITHUB_OAUTH_CLIENT_SECRET`, `GITHUB_APP_ID`, `GITHUB_APP_PRIVATE_KEY`, `GITHUB_APP_INSTALLATION_ID`, `GITHUB_APP_SLUG`, and `PUBLIC_BASE_URL`. The GitHub App must have only read access to pull requests and contents and be installed by the team.

No local Docker daemon was available; the successful ACR build is the production-image verification.

## Known limits

The deployed instance has no GitHub OAuth/App secrets, so the real sign-in/import action correctly reports that it is not configured instead of exposing packet data. Once the factory provisions those secrets, active GitHub organizations become the tenant boundary; users without an active organization receive a private workspace. A production role-management flow is the next extension. No payment feature is offered.
