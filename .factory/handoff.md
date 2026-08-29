# Diff Gate verification 20 repair handoff — PASS

**Repair commit:** `bba39b7b5bf04d8a43f195f2eae55064b89fe835`
**Failed candidate/report:** `a1eaeea89db9be13f74d8ec5ff137e104b753551` / `535515266bf5e37e1ee6a247b22a816472804642`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified and deployed:** 2026-08-29 UTC

## Result

**PASS — the release-blocking real-team workflow is available again.** The
dedicated stateful deployment was built and applied successfully. Live health
now returns HTTP 200 for repair commit `bba39b7`, the public identity endpoint
reports `service_ready:true`, and Sociobot Entra sign-in redirects to the
configured tenant with PKCE.

## What was repaired

Verification 20 captured a candidate image running under the generic
container topology: only `PORT`, one-to-three replicas, no Azure Files
`/data` mount, and no production database or Entra settings. The application
correctly failed closed (`503 unsafe_configuration`), but that made real team
sign-in and review work unavailable.

- Added an exact verification-20 control-plane regression fixture in
  `unit/production-deployment.test.mjs`. It asserts that the captured
  `a1eaeea89db` configuration is rejected with all ten missing invariants, and
  that `renderProductionTemplate` repairs it to a valid one-replica,
  Azure-Files-backed, Entra-configured template.
- Built the multi-stage container image in ACR from the committed source and
  ran `scripts/deploy-production.sh`. It applied the image and complete
  stateful contract atomically: one replica, `agent-diff-gate-data-v4` mounted
  at `/data`, durable SQLite URL, public base URL, and Sociobot Entra runtime
  settings.

## Live evidence

- `GET /health` → HTTP 200:
  `{"status":"ok","build":"bba39b7b5bf04d8a43f195f2eae55064b89fe835","storage_id":"1da0c91d-ce8d-4ea1-983d-665beebfbe13"}`.
- `GET /api/auth/status` reports `service_ready:true`,
  `entra_sign_in_configured:true`, and `github_app_setup_available:true`.
- `GET /auth/entra` → HTTP 307 to
  `sociobotcustomers.ciamlogin.com/.../oauth2/v2.0/authorize` with the
  configured client, public callback, and `code_challenge_method=S256`.
- `./scripts/verify-live-deployment.sh` passed the Azure control-plane
  contract, live identity, designed 404, 100 concurrent health requests, and
  the global rate limit: 40 HTTP 200 then 60 HTTP 429, every rejection with
  `Retry-After: 1`. Its deliberate revision replacement retained storage ID
  `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- `node scripts/live-browser-smoke.mjs` passed on live desktop and 390px
  mobile: routes, keyboard focus, serious/critical Axe findings, console
  errors, privacy request origins, and an offline demo flow.

## Local verification

Ran from a clean dependency install:

```sh
npm ci
npm test                         # 10 Node tests; 25 Playwright tests
npx tsc --noEmit
npm run build                    # JS 7.28 kB gzip; CSS 3.62 kB gzip
cargo fmt --check
cargo test                       # 21 backend tests
cargo clippy -- -D warnings
./scripts/verify-runtime-contract.sh
```

The PORT-only runtime contract passed: the release binary starts with only
`PATH`, `PORT`, and `BUILD_SHA`, then returns its build and storage identities.
All browser claim coverage, including sample isolation/export, mobile first
action, no third-party runtime, no-merge boundary, and audit export, passed.
Backend integration covers the team boundary, named approval evidence, Entra
tenant restriction, GitHub pagination/revision/App provisioning, policy,
retention/deletion, audit concurrency, file limit, retention limits, and
durable-store replacement.

Docker is not installed in this worker, so no local Docker command was run.
The production ACR build of the repository Dockerfile succeeded and is the
container now serving the live repair.

## Remaining limits

No release blocker remains. The worker has no human Entra or GitHub account,
so it verified the live tenant-only PKCE redirect and the signed-in packet,
approval, policy, and GitHub App flows through backend integration fixtures;
creating a private GitHub App installation still requires a real signed-in
team administrator, by design.

## How to re-run

```sh
npm ci
npm test
npx tsc --noEmit
npm run build
cargo fmt --check
cargo test
cargo clippy -- -D warnings
./scripts/verify-runtime-contract.sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in
node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
```
