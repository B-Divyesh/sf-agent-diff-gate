# Diff Gate verification 18 handoff — FAIL

**Candidate:** `e262b9d3c038725f9f40a90705733f3cfb1c9cf6`
**URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC

## Current release result

**FAIL.** Fresh production evidence differs from the earlier deployment-only
report: the root page is HTTP 200, but the actual service is currently
fail-closed. Three fresh requests each to `/health`, `/api/auth/status`, and
`/auth/entra` returned HTTP 503. Health reports
`unsafe_configuration` and build `e262b9d3c038725f9f40a90705733f3cfb1c9cf6`;
the other routes say the service is waiting for durable production storage.

This is release-blocking. It makes the cold landing page log a 503 and prevents
Sociobot Entra sign-in, GitHub App setup/import, team packet persistence, and
owner approval. The documented live rate probe observed zero accepted requests
instead of its required 40, so it could not demonstrate 429 plus `Retry-After`.
Repair the stateful production configuration, then rerun the live identity and
rate-limit probes before accepting a release.

All 20 declared claim commands passed from a clean `npm ci` checkout. `npm test`,
typecheck, Vite build, Rust fmt/test/clippy/release build, and the local
PORT-only runtime contract all passed. The live sample demo, privacy request
log, desktop/mobile keyboard flow, reduced motion, header/caching checks, and
axe serious/critical scan also passed. Docker was unavailable, so the exact
image build was not independently executed.

The full evidence and command results are in
[`verification-18.md`](verification-18.md). No product source was changed.

---

# Diff Gate repair 15 handoff — historical PASS

**Verifier report repaired:** `a396942723efdb98b610abd27612dd1523065dda`

**Failed candidate reproduced:** `cfdd80845d42ebe477b3b51664eb41a5ab48fc68`

**Repair source:** `9d0ca5989a5649e3fa7452afb8dc2316a102d8bd`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC

## Release result

**PASS.** Verification 17's release blocker was reproduced from its exact
candidate before any change. Revision `sf-agent-diff-gate--0000067` used image
`...:cfdd80845d42`, allowed one to three replicas, supplied only `PORT`, and
had no Azure Files volume or `/data` mount. The committed production verifier
rejected the resulting ten missing stateful invariants before it made any
change.

The deployed repair is revision `sf-agent-diff-gate--0000069`, built by ACR run
`ch176` as `sociobotregistry.azurecr.io/sf-agent-diff-gate:9d0ca5989a56`
with digest
`sha256:79d11709287a804db8a19131ed01bcfbdffeed1668653fe8a10644704b7470b9`.
It has Single revision mode, `minReplicas: 1`, `maxReplicas: 1`, one `data`
Azure Files volume (`agent-diff-gate-data-v4`) mounted at `/data`, the durable
SQLite URL, all public/Sociobot Entra settings, and deployment contract version
`5`.

`/health` returns build `9d0ca5989a5649e3fa7452afb8dc2316a102d8bd` and the
durable store identity `1da0c91d-ce8d-4ea1-983d-665beebfbe13`. The deployment
script created a replacement revision and the completed live verifier confirmed
one shared identity across 100 concurrent requests, plus the same identity
after replacement.

## Repair and regressions

- Added the exact verification-17 fixture: revision `0000067`, candidate image
  `cfdd80845d42`, three-replica scale, `PORT` only, no mount, and no volume.
  The regression asserts all ten contract errors and verifies one render produces
  a valid one-replica durable template.
- Raised the stateful deployment-contract marker from `4` to `5`.
- Added a server-side fail-closed guard. If the public production host receives
  traffic without the exact durable SQLite/public/Entra contract, `/health` and
  stateful API/auth routes return `503`; they cannot operate against an
  ephemeral multi-replica store. The required local `PORT`-only startup remains
  available on non-production hosts.
- Added a Rust regression that proves the recorded `PORT`-only public runtime
  returns `503 unsafe_configuration` for health and auth status.
- Stabilized the existing keyboard workflow test by waiting for the first
  review action's re-render before operating the remaining check. The product
  workflow is unchanged.

## Local verification

```text
npm ci                                                   PASS — 58 packages, 0 vulnerabilities
npm test                                                 PASS — 8 Node + 24 Playwright tests
npx tsc --noEmit                                         PASS
npm run build                                            PASS — dist/ generated
cargo fmt --all -- --check                               PASS
cargo test --all                                         PASS — 21 backend tests
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release                                    PASS
./scripts/verify-runtime-contract.sh                     PASS — clean PORT-only startup
```

All 20 exact commands in `.factory/claims.json` passed after the clean install.
The production frontend is 22.50 kB JavaScript (7.19 kB gzip) and 12.23 kB CSS
(3.62 kB gzip). This is a web application, not a published package, so a
package-consumer check does not apply. Docker is not installed in this worker,
but the exact multi-stage image build passed in ACR from a source archive that
excluded `.git`.

## Live verification

```text
./scripts/verify-live-deployment.sh \
  https://agent-diff-gate.sociobot.in '' \
  9d0ca5989a5649e3fa7452afb8dc2316a102d8bd \
  sociobotregistry.azurecr.io/sf-agent-diff-gate:9d0ca5989a56
PASS — production contract, build identity, one shared storage identity,
       40 accepted + 60 throttled requests with Retry-After: 1, Entra redirect

./scripts/verify-live-identity.sh https://agent-diff-gate.sociobot.in
PASS — Sociobot Entra only; authorization code + PKCE S256
```

`verify-url.sh` passed: HTTPS 200, title, `lang=en`, one `h1`, a `main`
landmark, complete image alt text, and no page or console errors. The live
browser smoke test passed desktop and 390×844 mobile, keyboard focus,
reduced-motion treatment, every public route, designed HTTP 404 recovery,
offline-after-load demo use, same-origin-only demo requests, and zero serious
or critical axe findings.

Live response-policy checks passed: documents use `no-cache`, hashed assets use
one-year immutable caching, anonymous packets return 401, unknown routes return
HTTP 404 with `X-Diff-Gate-Route: not-found` and `X-Robots-Tag: noindex`, and
HSTS, `nosniff`, strict-origin referrer policy, and the self-contained CSP are
present.

Fresh mobile Lighthouse results:

```text
Performance       100
Accessibility     100
Best practices    100
SEO               100
FCP                0.9 s
LCP                1.7 s
CLS                0
TBT                0 ms
Transferred        170 KiB
```

Evidence is in `.factory/repair-15-artifacts/`.

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

Do not use the generic container helper for this SQLite service. It replaces
the one-replica mounted template with independent ephemeral stores. The service
now fails closed if such a public configuration reaches it.

## Known scope limits

No signed-in test Entra account or private GitHub organization was available.
The live anonymous boundary, Entra redirect, durable storage/replacement,
rate-limit policy, browser/demo workflow, and response policy were exercised
directly. Authenticated team isolation, GitHub import and paging, revision
refresh, required-owner approval, audit conflict, retention, and deletion pass
their isolated integration claim tests.

Diff Gate does not claim installable PWA or offline reload/update support. Its
already-loaded sample remains reviewable offline, which passed.
