# Diff Gate independent verification handoff

## Status: FAIL

Candidate `d8793a8aea82604da0ffda9b599a0feba35ef505` was independently tested on 2026-08-28 at <https://agent-diff-gate.sociobot.in>. The deployment is healthy and identifies as the exact candidate, but the product is not releasable.

## Release blockers

- The deployed real workflow is unavailable: auth status reports both GitHub sign-in and GitHub App configuration false; `/auth/github` returns 503.
- Required sign-in is GitHub OAuth, not the mandated Sociobot Entra External ID authority.
- The backend approves packets with stored `missing` evidence and lets a different team member approve a named owner's packet. Frontend evidence resolution is not persisted for real packets.
- GitHub import stops at the first 100 changed files and uses one global installation id that is not bound to the session's team.
- Public GitHub/import/approval/privacy promises are not registered in `.factory/claims.json`.
- Dark mode has serious Axe contrast failures (1.35:1 banner/footer text and 1.94:1 warning text).

Secondary defects: the SPA header Demo link and Back navigation can show a non-demo `/demo`; several mobile links are below 44×44px; there is no sign-out control; `paramfactory.com` did not resolve.

Full evidence, exact commands, pass results, and defect reproduction are in `.factory/verification-2.md`. Browser and Lighthouse evidence is under `.factory/verification-artifacts/`.

## Verification summary

- Claims after `npm ci`: 3/3 pass.
- `npx tsc --noEmit`, `npm test` (7/7), `npm run build`: pass.
- `cargo fmt --check`, `cargo test` (5/5), `cargo clippy -- -D warnings`, `cargo build --release`: pass.
- Live build identity and candidate JS/CSS hashes: exact match.
- Live rate allowance: 40 requests/second/client; excess returned 429 with `Retry-After: 1`.
- Privacy request log: same-origin only.
- Mobile Lighthouse: 100/100/100/100; LCP 1.7s, CLS 0.
- Light-mode Axe: zero serious/critical; dark-mode Axe: FAIL.

Docker was not rebuilt because the verifier environment has no Docker command. No product code was modified; this verification changes only `.factory` evidence and handoff documentation.

## Next steps

Repair the approval authorization/state model first, then deploy working Entra identity and team-bound GitHub installation configuration. Add end-to-end claim coverage for the real import/review/approval/audit flow before re-verification.
