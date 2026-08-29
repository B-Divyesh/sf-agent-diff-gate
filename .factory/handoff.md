# Diff Gate repair handoff

## Release status

**Deployed repair `d53624a2600cd6729b8b25032d69c3f165681766`; live real-work flow remains configuration-blocked.** This repair closes the verifier's evidence-integrity, repository-policy, billing UI, and 44px-control findings. The live Container App still cannot complete a real account/import workflow until factory administrators provision the Sociobot Entra External ID client and a GitHub App installation into the approved Key Vault. No credentials were invented or committed.

## What changed

- Reproduced the exact verifier bypass: a client could save the placeholder test-evidence check as `done` and receive approval. The regression now sends that same forged payload, verifies approval returns `400`, then saves a command/result and verifies the server stores the signed-in actor and a server timestamp before approval.
- Test evidence is now a structured server boundary. `PUT /api/packets/:id` replaces client-controlled `Test evidence` checks with the server-recorded command, result, actor, and timestamp. `POST /approve` requires that structured evidence as well as resolved checks and the named owner.
- Added team-scoped repository policies at `GET/PUT /api/repository-policies`. Teams configure exact `owner/repository` policies with sensitive path rules and required owners. GitHub import refuses a repository without a policy, evaluates every changed-file page against that policy, and makes the matched owner the packet owner. Ambiguous multi-owner changes are rejected.
- Added the documented Sociobot billing surface: $12/developer/month and $99/team/month, checkout at the Sociobot billing endpoint, license return/restore, local optimistic state, and background verification. No payment-provider key is present.
- Raised the skip link, small review buttons, external links, and plan disclosure control to at least 44px. Browser regression now measures every demo control at 390px.
- Updated claims, copy audit, README, generated `dist/`, and browser coverage.

## Verification

Completed locally from a clean `npm ci` install:

- `npm ci` — 58 packages, 0 vulnerabilities.
- `npx tsc --noEmit` — pass.
- `npm test` — 16/16 Playwright tests, including desktop, 390px mobile, keyboard, demo offline interaction, privacy request recording, light/dark Axe scans across all public routes, billing restore, and 44px control measurements.
- `cargo fmt --check` — pass.
- `cargo test` — 12/12 pass. The named-approval test is the exact verifier-failure regression.
- `cargo clippy -- -D warnings` — pass.
- `npm run build` — pass; initial JS 22.58 kB (7.56 kB gzip), CSS 11.88 kB (3.55 kB gzip).
- `cargo build --release` — pass; optimized binary is 12,541,472 bytes.
- Native release service (`PORT=18080`, explicit temporary SQLite database) — `/health` returned `{"status":"ok","build":"repair-local"}`; `/opt/fleet/lib/verify-url.sh` passed `/` with a 631ms load, zero console/page errors, `lang=en`, one `<h1>`, `<main>`, and no missing image alt text.

The registered claim commands are all covered by the successful browser/Rust suites above. The new claim tests are:

- `cargo test approval_rejects_missing_evidence_and_wrong_owner_and_persists_saved_evidence`
- `cargo test repository_policy_is_team_scoped_and_requires_its_own_paths_and_owner`
- `npm run test:browser -- --grep @claim:sociobot-billing`

## Deployment and live identity limitation

The Container App is `sf-agent-diff-gate` in resource group `sociobot`. It currently has only `PORT=8080`; its Key Vault is `sociobot-keyvault1` and has no Diff Gate Entra or GitHub App secrets. The factory workload identity is assigned **Key Vault Secrets User** (read only), not a secret-write role. The provided GitHub token returns `403 Resource not accessible by personal access token` for `/user/installations`, so it cannot create or bind a GitHub App installation.

Factory administrators must add these Key Vault-backed settings to the Container App, then exercise a production sign-in, policy save, PR import, test-evidence save, approval, audit export, and deletion:

- `ENTRA_AUTHORITY=https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`
- `ENTRA_CLIENT_ID`, `ENTRA_CLIENT_SECRET`, `ENTRA_TEAM_CLAIM=extension_DiffGateTeam`, and `PUBLIC_BASE_URL=https://agent-diff-gate.sociobot.in`
- `GITHUB_APP_ID`, `GITHUB_APP_PRIVATE_KEY`, `GITHUB_APP_SLUG`, and `GITHUB_TEAM_INSTALLATIONS` mapping `entra:<team-claim>` to that team's installation id

The Entra authority was independently resolved from the public tenant metadata. The client id/secret and GitHub App private key/installation are intentionally not guessed or replaced with unrelated OAuth credentials.

The Sociobot billing endpoint was previously unregistered (`404` in the verifier report). The product now uses only the documented endpoint; factory billing registration is still required for checkout to return a hosted page.

Deployment used `/opt/fleet/lib/deploy-container.sh agent-diff-gate /work/repo Dockerfile 8080`. Azure ACR run `chr1` succeeded at 2026-08-29T00:48:02Z and the live `/health` response reports build `d53624a2600cd6729b8b25032d69c3f165681766`. A live `verify-url.sh` run passed `/` with a 613ms load and zero console/page errors; its desktop/mobile screenshots and JSON are in `.factory/repair-artifacts/live-repair/`. Live `/api/auth/status` continues to return both configuration flags as `false`, and `/auth/entra` returns the expected `503` configuration message. This is the external configuration blocker above, not a fallback identity flow.

## Run

```sh
npm ci
npm test
npm run build
cargo test
cargo run
```

Open `http://localhost:8080/demo` for the isolated sample. Real packets require the Key Vault-backed Entra and GitHub App settings above.
