# Diff Gate repair handoff

## Repaired findings

- Reproduced the verifier's exact approval failure first: the prior route accepted an owner packet with `missing` evidence. The replacement regression proves it now returns `400`, rejects a different same-team reviewer with `403`, persists a saved evidence update and audit entry, then permits only the named owner to approve.
- Replaced GitHub OAuth with Sociobot Entra External ID authorization-code configuration at `/auth/entra`. ID tokens are verified against OIDC metadata, issuer, audience, and JWKS before a session is created.
- Replaced the deployment-wide GitHub installation id with `GITHUB_TEAM_INSTALLATIONS`, keyed by the exact Entra team claim (`entra:<team-id>`). An unmapped team cannot obtain an installation token. Changed-file import now paginates all GitHub pages and refuses an unsafe import above 10,000 files.
- Added durable real-packet evidence updates, immutable approvals, a visible sign-out control, saved-packet reopening, correct demo/back routing, durable demo approval display, dark-theme contrast fixes, 44px navigation/footer targets, and removed the dead Param Factory link.
- Registered the reliance-worthy approval and Entra/team-installation claims in `.factory/claims.json` with exact Rust regression commands.

## Verification

Run from a clean checkout:

```sh
npm ci
npx tsc --noEmit
npm test
npm run build
cargo fmt --check
cargo test
cargo clippy -- -D warnings
cargo build --release
```

Completed locally on 2026-08-28:

- `npm ci`: 0 vulnerabilities reported.
- `npx tsc --noEmit`: pass.
- `npm test`: 11/11 browser tests, including dark Axe, demo history, persistent sample approval, and 390px touch targets.
- `npm run build`: pass; `dist/` produced (5.67 KB gzip JS, 3.31 KB gzip CSS).
- `cargo fmt --check`, `cargo test` (7/7), `cargo clippy -- -D warnings`, and `cargo build --release`: pass.
- Release binary smoke: `/health` returned the supplied build id and an unauthenticated approval returned `401` with the Sociobot sign-in message.

## Deployment configuration

The container remains a Rust/Axum service on `PORT=8080`; the Docker image starts without credentials for the local demo and health endpoint. The real workflow requires these deployment secrets/configuration values, which are intentionally not in the repository:

```text
ENTRA_AUTHORITY=https://sociobotcustomers.ciamlogin.com/<tenant>
ENTRA_CLIENT_ID=<Entra application id>
ENTRA_CLIENT_SECRET=<Entra application secret>
ENTRA_TEAM_CLAIM=extension_DiffGateTeam
GITHUB_APP_ID=<GitHub App id>
GITHUB_APP_PRIVATE_KEY=<GitHub App PEM>
GITHUB_TEAM_INSTALLATIONS={"entra:<team-id>":"<installation-id>"}
GITHUB_APP_SLUG=<GitHub App slug>
PUBLIC_BASE_URL=https://agent-diff-gate.sociobot.in
```

The Entra application must issue the selected team claim. The GitHub App needs only pull-request and contents read access and must be installed by each mapped team. No global installation fallback exists.

## Deployment evidence

- ACR build `chnx` produced `sociobotregistry.azurecr.io/sf-agent-diff-gate:0d0c4163186c` (digest `sha256:8623bece7abb69547772389af897fe231cfdc021d921ef43fbc4bf7302129f6e`).
- Container App `sf-agent-diff-gate` is deployed as revision `sf-agent-diff-gate--0000003`, healthy and running with that image.
- Live <https://agent-diff-gate.sociobot.in/health> returned `{"status":"ok","build":"0d0c4163186c8f78a19270a51cc13d3313596b4c"}`. Live auth status exposes `entra_sign_in_configured:false` and `/auth/entra` returns the expected configuration-only `503` until factory secrets are supplied.

## Known external dependency

This repair cannot make a production user sign in until the factory provisions the Entra and GitHub App secret values above. The live deployment should be rechecked with a real mapped team after those values are supplied; the sample demo is unaffected.
