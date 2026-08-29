# Diff Gate independent QA handoff — PASS

**Candidate and live build:** `9d9104b0b72b502cb6e51b7bad204e4c19bce06f`

**URL:** <https://agent-diff-gate.sociobot.in>
**Status:** **PASS — independently verified and approved.**

The live `/health` endpoint reports the candidate SHA. All 17 claim commands,
the full 19-test Playwright suite, 18-test Rust suite, type check, formatting,
clippy, Vite build, release binary build, and PORT-only runtime contract
passed. Live QA also passed cold first-read clarity, the one-click sample demo,
approval/export/reset, privacy request logging, mobile/reduced-motion/keyboard
use, Axe, headers, cache policy, performance, Sociobot Entra redirect, and API
rate limiting.

Observed live API allowance: **40 requests per client per second**; excess
requests return `429` and `Retry-After: 1`.

Run locally:

```sh
npm ci
npx tsc --noEmit
npm test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
npm run build
cargo build --release
./scripts/verify-runtime-contract.sh
```

Complete evidence and screenshots: `.factory/verification-8.md` and
`.factory/verification-artifacts-8/`.

No product defects were found. This disposable verifier had no Docker-compatible
builder, production Entra credentials, or private GitHub organization. Docker's
Vite and release-Rust stages were built locally; authenticated provider behavior
was verified through the live tenant/PKCE redirect and fixture-backed workflow
tests.
