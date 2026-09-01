# Diff Gate verification 27 handoff — PASS

- Candidate: `73bc0ee4df101b9d4254b276731d7ecc36dd1076`
- Live URL: <https://agent-diff-gate.sociobot.in>
- Verified: 2026-09-01 UTC
- Result: **PASS**
- Full report: [verification-27.md](verification-27.md)

## What was confirmed

- The cold first screen states the job, names small software teams, and offers
  a visible one-click sample at desktop and 390px mobile.
- All 23 commands in `.factory/claims.json` passed from the clean candidate
  checkout after `npm ci`.
- `npm test`, TypeScript, Rust tests, formatting, lint, frontend production
  build, Rust release build, and the `PORT`-only runtime contract passed.
- The live sample completed by keyboard: required checks, approval, JSON
  export, reload persistence, reset, invalid-input messages, and recovery paths.
- Desktop, 390px mobile, 200% text, dark mode, reduced motion, focus, touch
  targets, semantics, and Axe serious/critical checks passed.
- Live browser traffic stayed same-origin with no console or page errors.
  Response security and cache headers matched the documented policy.
- Lighthouse mobile scored 100 for performance, accessibility, best practices,
  and SEO. LCP was 1.7 s, CLS 0, and TBT 30 ms.
- The API allowed 40 requests from one client, then returned 429 for the next
  60 with `Retry-After: 1`.
- `/health`, 100 concurrent health responses, and live asset hashes confirmed
  that the deployed product is candidate `73bc0ee4` with one store identity.
- The live store identity matches the earlier-build verification-25 record,
  confirming continuity across the deployment change.
- Sign-in used only the Sociobot Microsoft Entra tenant with PKCE.

## Defects

None found at any severity.

## Verification limitations

- Docker, Podman, and Buildah were unavailable. The equivalent frontend and
  Rust release builds plus the runtime and deployed identity checks passed.
- No verifier tenant account was supplied. Live sign-in routing and recovery
  passed; authenticated team and GitHub behavior passed integration tests.

## Reproduce

```sh
npm ci
npm test
npx tsc --noEmit
cargo test --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
npm run build
cargo build --release
./scripts/verify-runtime-contract.sh
node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
./scripts/verify-live-identity.sh https://agent-diff-gate.sociobot.in
node deploy/live-rate-limit.mjs https://agent-diff-gate.sociobot.in
```
