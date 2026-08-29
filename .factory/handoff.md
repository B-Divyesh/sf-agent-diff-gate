# Diff Gate verification 11 handoff — FAIL

**Candidate:** `076df6c3aaf53de4b8aae83f07de857c29bfa001`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Result:** **FAIL**

Independent verification is recorded in [`.factory/verification-11.md`](verification-11.md). No product code was changed.

## Release blockers

1. **Critical — production storage is not durable.** Azure reports no volumes or volume mounts and allows 1–3 replicas. The server writes SQLite state to `/data/diff-gate.db`, so replacement loses all state and scale-out creates conflicting sessions, packets, approvals, and audit histories. The repository's `verify-live-deployment.sh` exits 1 against production.
2. **High — missing routes return HTTP 200.** The styled recovery page works, but `/not-a-real-route` and `/404` do not return the required HTTP 404 status.

## What passed

- All 20 declared claim commands after `npm ci`.
- `npm test` (24 Playwright tests), `npm run build`, `npx tsc --noEmit`, `cargo fmt --check`, `cargo test` (20), and `cargo clippy -- -D warnings`.
- PORT-only release runtime contract.
- Live candidate identity and exact JS/CSS/hero hash parity.
- Keyboard-only demo review, approval, reload, JSON export, reset, and sandbox exit.
- Same-origin-only demo requests, protected API boundaries, Sociobot Entra tenant/PKCE redirect, and 40 requests/client/second enforcement with 429 plus `Retry-After: 1`.
- Desktop and 390 px mobile layout, 200% text, reduced motion, visible focus, 44 px targets, and zero serious/critical Axe findings.
- Lighthouse: 100 in performance, accessibility, best practices, and SEO; LCP 1.71 s, TBT 6 ms, CLS 0.

The exact Docker build command could not run because the verifier image has no container runtime. The independently built frontend and Rust release binary passed. A real Entra account/private GitHub organization was unavailable, so external sign-in and installation submission were not performed; fixture-backed authenticated flows passed.

## Reverify

After correcting deployment and 404 status:

```sh
npm ci
npm test
npm run build
npx tsc --noEmit
cargo fmt --check
cargo test
cargo clippy -- -D warnings
./scripts/verify-runtime-contract.sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in --replace
node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
```

Then confirm Azure shows one replica, an Azure Files volume mounted at `/data`, the storage id survives replacement, and an unknown URL returns the styled page with HTTP 404.
