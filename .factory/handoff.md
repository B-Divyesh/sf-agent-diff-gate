# Diff Gate verifier handoff 24 — FAIL

- **Work order:** `agent-diff-gate-verify-24`
- **Candidate / live build:** `e43c4da31769b958ba9b70a575f7b8fd5e3cd458`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC

## Result

**FAIL — do not release.** The earlier deployment-only problem is repaired:
the live service has the candidate build, one durable SQLite store, one
replica, correct Entra PKCE configuration, and an enforced 40-request rate
allowance. However, a canceled or denied Entra login reaches
`/auth/callback?error=access_denied` and returns raw HTTP 400 deserialization
text (`missing field code`) rather than a Diff Gate error screen with a retry
or return action. This fails the required error/recovery path for the real
signed-in workflow.

## Verification performed

- Ran every exact command in `.factory/claims.json`: all 21 passed.
- Passed `npm test` (16 Node + 25 Playwright tests), TypeScript, production
  Vite build, Rust formatting, 21 Rust tests, warning-free Clippy, release
  binary build, and the PORT-only runtime contract.
- Freshly confirmed the live service build ID, durable identity, stateful
  deployment contract, Sociobot Entra-only PKCE redirect, and the rate limit:
  40 accepted, 60 `429`, every throttled response `Retry-After: 1`.
- Exercised the live demo end to end on desktop and 390px mobile, including
  evidence checks, approval, JSON export, reset, exit, keyboard, offline use,
  privacy request logging, response headers, cache policy, 404, reduced
  motion, and Axe. Lighthouse scored 99 performance and 100 for accessibility,
  best practices, and SEO.
- Compared live root, JS, CSS, and hero-asset SHA-256 values with this
  candidate’s `dist/`; they match exactly.

## Required next step

Handle OAuth callback error parameters before requiring `code`. Render a
plain-language Diff Gate recovery page stating that sign-in did not complete,
with **Try sign-in again** and **Try it with sample data**. Add an observable
browser/integration test for Entra `error=access_denied`, then re-run
independent verification.

## Evidence and known limits

Full evidence is in [verification-24.md](verification-24.md) and
`.factory/verification-24-artifacts/`. A real tenant-user sign-in and private
GitHub installation could not be completed without a factory test identity;
their configuration and all local isolation/approval/import claims were
verified. Docker/Podman/Buildah are absent in this worker, so the local
container image build was not run; the deployed candidate identity and static
assets were independently matched.
