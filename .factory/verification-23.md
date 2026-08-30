# Diff Gate independent verification 23 — FAIL

- **Candidate:** `3869a47e182c9a2040d62280ee2e0cdc9260324f`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC
- **Verdict:** **FAIL — do not release.**

## Release-blocking defect

### Critical — the real team workflow is unavailable in production

The public site serves this exact candidate, but the backend is running without
the stateful deployment and identity configuration required by the product.
The application correctly fails closed. A team cannot sign in, connect its
GitHub App, import or create a real review packet, save evidence, approve,
inspect audit history, configure retention, or delete its data.

Fresh public evidence:

- `GET /health` returned HTTP **503** with
  `status:"unsafe_configuration"`, build
  `3869a47e182c9a2040d62280ee2e0cdc9260324f`, and storage ID
  `bf5a48a2-41c2-4187-b478-375b02d83200`.
- `GET /api/auth/status` returned HTTP 200 but
  `service_ready:false`, `entra_sign_in_configured:false`, and
  `github_app_setup_available:false`.
- `GET /api/packets` and `GET /auth/entra` each returned HTTP **503**:
  “Diff Gate is waiting for its durable production storage configuration.”
- Azure Container Apps revision `sf-agent-diff-gate--0000089` uses image
  `sociobotregistry.azurecr.io/sf-agent-diff-gate:3869a47e182c`, has only
  `PORT=8080`, permits **1–3 replicas**, and has no volume or volume mount.
- `./scripts/verify-live-deployment.sh` failed immediately: the one-replica,
  Azure Files `/data`, durable `DATABASE_URL`, public base URL, Sociobot Entra,
  and deployment-contract-version assertions are all unmet.

This is fresh deployment drift, not a stale builder report. The current build
identity and the candidate's HTML, JS, CSS, and product-image SHA-256 hashes
match exactly. A 100-request concurrent health probe returned 100 HTTP 503
unsafe responses for the candidate. It reached one current replica and one
ephemeral storage identity, but the live template still permits scale-out to
three and has no durable storage. Live replacement persistence cannot be
accepted while that contract is absent.

## First-read and demo gate

**PASS.** A cold desktop and 390×844 mobile load answer all three mandatory
questions in the first screen:

- What: “Review agent-authored changes before merge.”
- For whom: small software teams needing a required owner and test evidence.
- First action: **Try it with sample data**, followed by what the sample opens.

At 390×844 the full action was visible at `x=20`, `y=542`, size
`207.6×46.3`. One click opened a populated packet. The persistent banner said
“Demo — sample data, nothing is saved” and exposed Reset demo and Start for
real.

The live sample flow passed independently: approval began disabled with clear
recovery text, keyboard Enter resolved both required checks, approval became
enabled, the approved state rendered, Reset demo restored two checks, and Start for
real removed the sole `demo:diff-gate` session-storage key. The visible focus
outline was 3px solid coral. The demo remains usable offline after loading.

## Claims gate

After the clean-clone prerequisite `npm ci` (58 packages, zero reported audit
vulnerabilities), every exact command in `.factory/claims.json` passed.

| Claim | Exact test | Result |
| --- | --- | --- |
| `sample-sandbox` | `npm run test:browser -- --grep @claim:sample-sandbox` | PASS |
| `packet-export` | `npm run test:browser -- --grep @claim:packet-export` | PASS |
| `demo-query-path` | `npm run test:browser -- --grep @claim:demo-query-path` | PASS |
| `mobile-first-action` | `npm run test:browser -- --grep @claim:mobile-first-action` | PASS |
| `no-merge-action` | `npm run test:browser -- --grep @claim:no-merge-action` | PASS |
| `team-packet-boundary` | `cargo test packet_reads_and_approvals_are_scoped_to_the_signed_in_team` | PASS |
| `named-approval` | `cargo test approval_rejects_missing_evidence_and_wrong_owner_and_persists_saved_evidence` | PASS |
| `entra-team-installation` | `cargo test entra_and_github_installations_are_configured_per_team` | PASS |
| `github-complete-import` | `cargo test github_import_paginates_and_classifies_all_changed_paths` | PASS |
| `github-revision-refresh` | `cargo test github_revision_change_refreshes_packet_and_blocks_approval` | PASS |
| `github-app-provisioning` | `cargo test github_app_manifest_is_read_only_and_bound_to_the_signed_in_team` | PASS |
| `repository-policy` | `cargo test repository_policy_is_team_scoped_and_requires_its_own_paths_and_owner` | PASS |
| `retention-deletion` | `cargo test retention_and_explicit_deletion_remove_packets_and_audit` | PASS |
| `audit-history` | `cargo test audit_history_is_team_scoped_and_concurrent_approval_reports_conflict` | PASS |
| `audit-export` | `npm run test:browser -- --grep @claim:audit-export` | PASS |
| `no-third-party-runtime` | `npm run test:browser -- --grep @claim:no-third-party-runtime` | PASS |
| `github-file-limit` | `cargo test github_import_rejects_more_than_10000_files` | PASS |
| `retention-limits-and-cleanup` | `cargo test retention_limits_default_and_read_cleanup_are_enforced` | PASS |
| `runtime-port-health` | `./scripts/verify-runtime-contract.sh` | PASS |
| `durable-store-replacement` | `cargo test durable_storage_identity_survives_database_reopen` | PASS |

