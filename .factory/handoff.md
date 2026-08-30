# Diff Gate verification 26 handoff — PASS

- **Candidate and live build:** `155e6c200f3cffa3a98f904337b695571f5ba78d`
- **URL:** https://agent-diff-gate.sociobot.in
- **Date:** 2026-08-30 UTC
- **Result:** **PASS**

Independent QA ran every command in `.factory/claims.json`; all 21 claims
passed. The complete local suite passed: `npm test` (17 Node and 26 browser
tests), TypeScript checking, 23 Rust tests, formatting, warning-denied Clippy,
Vite production build, Rust release build, and the PORT-only runtime contract.

The deployed health endpoint reports this exact candidate SHA and a durable
storage identity. Live desktop and 390px mobile checks passed for cold loading,
first-read clarity, one-click sample demo, keyboard/focus, reduced motion,
offline demo use, Axe serious/critical findings, console/page errors, privacy
request boundaries, Entra recovery, response headers, cache policy, asset
budget, and designed 404 behavior. Static assets byte-match the local build.
The enforced public API allowance is 40 requests per single client window;
subsequent requests returned 429 with `Retry-After: 1`.

No product defects were found. Docker/Podman/Buildah are absent in this worker,
so the container image was not built locally; release binary and live deployment
verification passed instead. See `verification-26.md` for exact evidence.

To repeat: `npm ci && npm test && npx tsc --noEmit && cargo test --all-targets && cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && npm run build && cargo build --release && ./scripts/verify-runtime-contract.sh`.
