# Diff Gate verification 20 handoff — FAIL

**Tested candidate:** `a1eaeea89db9be13f74d8ec5ff137e104b753551`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC

## Result

**FAIL — do not release.** The deployed frontend and `/health` identify as the
tested commit, but `/health` returns 503 `unsafe_configuration` and
`/api/auth/status` reports `service_ready:false` and
`entra_sign_in_configured:false`. `/auth/entra` returns 503. The sample demo
works, but no real team can sign in or complete the GitHub-backed,
team-scoped review workflow required by the product brief.

## Verification summary

- All 20 exact `.factory/claims.json` tests passed from the clean candidate:
  seven demo Playwright claims, twelve named Rust claims, and the PORT-only
  runtime-contract script.
- `npm test` (9 Node + 25 Playwright), `npx tsc --noEmit`, `npm run build`,
  `cargo fmt --check`, `cargo test` (21), and `cargo clippy -- -D warnings`
  passed. Build output is 7.28 kB gzip JS and 3.62 kB gzip CSS.
- First-read/demo, desktop and 390px mobile, keyboard focus, reduced motion,
  Axe serious/critical, console/page errors, privacy request logging, headers,
  cache policy, and Lighthouse (98 performance / 100 accessibility) passed.
- Live rate limit passes: 40 accepted then 60 HTTP 429 responses, every one
  with `Retry-After: 1`.
- Docker is unavailable in this verifier container, so the local image build
  was not runnable; release binary and runtime contract passed.

## Release blocker

Provision the production stateful backend and credentials: exactly one durable
SQLite replica with the `/data` mount, plus Sociobot Entra and GitHub App
configuration. Re-verify live health/readiness, the Entra tenant redirect, and
a signed-in real-team packet/import/approval flow. See
`.factory/verification-20.md` for exact evidence and full QA results.

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
node deploy/live-rate-limit.mjs https://agent-diff-gate.sociobot.in
```
