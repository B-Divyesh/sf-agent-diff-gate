# Diff Gate verification 21 handoff — FAIL

**Candidate:** `ce5bf429b0b5bf119773fd50eee846ff69c97612`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-30 UTC

## Result

**FAIL — do not release.** Fresh evidence shows that the live deployment is
this exact candidate, but its backend is unsafe-configured and deliberately
fail-closed. `/health` returns 503, `/api/auth/status` says
`service_ready:false` and `entra_sign_in_configured:false`, and `/auth/entra`
returns 503. Real teams cannot use the product's required GitHub review packet
workflow.

The complete evidence and exact commands are in
[`verification-21.md`](verification-21.md). No product code was modified.

## What verified successfully

All 20 claims passed from a clean install, including the isolated sample demo,
packet export, no-third-party-runtime check, team boundary, approval evidence,
Entra restriction, GitHub import behavior, retention, audit history, and the
PORT-only runtime contract. `npm test` (10 unit + 25 Playwright), typecheck,
production build, Rust format/test/clippy all passed. Live desktop and 390px
mobile checks passed for accessibility, keyboard focus, reduced motion, no
console errors, offline demo behavior, headers, caching, privacy request
origins, 404 recovery, and the 40-request-per-client rate allowance (then 429
with `Retry-After: 1`).

## Next step

Deploy the candidate with durable storage and required Sociobot Entra/GitHub
App configuration, then repeat the live identity and signed-in real-team
workflow verification. The sample demo alone is not sufficient for release.
