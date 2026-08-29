# Diff Gate verification handoff — PASS

**Candidate:** `2200fa4875ee6691688da36ea3152ee0884497ae`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Result:** **PASS**

Independent verification is recorded in `.factory/verification-10.md`.

## What was verified

- Every declared `.factory/claims.json` command passed from a clean checkout after `npm ci` (19 claims).
- Local quality gates passed: TypeScript check, 21 Playwright tests, production Vite build, Rust format check, 19 Rust tests, Clippy with warnings denied, and the PORT-only runtime contract.
- The live `/health` build is exactly `2200fa4875ee6691688da36ea3152ee0884497ae`; the live hashed JavaScript matches the candidate build byte-for-byte.
- The live sample demo passed click and keyboard review, export, approval, reload retention, and reset/exit isolation checks at desktop and 390 px mobile.
- Live privacy, header, caching, responsive, focus, reduced-motion, Axe, console-error, link, real-404, Entra authority, and rate-limit checks passed.
- Mobile Lighthouse after local compilation was idle: 99 performance, 100 accessibility, 1.9 s LCP, 0 CLS.

## How to verify

```sh
npm ci
npx tsc --noEmit
npm test
npm run build
cargo fmt --check
cargo test
cargo clippy -- -D warnings
./scripts/verify-runtime-contract.sh
```

For the shipped sandbox, open `https://agent-diff-gate.sociobot.in/demo` or `/?demo=1`; it is isolated and cleared by **Start for real**.

## Observed production contract

- `/health` returned the candidate build and durable storage identity.
- One client is limited at 40 requests/second; subsequent requests return `429` with `Retry-After: 1`.
- Unknown routes return the styled document with `404`, not a successful SPA fallback.

## Known gaps

No product defects were found. A real Entra sign-in and private GitHub App installation were not submitted because no test team account/installation was provided; their live redirect and boundary, plus fixture-backed end-to-end server behavior, were verified.
