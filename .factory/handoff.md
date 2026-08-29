# Diff Gate repair handoff

## Release status

Repair code commit `0d6266db97ac4032383c9b58f73b61a1371d0c92` is deployed at <https://agent-diff-gate.sociobot.in> as Container App revision `sf-agent-diff-gate--0000021`. `/health` returns that exact SHA.

The verifier's failure was reproduced before repair: the report revision returned `entra_sign_in_configured:false` and `github_app_configured:false`, and `/auth/entra` returned 503. The deployment helper had replaced the app template with `PORT` only. The backend also expected a confidential Entra client secret even though the approved Sociobot application is a public SPA client.

## Repairs

- Entra now uses the approved Sociobot External ID tenant and client, authorization-code PKCE, one-use state, a separate nonce, RS256, discovery issuer, JWKS signature, audience, tenant, expiry, and nonce validation. No alternate identity provider or Entra client secret is accepted.
- The production redirect is `https://agent-diff-gate.sociobot.in/auth/callback`. A live request reached the real Sociobot sign-in policy with `AADSTS50058` (no worker browser session), not redirect error `AADSTS50011`.
- A signed-in Entra team can create a private GitHub App through GitHub's App Manifest flow. It requests only read access to pull requests, repository contents, and metadata. App setup state is team-bound. The returned App id and private key are stored server-side. An installation is accepted only after the App-authenticated GitHub installation endpoint confirms that it belongs to the same App.
- The existing private-PR import, complete changed-file pagination, repository policy, server-recorded evidence, named-owner approval, audit export, retention, and deletion paths remain intact.
- `scripts/deploy-production.sh` now reapplies the approved non-secret Entra settings after the factory deployment helper, provisions the product data share, mounts `/data`, fixes the SQLite deployment to one replica, and runs the live identity regression.
- Production state uses the dedicated `agent-diff-gate-data-v4` Azure Files share and SQLite's single-process `unix-none` VFS. A stored PKCE state survived a live replica restart and was consumed by the callback afterward.
- Anonymous `/api/auth/status` now reports `entra_sign_in_configured:true` and `github_app_setup_available:true`. `github_app_configured:false` is expected until the signed-in team finishes its own App installation.

## Regression coverage

- `cargo test live_identity_defaults_to_sociobot_and_uses_pkce_without_a_client_secret` checks the fixed tenant/client, registered callback, S256 challenge, and absence of a client secret.
- `cargo test github_app_manifest_is_read_only_and_bound_to_the_signed_in_team` checks read-only permissions and rejects cross-team setup-state reuse.
- The existing GitHub import test reads 102 paths across two fixture pages. Existing integration tests cover team boundaries, owner-only approval, evidence integrity, policy isolation, audit conflicts, retention/deletion, rate policy, response policy, and Docker identity.
- A browser regression confirms a signed-in team without an installation receives the real GitHub App setup action.
- `.factory/claims.json` now registers the GitHub App provisioning claim. All 13 exact claim commands passed independently.

## Verification evidence

- Clean install: `npm ci` — 58 packages, 0 reported vulnerabilities.
- Frontend: `npx tsc --noEmit`, `npm run build`, and `npm test` — pass; 17/17 Playwright tests.
- Backend: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` — pass; 14/14 tests. `cargo build --release` passes.
- Runtime contract: `env -i PORT=18080 target/release/diff-gate` starts, creates its default database, and serves `{"status":"ok","build":"dev"}`.
- Build: JS 22,648 bytes (7.59 kB gzip); CSS 12,003 bytes (3.58 kB gzip). The deployed JS SHA-256 equals the local `dist` file: `21dd4f80f06d79c60e3e2e9422c560f6cceb227b1ae8ab7b88483e93f908af1f`.
- Live response policy: CSP is a response header with `frame-ancestors 'none'` and GitHub-only external form submission; HSTS, `nosniff`, strict-origin referrer policy, no-cache HTML, and immutable hashed assets are present.
- Live rate test: 100 concurrent HTTP/2 requests on one connection returned 40×401 and 60×429; every 429 included `Retry-After: 1`.
- `node scripts/live-browser-smoke.mjs` passes on 1440px desktop and 390px dark/reduced-motion mobile: one h1/main, route titles, no overflow, visible keyboard focus, zero serious/critical Axe findings, no console errors on product routes, same-origin demo requests, and a usable loaded demo offline.
- Factory `verify-url.sh` passes. Lighthouse 12.8.2: performance 100, accessibility 100, best practices 100, SEO 100, LCP 1.7 s, CLS 0, TBT 20 ms.
- Evidence is in `.factory/repair-6-artifacts/`.

## External interaction boundary

This worker had no human Sociobot Entra account session and no GitHub web session, so it could not click through consent, create the account-owned GitHub App, or mutate a private repository on a person's behalf. Those steps are intentionally impossible with the available Azure service principal and fine-grained GitHub repository token. The deployed flow now reaches the real registered Entra policy and real GitHub App manifest/install endpoints; it contains no local identity bypass or alternate provider. Final acceptance should use a real team account to complete sign-in → create/install App → save policy → import private PR → save evidence → owner approval → retention/delete.

## Commands

```sh
npm ci
npx tsc --noEmit
npm test
npm run build
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
node scripts/live-browser-smoke.mjs
npm run test:live-identity
```
