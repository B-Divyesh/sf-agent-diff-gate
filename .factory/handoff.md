# Independent QA handoff — FAIL

## Candidate and decision

- Candidate: `9fb9afa9361a2ff234885b49e35bb3874550156f`
- Live: <https://agent-diff-gate.sociobot.in>
- Verified: 2026-08-28
- Decision: **FAIL — do not release**

The exact candidate is deployed and the sample works well, but the real product is unavailable. Live `/api/auth/status` reports `entra_sign_in_configured:false` and `github_app_configured:false`; `/auth/entra` returns 503. Teams therefore cannot sign in or perform the brief's real GitHub review workflow.

## Release blockers

1. Configure and exercise the live Sociobot Entra External ID and team-bound GitHub App workflow. Restrict `ENTRA_AUTHORITY` to `sociobotcustomers.ciamlogin.com`.
2. Fix three serious dark-mode contrast failures on the landing page (2.06:1 primary action; 1.1:1 boundary heading/body).
3. Add configurable retention and deletion for packet/session/audit data, and document the policy.
4. Fix the false privacy statement that demo state is discarded whenever demo mode is left. Ordinary navigation retains and restores `demo:diff-gate`.
5. Register and test all public import/privacy claims, especially reading every changed path with the team-bound GitHub App.

Additional findings: simultaneous approvals return `200` plus a misleading `404`; audit records have no user-facing history/export; the researched paid tier is absent; unhashed images are cached immutable for a year; the sitemap uses relative URLs; and `.factory/copy-audit.md` is stale.

## Verification summary

Passed:

- Every command in `.factory/claims.json` after `npm ci`.
- Cold first-read and one-click sample demo.
- `npx tsc --noEmit`.
- `npm test` (11/11 browser tests).
- `npm run build` (`dist/` produced).
- `cargo fmt --check`.
- `cargo test` (7/7).
- `cargo clippy -- -D warnings`.
- `cargo build --release`.
- Release-binary startup with only `PORT`, health/build identity, input boundaries, evidence/owner enforcement, SQLite restart persistence, and a 100-request concurrent health smoke.
- Live candidate SHA and asset-hash parity.
- Live rate limit: 40 requests/client/second, then 429 with `Retry-After: 1`; health exempt.
- Same-origin demo request log, normal desktop/390px layout, keyboard/focus, reduced motion, routes, security headers, and zero console/page errors.
- Mobile Lighthouse: 98 performance / 100 accessibility; LCP 1.7s, TBT 150ms, CLS 0, 162 KiB transfer.
- Factory `verify-url.sh` on `/` and `/demo`.

Docker was not available in the verifier image, so the Dockerfile could not be rebuilt. Native production builds passed, the Dockerfile contract is structurally correct, and live build identity is the candidate SHA.

Full evidence and reproduction details are in `.factory/verification-3.md`.
