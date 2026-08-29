# Diff Gate verification 16 handoff — FAIL

**Requested candidate:** `88c39207f693df8986a96fb0754d3925496d4b6c`

**Tested checkout and live build:** `88c392a7825d7f92d2b97f7c44415532ffe5deec`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Live revision:** `sf-agent-diff-gate--0000061`

**Verified:** 2026-08-29 UTC

## Result

**FAIL. Do not release.** The requested candidate SHA cannot be found or
matched. The build that is live matches the available checkout, but production
is running three independent ephemeral SQLite stores. A 300-request live probe
returned three `storage_id` values. The source allowance is 40 requests per
client per second, but live accepted 120 before returning 429 because each
replica has an independent limiter.

The active revision permits three replicas, currently has three healthy
replicas, has no Azure Files volume or `/data` mount, and supplies only `PORT`.
`./scripts/verify-live-deployment.sh` rejects all required stateful settings.

## What passed

- All 20 exact claim commands passed after `npm ci`. The literal pre-install
  claim run failed seven browser commands because dependencies were absent;
  the 13 Rust/runtime commands passed.
- `npm test` passed 5 unit and 24 Playwright tests; TypeScript and the exact
  Vite production build passed.
- `cargo fmt`, all 20 Rust tests, clippy with warnings denied, the release
  build, and the PORT-only runtime check passed.
- The cold first-read and one-click sample gates passed.
- The live keyboard-only sample review, approval, reload, JSON export, reset,
  same-origin privacy log, desktop/mobile layouts, reduced motion, axe checks,
  console/page-error checks, headers, caching, and 404 recovery passed.
- Live assets match the checkout. Mobile Lighthouse measured LCP 1.7 s, CLS
  0, and 170 KiB transferred; category scores were 100/100/100/100 on the
  complete run.
- Sociobot Entra is the only sign-in authority and uses PKCE S256.

Docker, Podman, and Buildah are unavailable in this worker. The component
production builds and release-binary runtime contract passed. No real Entra
account or private GitHub organization was available; those boundaries passed
isolated integration tests.

## Required repair and verification

1. Resolve which SHA is the actual candidate.
2. Deploy it with `scripts/deploy-production.sh`, not the generic helper.
3. Confirm one replica, `agent-diff-gate-data-v4` mounted at `/data`, and the
   complete SQLite/public/Entra deployment environment.
4. Rerun `scripts/verify-live-deployment.sh` and require one store identity
   across concurrent traffic and a global 40-request allowance with
   `Retry-After` on 429.

Full evidence is in `.factory/verification-16.md`. Product code was not
modified during verification.
