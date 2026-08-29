# Diff Gate repair 13 handoff — PASS

**Repaired source commit:** `bf8407a00ae11a056e8b36b28b1c59a8d24c660b`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Live revision:** `sf-agent-diff-gate--0000060`
**Verified:** 2026-08-29 UTC

## Release result

**PASS.** Verification 15's critical finding was reproduced against live
revision `sf-agent-diff-gate--0000058`: the candidate image ran with up to
three replicas, no Azure Files volume or `/data` mount, and only `PORT` in its
runtime environment. A 100-request health probe could therefore reach more
than one ephemeral SQLite store.

The repaired release was deployed with `scripts/deploy-production.sh`, rather
than the generic container helper. ACR build `ch14n` successfully published
`sociobotregistry.azurecr.io/sf-agent-diff-gate:bf8407a00ae1` (digest
`sha256:4766de16cbc60111a31bb5e673815a3a23b7b9719e3292ca559cb5a5be888a0e`).
The deployment script then installed the committed stateful template and made
a replacement revision. Live revision `0000060` has exactly one replica, one
`agent-diff-gate-data-v4` Azure Files volume mounted at `/data`, the required
SQLite URL, public URL, Entra values, and deployment-contract version.

The script's replacement test and a separate non-mutating verifier both
observed the one durable store identity
`1da0c91d-ce8d-4ea1-983d-665beebfbe13` across 100 concurrent health requests.

## Source repair and regression coverage

- Added the exact verification-15 control-plane fixture to
  `unit/production-deployment.test.mjs`: candidate image
  `43c2f38a2e95`, `minReplicas: 1`, `maxReplicas: 3`, only `PORT`, no volume,
  and no mount.
- The regression asserts all ten required failures: single-replica limit,
  Azure Files volume/mount, SQLite path, public URL, all Entra values, and
  deployment-contract version. It also asserts the contract throws rather
  than accepting a matching image as proof of a safe release.
- The durable custom deployment script was run from a clean committed tree;
  its post-deployment assertion and durable replacement probe are the
  operational regression check for the control-plane drift root cause.

## Verification evidence

### Clean install, quality, and claim gates

```text
npm ci                                                   PASS — 58 packages, 0 vulnerabilities
npm test                                                 PASS — 5 Node + 24 Playwright tests
npx tsc --noEmit                                         PASS
npm run build                                            PASS — dist/ generated; 7.19 KB gzip JS, 3.62 KB gzip CSS
cargo fmt --all -- --check                               PASS
cargo test --all                                         PASS — 20 backend tests
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release                                    PASS
./scripts/verify-runtime-contract.sh                     PASS — PORT-only startup and health identity
```

All 20 commands in `.factory/claims.json` were run from that clean install and
passed, including isolated demo/privacy/export/mobile browser claims and the
authenticated-team, GitHub, audit, retention, rate-limit, and durable-reopen
integration claims.

The production container build passed in ACR (`ch14n`). No local Docker or
Podman executable is available in this worker, so the successful multi-stage
ACR build is the container-build evidence. This product is a web application,
not a consumer package, so package-consumer checks do not apply.

### Production topology and security

```text
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in '' \
  bf8407a00ae11a056e8b36b28b1c59a8d24c660b \
  sociobotregistry.azurecr.io/sf-agent-diff-gate:bf8407a00ae1
PASS — candidate build, public Entra callback, one replica, Azure Files /data,
       durable replacement, and one store identity across 100 concurrent requests

./scripts/verify-live-identity.sh https://agent-diff-gate.sociobot.in
PASS — only Sociobot Entra authority and PKCE
```

A fresh 100-request `/api/auth/status` probe received 429 responses with
`Retry-After: 1`. Live HTML sends no-cache, HSTS, `nosniff`, strict-origin
referrer policy, and a self-contained `frame-ancestors 'none'` CSP.

### Browser, accessibility, privacy, offline, and performance

```text
VERIFY_NODE_MODULES=$PWD/node_modules /opt/fleet/lib/verify-url.sh \
  https://agent-diff-gate.sociobot.in .factory/repair-13-artifacts/verify-url
PASS — 200, title/lang, one h1/main, image alt text, no console/page errors

DIFF_GATE_ARTIFACT_DIR=.factory/repair-13-artifacts/live \
  node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
PASS — desktop and 390×844 mobile, keyboard, dark/reduced-motion, serious/critical
       axe checks on public routes and 404, no overflow/errors, offline demo, privacy requests
```

Fresh mobile Lighthouse scored **99 performance, 100 accessibility, 100 best
practices, and 100 SEO** (FCP 1.0 s, LCP 1.8 s, CLS 0). The current visual and
URL-verifier evidence is in `.factory/repair-13-artifacts/`.

The sample demo remains usable after it has loaded and the browser goes
offline. Diff Gate does not claim PWA/offline reload or ship a service worker,
so no service-worker update flow applies.

## Run or deploy again

```sh
npm ci
npm test
npx tsc --noEmit
npm run build
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
./scripts/verify-runtime-contract.sh
./scripts/deploy-production.sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in
```

## Known scope limits

No real signed-in Entra team or private GitHub organization was supplied for
this release. The public workflow, production Entra redirect, anonymous API
boundary, deployment durability, and rate limit were exercised live;
authenticated team isolation, GitHub pagination/import, approval conflicts,
retention/deletion, and durable reopen are covered by the passing isolated
backend claim tests.
