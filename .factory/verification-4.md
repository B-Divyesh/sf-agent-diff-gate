# Independent verification 4 — FAIL

**Candidate:** `586c24f96572fde8b8eef6701fdebb6210670f63`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC from a clean candidate checkout

## Release decision

**FAIL.** The exact candidate is deployed. Every registered claim test and repository quality gate passes, and the sample is clear, private, accessible, responsive, and fast. The release still does not complete the researched job. Production reports both Sociobot Entra and the team-bound GitHub App as unconfigured, leaving no way to sign in or review a real pull request. Independently, the approval backend treats a client-controlled `state: "done"` flag as sufficient evidence, so a packet can be approved while its stored test-evidence text still says to attach a command and result.

This confirms the builder's deployment-only warning with fresh live evidence, but also identifies a separate product-integrity blocker that remains after deployment configuration is supplied.

## Mandatory first checks

### Claims gate

`.factory/claims.json` exists and contains ten claims. Its commands were the first test activity. The first browser invocation could not start before dependency installation because the clean clone did not yet contain `@playwright/test`; after the required `npm ci`, all ten exact commands ran and passed. The initial missing-dependency preflight was not a product assertion failure.

| Claim | Exact command | Result |
|---|---|---|
| `sample-sandbox` | `npm run test:browser -- --grep @claim:sample-sandbox` | PASS, 1/1 |
| `packet-export` | `npm run test:browser -- --grep @claim:packet-export` | PASS, 1/1 |
| `team-packet-boundary` | `cargo test packet_reads_and_approvals_are_scoped_to_the_signed_in_team` | PASS, 1/1 |
| `named-approval` | `cargo test approval_rejects_missing_evidence_and_wrong_owner_and_persists_saved_evidence` | PASS, 1/1 |
| `entra-team-installation` | `cargo test entra_and_github_installations_are_configured_per_team` | PASS, 1/1 |
| `github-complete-import` | `cargo test github_import_paginates_and_classifies_all_changed_paths` | PASS, 1/1 |
| `retention-deletion` | `cargo test retention_and_explicit_deletion_remove_packets_and_audit` | PASS, 1/1 |
| `audit-history` | `cargo test audit_history_is_team_scoped_and_concurrent_approval_reports_conflict` | PASS, 1/1 |
| `audit-export` | `npm run test:browser -- --grep @claim:audit-export` | PASS, 1/1 |
| `no-third-party-runtime` | `npm run test:browser -- --grep @claim:no-third-party-runtime` | PASS, 1/1 |

### Cold first read

**PASS.** At 1440×900 the first screen says **“Review agent changes before merge,”** names **small software teams** that need an owner and evidence, and presents **“Try it with sample data”** beside **“Opens a complete review packet.”** One click opens `/demo` with realistic changed files, evidence, risky paths, and the persistent **“Demo — sample data, nothing is saved”** banner. Evidence: `verification-artifacts-4/live-home-desktop.png` and `live-demo-mobile.png`.

## Release-blocking findings

### Critical — nobody can perform the real workflow on the live deployment

Fresh production responses:

- `GET /health` → `200` with build `586c24f96572fde8b8eef6701fdebb6210670f63`.
- `GET /api/auth/status` → `200` with `entra_sign_in_configured:false`, `github_app_configured:false`, and no installation URL.
- `GET /auth/entra` → `503 {"error":"Sociobot Entra sign-in is not configured on this deployment."}`.

The live real-work panel repeats that warning and renders no sign-in action. A team therefore cannot authenticate, import a GitHub pull request, create or reopen a real packet, save evidence, or record a real approval. The one-click sample cannot substitute for Definition of Done #1 or the brief's smallest useful product.

The repository does restrict configured identity authorities to HTTPS on `sociobotcustomers.ciamlogin.com`, and the corresponding claim test passes. That does not make the production path usable without the Entra application and team-bound GitHub App settings.

### Critical — approval accepts a flag instead of test evidence

An imported packet starts with this stored check:

```json
{"label":"Test evidence","detail":"Attach the test command and result before owner approval.","state":"missing"}
```

The only real-packet UI control is **Mark reviewed**. It changes `state` to `done` without asking for or recording a command, result, check run, or attachment. Backend `evidence_is_complete` validates only that every state is `ready` or `done`.

Fresh reproduction against the release binary with an authenticated seeded team:

1. Created a packet whose test-evidence detail remained **“Attach the test command and result before owner approval.”** but whose client-supplied state was `done` → `201`.
2. Approved it as the named owner without supplying any test result → `200` and durable status `approved`.

The stored approved packet still contained the placeholder instruction. This defeats the core promise that test evidence exists before merge. The passing `named-approval` claim test checks state transitions, not the presence of evidence, so the public evidence claim is under-tested as well as false at this boundary.

### High — policy is fixed, not repository-specific

The GitHub import uses two hard-coded filename checks: contract risk when a path contains `api/` or `contract` or ends in `.graphql`, and migration risk when a path contains `migration` or starts with `db/`. There is no repository/team policy model or settings UI for risky paths, required owners, or checks. This misses the brief's product distinction: repo-specific policy and risky-file escalation as the default review unit. A repository using `schema/`, `infra/`, `auth/`, or another sensitive convention cannot configure it.

## Other findings

### Medium — the researched paid offering is absent

The brief calls for `$12/developer/month` or `$99/team/month` through Sociobot billing. The candidate has no pricing, paid tier, checkout, or entitlement flow. A fresh request to `https://api.sociobot.in/api/v1/products/agent-diff-gate/checkout` returned `404 {"error":"enabled factory product","status":404}`, so this also needs factory-side product registration before honest UI can be added. The first screen consequently does not provide the required price fact.

### Low — the focus-only skip link is 42px high

