# Diff Gate repair 16 handoff — PASS

**Verifier reports repaired:** `a396942723efdb98b610abd27612dd1523065dda`
(verification 17) and `063824f293b269c6647f362a7d4604f17e6b55f0`
(verification 18)

**Failed candidate reproduced:** `e262b9d3c038725f9f40a90705733f3cfb1c9cf6`

**Repair source and deployed build:**
`b5e4f2e9edaa7ab03448fd3d2a8db35817070421`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC

## Release result

**PASS.** The verification-18 production failure and the controller's newer
cold-load evidence were reproduced before the repair. The document returned
200, but each of four fresh browser contexts requested
`GET /api/auth/status`, received 503, and logged
`Failed to load resource: the server responded with a status of 503`.
`/health` and `/auth/entra` were also fail-closed, preventing the required
Sociobot Entra sign-in and every real team workflow.

The exact request is now a read-only readiness probe. In an unsafe public
topology, it returns 200 with `service_ready:false` and no database access, so
the landing page shows a clear workspace-unavailable state without a failed
resource. All stateful packet and auth routes still return 503 in that unsafe
condition. In the deployed safe topology it returns
`service_ready:true`, `entra_sign_in_configured:true`, and signed-out status.

Production was deployed only through `scripts/deploy-production.sh`. ACR run
`ch17t` built
`sociobotregistry.azurecr.io/sf-agent-diff-gate:b5e4f2e9edaa` with digest
`sha256:709c5ac6a4eeefdc76a42662e7cf6b037ca3ff83ba5f344cc677330eea7b16ac`.
The live Container App is Single revision mode with exactly one replica,
Azure Files `agent-diff-gate-data-v4` mounted at `/data`, the durable SQLite
URL, all production public/Entra settings, and deployment-contract version 5.
The replacement verifier confirmed the one durable store ID
`1da0c91d-ce8d-4ea1-983d-665beebfbe13` before and after process replacement.

## Repairs and regression coverage

- Preserved fail-closed protection for every stateful API/auth route when the
  production SQLite contract is absent.
- Made only anonymous `GET /api/auth/status` a read-only readiness response in
  that state. It cannot read or write the ephemeral store.
- Added the `service_ready` contract to the frontend and a plain recovery panel
  for a temporarily unavailable real team workspace. The one-click demo and
  signed-in workflow are unchanged.
- Added a Rust regression that recreates the `PORT`-only public deployment:
  health and packets return 503, while auth status returns the safe anonymous
  readiness JSON.
- Added a browser regression asserting that this readiness response yields a
  successful cold landing with no failed same-origin resource or console error.
- Strengthened `scripts/live-browser-smoke.mjs`: four independent cold
  contexts must each receive a 200 document, no same-origin resource status
  >=400, and no console/page error. `deploy-production.sh` now runs that check
  after the topology/replacement verifier.

## Local verification

```text
npm ci                                                   PASS — 58 packages, 0 vulnerabilities
npm test                                                 PASS — 8 Node + 25 Playwright tests
npx tsc --noEmit                                         PASS
npm run build                                            PASS — JS 22.86 kB (7.28 kB gzip), CSS 12.23 kB (3.62 kB gzip)
cargo fmt --all -- --check                               PASS
cargo test --all                                        PASS — 21 backend tests
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release                                   PASS
./scripts/verify-runtime-contract.sh                     PASS — clean PORT-only startup and health identity
```

All 20 exact commands in `.factory/claims.json` passed from the clean install.
This is a web application, not a published library or package, so a consumer
package check is not applicable. Docker is unavailable locally, but the exact
multi-stage production image built successfully in ACR from the source archive
without `.git`.

## Live verification

```text
./scripts/verify-live-deployment.sh ... b5e4f2e9... image
PASS — safe control plane; expected build; one concurrent storage identity;
       Entra PKCE; 40 accepted + 60 HTTP 429, every rejection Retry-After: 1

./scripts/verify-live-identity.sh https://agent-diff-gate.sociobot.in
PASS — Sociobot Entra only; authorization-code flow with PKCE S256

/opt/fleet/lib/verify-url.sh ...
PASS — HTTPS 200; title; lang=en; one h1; main landmark; complete image alt;
       zero browser console/page errors
```

The controller regression passed on four fresh public browser contexts. Every
attempt loaded `/`, its CSS, JS, hero art, and `/api/auth/status` at HTTP 200
with zero console errors. The full live smoke also passed desktop and 390×844
mobile, keyboard focus, reduced motion, public routes and 404 recovery,
offline-after-load demo use, same-origin-only demo traffic, and zero serious or
critical axe violations. Evidence is in `.factory/repair-16-artifacts/`.

Response-policy checks passed: documents/API use `no-cache`, hashed assets use
one-year immutable caching, anonymous packets return 401, and the designed 404
has its `not-found` and `noindex` headers. HSTS, `nosniff`, strict-origin
referrer policy, and the self-contained CSP are present.

The local Lighthouse CLI could not attach to the container's Playwright Chrome,
so no new numeric Lighthouse score is recorded. This is an environment tooling
gap, not a product failure: the deployed browser smoke, `verify-url`, and Axe
checks above completed against the live page.

## Run and deploy

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
```

Do not use the generic container helper for this SQLite service. It removes the
durable single-replica topology and Diff Gate will close its real workspace
until the contract is restored.

## Known scope limits

No signed-in test Entra account or private GitHub organization was available.
The live anonymous boundary, Entra redirect, durable storage/replacement,
rate policy, browser/demo workflow, and response policy were exercised
directly. Authenticated team isolation, GitHub import/pagination, revision
refresh, required-owner approval, audit conflict, retention, and deletion pass
their isolated integration claim tests.

Diff Gate does not claim installable PWA or offline reload/update support. Its
already-loaded sample remains reviewable offline, which passed.
