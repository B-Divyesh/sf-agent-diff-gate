# Diff Gate verification handoff

## Release decision

**FAIL** for candidate `586c24f96572fde8b8eef6701fdebb6210670f63` at <https://agent-diff-gate.sociobot.in>, independently verified on 2026-08-29 UTC.

The exact candidate is deployed and all claims/build/test gates pass. Release is blocked because the live deployment has neither Sociobot Entra nor the team-bound GitHub App configured, so no user can perform the real workflow. The backend also approves a packet when the client changes a check to `done` even if its stored test-evidence text still says to attach a command and result. See `.factory/verification-4.md` for exact reproduction evidence.

## What was verified

- All ten exact commands in `.factory/claims.json`: PASS after `npm ci`.
- `npm ci`, `npx tsc --noEmit`, `npm test` (13/13), and `npm run build`: PASS.
- `cargo fmt --check`, `cargo test` (11/11), `cargo clippy -- -D warnings`, and `cargo build --release`: PASS.
- Release binary startup with only `PORT`, authenticated team boundaries, owner/evidence errors, retention bounds, concurrent approval, deletion, audit, and restart persistence: PASS, apart from the evidence-content bypass above.
- Live candidate parity, demo/export/reset, request privacy, headers/caching, links/routes/404, desktop, 390px light/dark, keyboard, reduced motion, 200% text, and Axe: PASS, with one 42px skip-link target noted.
- Live rate limit: 40 requests per client per second; a 120-request burst returned 40×200 and 80×429, all with `Retry-After: 1`.
- Mobile Lighthouse: 100 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 1.7s, TBT 40ms, CLS 0, 165 KiB transfer.
- `/opt/fleet/lib/verify-url.sh` passed `/` and `/demo` with zero supported-route console/page errors.

## Required next steps

1. Provision production `ENTRA_AUTHORITY=https://sociobotcustomers.ciamlogin.com/<tenant>`, Entra client credentials/team claim, GitHub App credentials, and per-team installation mappings; then run a real production PR through import, evidence, approval, history, and deletion.
2. Replace client-controlled evidence completion with server-validated evidence content (at minimum command, result, and actor/time) and add a regression proving placeholder/empty evidence cannot be approved.
3. Implement repository-specific path/owner policy instead of only the fixed filename heuristic.
4. Register and implement the brief's Sociobot-billed plan, or approve a documented scope change. The current checkout endpoint returns 404.
5. Raise the skip link from 42px to the required 44px target.

## Artifacts and limitations

Fresh screenshots, URL checks, and Lighthouse JSON are in `.factory/verification-artifacts-4/`. No product code was modified.

Docker is not installed in this verifier container, so the image was not rebuilt locally. Native production builds passed, Dockerfile contract inspection passed, and the exact container is live. Library/CLI and service-worker checks are not applicable.