Landing, README, privacy, terms, and demo claim-like copy was cross-checked
against the manifest. No unlisted release claim was found.

## Clean local verification

- `npm test`: **PASS** — 12 Node tests and 25 Playwright tests.
- `npx tsc --noEmit`: **PASS**.
- `npm run build`: **PASS** and produced `dist/`.
- `cargo fmt --check`: **PASS**.
- `cargo test`: **PASS** — 21 tests.
- `cargo clippy --all-targets --all-features -- -D warnings`: **PASS**.
- `cargo build --release`: **PASS**.
- `./scripts/verify-runtime-contract.sh`: **PASS** with PORT-only startup,
  build identity, and durable-store identity.
- A container build was not run because this worker has no Docker, Podman,
  Buildah, or nerdctl executable. The Dockerfile uses unpinned `rust:1-alpine`,
  a multi-stage build, a non-root runtime user, `ARG BUILD_SHA=dev`, and does
  not read `.git`.

The production bundle is within budget: JS 22,863 B (7.28 kB gzip), CSS
12,233 B (3.62 kB gzip), and the hero WebP 136,640 B.

## Live browser, accessibility, privacy, and HTTP evidence

- Desktop 1440×1000 and mobile 390×844 passed for `/`, `/demo`, `/privacy`,
  `/terms`, and the designed HTTP 404 route. Route titles, one `h1`, one
  `main`, `lang=en`, alt text, 200% mobile text reflow, and link crawling
  passed. No page or console errors were observed.
- Axe reported zero serious or critical findings in light and dark treatments
  across all public routes and the 404. Lighthouse accessibility was 100.
- Keyboard operation passed the full demo review. Focus remained visible and
  there was no trap. Under `prefers-reduced-motion: reduce`, no element had a
  running animation; normal mode used only the documented 0.2s/0.4s packet
  motions.
- The independent live demo request log contained six same-origin requests:
  document, JS, CSS, two `/api/auth/status` fetches, and the hero image. There
  were no off-origin requests, analytics, failed resources, or console errors.
- Response security passed: HSTS, `nosniff`, strict-origin referrer policy,
  and a header-delivered CSP with `frame-ancestors 'none'`.
- Caching passed: documents `no-cache`; hashed JS/CSS one-year immutable;
  the product WebP one-hour `must-revalidate`; unknown paths return HTTP 404
  with `X-Diff-Gate-Route: not-found` and `X-Robots-Tag: noindex`.
- Factory `verify-url.sh` passed in 602 ms with a title, `lang`, one `h1`, a
  main landmark, complete image alt text, labeled buttons, and no console
  errors.
- Isolated Lighthouse mobile: **100/100/100/100** Performance/
  Accessibility/Best Practices/SEO; FCP 1.0 s, LCP 1.8 s, TBT 40 ms, CLS 0.
  Desktop: **100/100/100/100**; FCP 0.3 s, LCP 0.5 s, TBT 30 ms, CLS 0.

## Backend boundaries

- The live rate limiter allowed exactly **40** requests from one client in its
  one-second window, then returned **60×429**; every rejection had
  `Retry-After: 1`. Health is intentionally exempt.
- Local integration tests passed team isolation, required-owner enforcement,
  forged evidence rejection, duplicate concurrent approval conflict, GitHub
  pagination and 10,000-file boundary, revision invalidation, repository
  policy isolation, retention limits `1..=3650`, expiry cleanup, deletion, and
  database reopen persistence.
- Source and integration tests restrict identity to
  `sociobotcustomers.ciamlogin.com` with PKCE. The live redirect cannot be
  verified because the current deployment has no Entra configuration; this is
  part of the critical defect.
- Library/CLI consumer checks and PWA service-worker checks are not applicable
  to this web-with-backend artifact.

## Required remediation

Deploy candidate `3869a47e182c9a2040d62280ee2e0cdc9260324f` only through
`scripts/deploy-production.sh`: one replica, Azure Files
`agent-diff-gate-data-v4` mounted at `/data`, the durable SQLite URL, public
base URL, deployment contract version, and Sociobot Entra settings. Then run
the live contract without bypassing it, confirm the Entra redirect, send 100
concurrent health requests, replace the revision, and prove the same storage
identity and 40-request allowance before reconsidering release.

Evidence is in `.factory/verification-23-artifacts/`.
