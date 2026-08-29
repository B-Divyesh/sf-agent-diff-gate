# Diff Gate repair 17 handoff — PASS

**Verifier report repaired:** `34e3033317b4e52716f1749b7ef1a2249046f89c`
(`.factory/verification-19.md`)

**Repair commit and deployed build:**
`08bf31a80e033ca952962a608456634e459e39ea`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC

## Result

**PASS.** Verification 19’s two release blockers were reproduced before the
repair. Live revision `sf-agent-diff-gate--0000073` had only `PORT`, no
Azure Files volume or `/data` mount, and a one-to-three replica scale. The
production-contract verifier reported all ten missing durable SQLite and
public Entra invariants. `/health` returned 503 `unsafe_configuration`; the
read-only landing readiness response correctly reported `service_ready:false`.
The verifier had also observed the in-process limiter multiplied across live
replicas (80 HTTP 200 and 20 HTTP 429 from a 100-request single-client probe).

The repair was deployed through `scripts/deploy-production.sh`, never the
generic container helper. ACR run `ch18p` built
`sociobotregistry.azurecr.io/sf-agent-diff-gate:08bf31a80e03` with digest
`sha256:966cd827490d3471eb6124524979f9f5e528051d87a5047623eb9d4614276b0d`.
The live Container App is `sf-agent-diff-gate--0000075`, Single revision mode,
with exactly one replica, one `data` Azure Files volume
(`agent-diff-gate-data-v4`) mounted at `/data`, and the required durable
database/public Entra configuration. It reports build `08bf31a…e39ea` and the
same durable store identity `1da0c91d-ce8d-4ea1-983d-665beebfbe13` before and
after the deployment script’s deliberate replacement revision.

## Repair and regression coverage

- Added an exact verification-19 control-plane fixture for candidate
  `9df61fc1e555`, revision `0000073`, its PORT-only environment, absent
  volume/mount, and one-to-three scale.
- The regression asserts every required deployment-contract error, asserts the
  verifier’s exact `80` accepted / `20` throttled rate result is rejected, and
  proves the production template renderer turns that fixture into a safe
  single-replica Azure Files deployment.
- Preserved the existing fail-closed behavior for an unsafe public SQLite
  topology. The real workspace cannot silently use ephemeral, divergent data.
- Applied the checked stateful deployment workflow, restoring the unavailable
  production team workspace and the globally enforced 40-request allowance.

## Local verification

```text
npm ci                                                   PASS — 58 packages, 0 vulnerabilities
npm test                                                 PASS — 9 Node + 25 Playwright tests
npx tsc --noEmit                                         PASS
npm run build                                            PASS — JS 22.86 kB (7.28 kB gzip), CSS 12.23 kB (3.62 kB gzip)
cargo fmt --all -- --check                               PASS
cargo test --all                                         PASS — 21 backend tests
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release                                   PASS
./scripts/verify-runtime-contract.sh                     PASS — clean PORT-only startup and health identity
```

All 20 exact commands in `.factory/claims.json` passed from the clean install:
the seven isolated Playwright demo claims, twelve named Rust integration
claims, and the PORT-only runtime contract. This is a web application, not a
published package, so a package-consumer test does not apply.

The exact multi-stage production container build also passed in ACR from a
source archive excluding `.git`. A local release-binary check passed
`/opt/fleet/lib/verify-url.sh` and `scripts/live-browser-smoke.mjs`: desktop,
390×844 mobile, keyboard focus, Axe serious/critical checks, privacy request
boundaries, and an already-loaded offline demo all passed.

## Live verification

```text
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in '' \
  08bf31a80e033ca952962a608456634e459e39ea \
  sociobotregistry.azurecr.io/sf-agent-diff-gate:08bf31a80e03
PASS — safe control plane; expected build; one concurrent durable store;
       Sociobot Entra PKCE; 40 HTTP 200 + 60 HTTP 429 with Retry-After: 1

./scripts/verify-live-identity.sh https://agent-diff-gate.sociobot.in
PASS — Sociobot Entra-only authorization-code redirect with PKCE S256

node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
PASS — desktop and 390px mobile, keyboard, Axe, privacy, and offline demo

/opt/fleet/lib/verify-url.sh https://agent-diff-gate.sociobot.in <temp-dir>
PASS — HTTPS 200; title; lang=en; one h1; main; image alt; zero browser errors
```

Live response policy is correct: documents use `no-cache`, hashed JavaScript
uses `public, max-age=31536000, immutable`, and HSTS, `nosniff`, strict-origin
referrer policy, and the self-contained CSP with `frame-ancestors 'none'` are
present. Demo browser traffic was same-origin only; there is no analytics or
third-party runtime request. Live screenshots are in
`.factory/repair-17-artifacts/live/`.

Mobile Lighthouse against the live service (Chrome with `--no-sandbox`):

```text
Performance 99   Accessibility 100   Best practices 100   SEO 100
FCP 1.0 s        LCP 1.8 s            CLS 0            TBT 70 ms
```

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

Do not use the generic container helper for this stateful SQLite product. It
removes the durable one-replica topology; Diff Gate deliberately fails closed
instead of serving a split workspace.

## Known limits

No authorized test Sociobot Entra account and private GitHub organization were
available in this worker, so a full signed-in live packet/import completion was
not safely performed. The live readiness, PKCE redirect, durable replacement,
rate policy, public/demonstration workflows, and response policy were tested
directly. Team isolation, GitHub import/pagination, revision refresh,
owner-only approval, audit conflict, retention, deletion, and GitHub App
provisioning pass their isolated integration claim tests.
