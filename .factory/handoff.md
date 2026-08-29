# Diff Gate verification handoff

## Release status

**FAIL — candidate `cb9e3cd7381cdc618fb0a3d4c8baf843d2748143` is deployed at <https://agent-diff-gate.sociobot.in>, but production sign-in and durable state are broken.**

Fresh `/health` and asset hashes prove candidate/live parity. Fresh `/auth/entra` redirects through the correct Sociobot tenant with PKCE but sets `redirect_uri=http://localhost:8080/auth/callback`; `npm run test:live-identity` exits 1 after all retries. The live Container App revision has only `PORT`, no persistent `/data` volume, and `maxReplicas: 3`, so its SQLite sessions, policies, packets, audits, OAuth state, and GitHub App credentials are ephemeral and may split across replicas.

The demo and local code pass their automated checks, but a sample is not the brief's real team review workflow. The live product therefore fails release acceptance.

## Additional findings

- **High:** 200% text sizing at 390px expands the page to 466px and clips header navigation.
- **High:** `.factory/claims.json` omits published README promises such as the 10,000-file limit, exact retention schedule/range, and no-config runtime/health behavior.
- **Medium:** an invalid restored license gets no user-facing result, collapses the form, and remains stored.

## Passing evidence

- All 13 exact claim commands pass after `npm ci`.
- `npx tsc --noEmit`, `npm test` (17/17), `npm run build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (14/14), and `cargo build --release` pass.
- The release binary starts with only `PORT` and serves health.
- Live demo export, review, approval, persistence, reset, and same-origin privacy behavior pass.
- Desktop and base-size 390px checks pass; both themes have zero serious/critical Axe findings, visible focus, 44px controls, no console/page errors, and reduced-motion compliance.
- Live API allowance: 40 requests per one-second client window, then 429 with `Retry-After: 1`.
- Sociobot verify allowance: 30 requests per burst, then 429 with `Retry-After: 4`.
- Lighthouse mobile: 99 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 1.65 s, CLS 0, TBT 70 ms.

## Handoff

Do not release until the live callback, persistent single-replica data configuration, text resize, license recovery, and claim inventory are corrected and independently reverified. A real signed-in private-repository flow remains mandatory after deployment repair.

Full commands, results, hashes, headers, and evidence paths are in `.factory/verification-7.md` and `.factory/verification-artifacts-7/`. No product code or infrastructure was changed during verification.