At 390px in both color schemes, every other visible interactive target measured at least 44×44 CSS pixels. The focused **Skip to content** link measured 124×42, two pixels below the accessibility contract. It remains keyboard operable with a visible 3px coral focus outline.

## End-to-end and boundary evidence

- Live demo: sample contained three changed files and four checks; resolving both risky checks enabled approval; approval survived reload in `demo:diff-gate`; reset restored two blockers; **Start for real** cleared the demo key.
- Live export: `diff-gate-packet.json` parsed successfully with the expected title, owner, three paths, and four checks.
- Local authenticated UI at 390px: invalid non-GitHub URL showed **“Use a github.com pull request URL.”**; a valid URL failed softly with the missing GitHub App message; manual packet creation, saved evidence, named approval, audit export, delete confirmation/cancel, and 30-day retention all operated without horizontal overflow.
- Local authenticated API: empty and 181-character titles returned 400; cross-team packet/audit reads returned 404; wrong-owner approval returned 403; unresolved evidence returned 400; malformed evidence returned 400; retention accepted 1 and 3,650 and rejected 0 and 3,651.
- Concurrency: two simultaneous valid approvals returned one 200 and one truthful 409; only one approval audit entry was stored.
- Persistence/deletion: a packet remained readable after process restart; explicit deletion removed both packet and audit rows.
- GitHub input recovery: malformed URL, wrong host, and nonnumeric PR returned specific 400 responses; a valid GitHub URL returned the actionable missing-App 503 in an unconfigured runtime.
- Unauthenticated valid packet reads returned 401. Malformed JSON bodies can return Axum's 422 before handler authentication, but no packet data was exposed.

## Quality, deployment, privacy, and accessibility

- `npm ci`: PASS; 58 packages, 0 reported vulnerabilities.
- `npx tsc --noEmit`: PASS.
- `npm test`: PASS; 13/13 Playwright tests. Vitest has no unit files and is intentionally invoked with `--passWithNoTests`.
- `npm run build`: PASS; exact production `dist/` produced.
- `cargo fmt --check`: PASS.
- `cargo test`: PASS; 11/11.
- `cargo clippy -- -D warnings`: PASS.
- `cargo build --release`: PASS.
- Runtime contract: `env -i PORT=18080 target/release/diff-gate` started successfully, logged generated default database configuration, and returned `{"status":"ok","build":"dev"}`.
- Deployment parity: `/health` reports the candidate SHA. Live HTML, JS, CSS, hero, social image, icons, robots, sitemap, and manifest SHA-256 hashes match the candidate `dist` files.
- Rate limit: 120 simultaneous requests from one forwarded client produced **40×200 and 80×429**. All 80 limited responses included `Retry-After: 1`. Observed allowance: **40 requests per client per one-second window**. `/health` is exempt.
- Privacy: the cold home → demo → export → resolve → approve → reload → reset → exit flow made ten requests, all to `https://agent-diff-gate.sociobot.in`. No analytics, CDN font/script, third-party request, failed request, page error, or console error occurred on supported routes.
- Headers: CSP is delivered as a response header; HSTS, `nosniff`, and strict-origin referrer policy are present. HTML/API use `no-cache`; hashed assets use one-year immutable caching; stable images use one-hour revalidation.
- Routes: `/`, `/demo`, `/privacy`, and `/terms` return 200; an unknown path returns a styled HTTP 404 with a route home; all landing links resolve.
- Accessibility: light and dark Axe scans on all four public routes at 390px found zero serious/critical issues. Each route has `lang=en`, one `<h1>`, one `<main>`, route-specific title, labels, and alt text. Keyboard review works with Enter/Space, visible focus is a 3px coral outline, reduced-motion produces no active animations, and 200% text remains readable without content loss (a 5px decorative overflow was measured).
- `/opt/fleet/lib/verify-url.sh`: PASS on `/` and `/demo`; zero console/page errors and all title/lang/main/alt checks passed. Evidence: `verification-artifacts-4/verify-home/verify.json` and `verify-demo/verify.json`.
- Performance: isolated mobile Lighthouse scored **100 performance / 100 accessibility / 100 best practices / 100 SEO**; FCP 0.9s, LCP 1.7s, TBT 40ms, CLS 0, total transfer 165 KiB. Evidence: `verification-artifacts-4/lighthouse-home.json`.
- Budgets: initial JS 18,517 bytes (6,431 gzip), CSS 11,615 bytes (3,515 gzip), hero WebP 136,640 bytes, no downloaded fonts.
- Product identity: the documented halftone change-control art direction is visible and original asset provenance is recorded. README, MIT license, privacy, terms, demo documentation, copy audit, sitemap, robots, social image, favicon, and 404 are present.

## Coverage notes

- Docker is unavailable in this verifier image, so the container was not rebuilt locally. Native frontend and optimized Rust production builds pass, the Dockerfile uses the required unpinned `rust:1-alpine`, declares `BUILD_SHA`, and runs as a non-root user; the exact candidate container is responding live.
- A real Entra redirect/token exchange and GitHub installation import cannot be exercised because the live deployment exposes neither configuration. This is the critical release finding, not a waived test.
- Library/CLI consumer checks and service-worker update/offline-reload checks do not apply. This is a web-with-backend product with no service worker and no offline-reload claim.

## Required before release

1. Provision the live Sociobot Entra External ID application and team-bound GitHub App mappings, then exercise a real PR import and named approval in production.
2. Require concrete test evidence at the backend boundary. Do not allow a client to convert placeholder text into accepted evidence by changing only a state flag.
3. Add per-team/repository policy configuration for sensitive paths and required owners, with an executable claim test.
4. Register the Sociobot billing product and implement the researched paid tier, or record an approved scope change.
5. Increase the skip-link target to at least 44px.
