# Diff Gate verification handoff

## Release status

**FAIL — candidate `eb8a164db197462ae1f62a942933ca52e095301a` is correctly deployed at `https://agent-diff-gate.sociobot.in`, but missing live Entra/GitHub configuration disables the real product.** Fresh `/health` returned this exact SHA. Fresh `/api/auth/status` returned `entra_sign_in_configured:false` and `github_app_configured:false`; consequently no team can sign in, import a real PR, create a real packet, or record an approval.

## What was verified

- All 12 exact `.factory/claims.json` commands passed after `npm ci`.
- `npm test` (16 Playwright tests), `npx tsc --noEmit`, `cargo fmt --check`, `cargo test` (12 tests), `cargo clippy -- -D warnings`, and `npm run build` passed.
- Live first-read, demo end-to-end, desktop/mobile, keyboard, reduced-motion, Axe, privacy request logging, response headers, cache policy, bundle budgets, deployment hash parity, and 40-request/second API rate limiting were independently checked.
- The demo safely resolves checks, exports JSON, records a sample approval, and resets. Its live requests were same-origin only.
- Release Docker execution was not possible because this verifier image has neither `docker` nor `podman`; native service startup with the default database was successful.

## Required next step

Provision the approved Sociobot Entra External ID client/team claim and team-bound GitHub App installation on the live service, then repeat a real production sign-in → policy save → PR import → recorded test evidence → owner approval → audit/retention/deletion flow.

See `.factory/verification-6.md` for exact commands, results, evidence paths, and the unambiguous release rationale.
