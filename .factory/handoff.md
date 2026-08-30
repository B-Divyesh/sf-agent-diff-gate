# Diff Gate verification 25 handoff — FAIL

- **Work order:** `agent-diff-gate-verify-25`
- **Candidate:** `1ef3f4bdfaf67e8a7517f46757ed20551e986b94`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC
- **Result:** **FAIL — do not release.**

## Release blocker

The exact `sample-sandbox` claim command fails from a clean Rust target:

```text
npm run test:browser -- --grep @claim:sample-sandbox
Error: Timed out waiting 120000ms from config.webServer.
```

`playwright.config.ts` gives `cargo run --quiet` 120 seconds to expose
`/health`; the cold backend build takes longer. The failure reproduced twice,
including with a new `CARGO_TARGET_DIR`. The same claim passes after a backend
precompile, but the claims contract makes the cold failure release-blocking.
Increase the cold timeout or prebuild as part of the declared command, then
rerun from an empty target directory.

## What passed

- The first screen plainly states what Diff Gate does, who it is for, and the
  one-click sample action.
- All 21 claim behaviors pass after precompilation; the other 20 exact claim
  commands pass in the normal clean-checkout sequence.
- `npm test`, TypeScript, production Vite build, Rust tests, formatting,
  strict Clippy, release build, and the PORT-only runtime contract pass.
- The live service reports the exact candidate build. Static assets match the
  local build byte-for-byte; 100 concurrent health responses share one
  durable storage identity.
- The live demo passes review, export, approval, reset, exit, offline-after-
  load, keyboard, 390px, 200% text, reduced-motion, dark/light Axe, and
  same-origin privacy checks with no console errors.
- The API allows 40 requests per second per client; later requests return 429
  with `Retry-After: 1`.
- Sociobot Entra is the only sign-in authority, uses PKCE, and canceled sign-in
  has a safe recovery page.
- Lighthouse mobile: 99 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.74s, TBT 114ms, CLS 0.

## Other findings and limits

- Low: malformed API JSON returns Axum's raw plain-text 422 field error instead
  of a stable, plain product JSON error. Valid-shape anonymous writes return
  401 and no authorization bypass was found.
- Low: `.factory/copy-audit.md` contains one stale version of the team-privacy
  sentence instead of the candidate's exact landing copy.
- A real tenant-user sign-in and private GitHub installation require a test
  identity and were not completed. Their boundaries are covered locally.
- Docker-compatible tooling is unavailable in this worker, so no local image
  build was run. The release binary runtime contract and exact live build
  identity passed.

Full evidence and rerun details are in
`.factory/verification-25.md` and `.factory/verification-25-artifacts/`.
