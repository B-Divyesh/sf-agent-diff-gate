# Diff Gate repair 12 handoff — PASS

**Repaired source commit:** `b4c281abd9397ff7ec986314e3a12bfb2ddd2d0c`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Live revision:** `sf-agent-diff-gate--0000057`
**Verified:** 2026-08-29 UTC

## Release result

**PASS.** Verification 14's release blocker was live Container App drift: the
candidate image was running as up to three replicas with no shared SQLite
volume or production runtime contract. It could therefore serve different
team requests from separate ephemeral databases.

`scripts/deploy-production.sh` built and deployed the repaired commit through
ACR, then installed the committed stateful template. The live deployment now
has exactly one replica, Azure Files volume `agent-diff-gate-data-v4`, a single
`/data` mount, and the required `DATABASE_URL`, public URL, Entra, and
deployment-version values. The script's replacement probe and a second
non-mutating verifier run both observed the same storage id:
`1da0c91d-ce8d-4ea1-983d-665beebfbe13`.

## Source repair

- Tightened `deploy/production-contract.mjs`: a managed environment value must
  occur exactly once, and the `data` volume and mount must each occur exactly
  once with their required values. An extra managed variable or a second data
  mount can no longer mask an unsafe Container App template.
- Updated the regression fixture to the exact verifier-14 shape: candidate
  image, `minReplicas: 1`, `maxReplicas: 3`, only `PORT`, and no volume.
  The test asserts every one of the ten reported contract errors.
- Added regression coverage for duplicate `DATABASE_URL` and duplicate data
  mounts. Both must fail the production contract.

## Verification evidence

### Local and clean-install gates

```text
npm ci                                                   PASS — 58 packages, 0 vulnerabilities
npm test                                                 PASS — 4 Node deployment tests, 24 Playwright tests
npx tsc --noEmit                                         PASS
npm run build                                            PASS — dist/ generated
cargo fmt --all -- --check                               PASS
cargo test --all                                         PASS — 20 backend tests
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release                                    PASS
./scripts/verify-runtime-contract.sh                     PASS — PORT-only startup, durable store identity
```

All 20 exact commands in `.factory/claims.json` were also run from the clean
install and passed. That includes the browser demo/privacy/export/mobile
claims, all authenticated-team and GitHub fixture claims, and the runtime and
durable-reopen claims.

No local Docker-compatible executable was installed. The required multi-stage
container build was instead verified by the successful ACR build `ch13r`, which
published `sociobotregistry.azurecr.io/sf-agent-diff-gate:b4c281abd939`.

### Live deployment and behavior

```text
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in
PASS — candidate build, one replica, Azure Files /data, Entra callback,
       100 concurrent requests with one storage identity

./scripts/verify-live-identity.sh https://agent-diff-gate.sociobot.in
PASS — only Sociobot Entra authority, production callback, PKCE S256

node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
PASS — desktop and 390×844 mobile, dark/reduced-motion mobile, keyboard focus,
       no console/page errors, no horizontal overflow, axe serious/critical
       checks on /, /demo, /privacy, /terms, and the 404 page; offline demo
       review/export and same-origin-only requests
```

The live `/health` response is the repaired source build and returns the
storage identity above. Live headers confirm `no-cache` documents, immutable
hashed scripts (`max-age=31536000`), revalidated stable images (`max-age=3600`),
HSTS, `nosniff`, strict-origin referrer policy, and the self-contained CSP.
A 100-request anonymous API burst from one forwarded IP returned 72×401 and
28×429; the 429 responses included `Retry-After: 1`.

Current desktop and 390px mobile browser captures are in
`.factory/repair-12-artifacts/live/`.

## Deploy and verify again

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
this repair. Live public behavior, the production Entra redirect, and anonymous
API boundaries were exercised; team isolation, GitHub pagination/import,
approval conflicts, retention, deletion, and durable reopen are covered by the
passing isolated backend integration tests.
