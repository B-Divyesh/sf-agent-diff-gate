# Independent verification 11 — FAIL

**Candidate:** `076df6c3aaf53de4b8aae83f07de857c29bfa001`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC from a clean checkout

## Release decision

**FAIL.** The live image is the requested candidate and its UI, demo, tests, accessibility, privacy behavior, identity redirect, rate limit, and performance pass. Production is nevertheless unsafe for the real job: the Container App has no durable `/data` volume and may scale to three replicas even though each process uses its own SQLite database. Sessions, packets, evidence, approvals, GitHub App credentials, and audit history can disappear on replacement or disagree between replicas. The repository's own live deployment verifier exits 1 against the current deployment.

A second release-blocking routing defect remains: unknown URLs serve the recovery UI with HTTP 200 instead of a real 404.

## Mandatory first checks

### Cold first read

PASS. A fresh 1440 x 900 browser context said **“Review agent-authored changes before merge.”** It identifies **small software teams** that need a required owner and test evidence. The first action is **“Try it with sample data”**, next to an explanation that it opens changed files, evidence, and owner checks. The action was fully visible without scrolling. At 390 x 844 its bottom edge was 588.52 px. One keyboard activation opened the complete sample.

Evidence: [`live-first-read/cold-desktop.json`](verification-artifacts-11/live-first-read/cold-desktop.json) and [`live-first-read/cold-desktop.png`](verification-artifacts-11/live-first-read/cold-desktop.png).

### Claims gate

`.factory/claims.json` exists with 20 entries. Per the instruction to run them before anything else, the exact commands were first invoked before dependency installation. All 13 Rust/runtime commands passed; the seven Playwright commands could not start because a clean clone did not yet contain `@playwright/test`. After the required `npm ci`, every exact claim command passed. The installed clean-clone claim gate therefore passes; the initial startup transcript is retained for transparency.

| Claim | Declared test | Installed clean-clone result |
| --- | --- | --- |
| `sample-sandbox` | `npm run test:browser -- --grep @claim:sample-sandbox` | PASS, 1/1 |
| `packet-export` | `npm run test:browser -- --grep @claim:packet-export` | PASS, 1/1 |
| `demo-query-path` | `npm run test:browser -- --grep @claim:demo-query-path` | PASS, 1/1 |
| `mobile-first-action` | `npm run test:browser -- --grep @claim:mobile-first-action` | PASS, 1/1 |
| `no-merge-action` | `npm run test:browser -- --grep @claim:no-merge-action` | PASS, 1/1 |
| `team-packet-boundary` | `cargo test packet_reads_and_approvals_are_scoped_to_the_signed_in_team` | PASS, 1/1 |
| `named-approval` | `cargo test approval_rejects_missing_evidence_and_wrong_owner_and_persists_saved_evidence` | PASS, 1/1 |
| `entra-team-installation` | `cargo test entra_and_github_installations_are_configured_per_team` | PASS, 1/1 |
| `github-complete-import` | `cargo test github_import_paginates_and_classifies_all_changed_paths` | PASS, 1/1 |
| `github-revision-refresh` | `cargo test github_revision_change_refreshes_packet_and_blocks_approval` | PASS, 1/1 |
| `github-app-provisioning` | `cargo test github_app_manifest_is_read_only_and_bound_to_the_signed_in_team` | PASS, 1/1 |
| `repository-policy` | `cargo test repository_policy_is_team_scoped_and_requires_its_own_paths_and_owner` | PASS, 1/1 |
| `retention-deletion` | `cargo test retention_and_explicit_deletion_remove_packets_and_audit` | PASS, 1/1 |
| `audit-history` | `cargo test audit_history_is_team_scoped_and_concurrent_approval_reports_conflict` | PASS, 1/1 |
| `audit-export` | `npm run test:browser -- --grep @claim:audit-export` | PASS, 1/1 |
| `no-third-party-runtime` | `npm run test:browser -- --grep @claim:no-third-party-runtime` | PASS, 1/1 |
| `github-file-limit` | `cargo test github_import_rejects_more_than_10000_files` | PASS, 1/1 |
| `retention-limits-and-cleanup` | `cargo test retention_limits_default_and_read_cleanup_are_enforced` | PASS, 1/1 |
| `runtime-port-health` | `./scripts/verify-runtime-contract.sh` | PASS |
| `durable-store-replacement` | `cargo test durable_storage_identity_survives_database_reopen` | PASS, 1/1 |

Evidence: [`claims-installed/`](verification-artifacts-11/claims-installed/) and the pre-install [`claims/`](verification-artifacts-11/claims/) transcripts.

The live landing page, legal pages, real-work controls, and README were cross-checked against the registry. No unlisted reliance-worthy claim was found.

## Clean-checkout quality and production builds

All available source gates passed:

```text
npm ci                         # 58 packages, 0 vulnerabilities
npm test                       # 24 Playwright tests passed; Vitest had no unit files
npm run build                  # Vite production build passed and produced dist/
npx tsc --noEmit               # passed
cargo fmt --check              # passed
cargo test                     # 20 passed
cargo clippy -- -D warnings    # passed
./scripts/verify-runtime-contract.sh  # release server started with PORT-only contract
```

The production frontend contains 22,499 B JavaScript (7.19 kB gzip) and 12,233 B CSS (3.62 kB gzip). The 136,640 B hero is below its 300 kB budget. `docker build --build-arg BUILD_SHA=...` was attempted, but no Docker, Podman, Buildah, or nerdctl executable exists in this verifier container. The Dockerfile's constituent Vite and Rust release builds both succeeded, and the release binary passed the runtime contract.

Evidence: [`gates/`](verification-artifacts-11/gates/) and [`asset-sizes.txt`](verification-artifacts-11/asset-sizes.txt).

## End-to-end product exercise

The smallest useful public flow passes in a fresh live browser:

1. Keyboard Enter opened the sample packet.
2. The packet showed three changed files, four evidence checks, and required owner Mira Chen.
3. Keyboard Enter resolved both required-owner checks.
4. Approval became available and recorded an immutable approval.
5. Reload retained the approval.
6. Export produced valid JSON with the title, all three files, four checks, and approved status.
7. Reset restored both unresolved checks.
8. Start for real returned home and removed `sessionStorage['demo:diff-gate']`.

The request log for this entire flow contained only `https://agent-diff-gate.sociobot.in`; there were no analytics, third-party scripts, or sample-data requests. Anonymous valid requests to create, update, approve, delete, refresh, import, or read audit data all returned 401. A malformed Entra callback returned a clear 400 recovery message. Fixture-backed server tests cover wrong-owner and missing-evidence rejection, cross-team reads, GitHub pagination and the 10,000-file boundary, revision changes, retention limits, deletion, durable reopen, and the concurrent approval conflict.

Evidence: [`live-independent-e2e.json`](verification-artifacts-11/live-independent-e2e.json), [`live-valid-unauth-api.txt`](verification-artifacts-11/live-valid-unauth-api.txt), and [`live-invalid-callback.txt`](verification-artifacts-11/live-invalid-callback.txt).

## Live deployment identity, headers, and allowance

- `/health` returned `200` with build `076df6c3aaf53de4b8aae83f07de857c29bfa001` and a current-process storage id.
- Fresh local/live SHA-256 hashes match exactly for JS, CSS, and the hero image.
- `/auth/entra` redirects only to `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650` with the production callback and PKCE S256.
- HTML is `no-cache`; hashed JS/CSS are `public, max-age=31536000, immutable`; the stable WebP is revalidated hourly.
- Responses include HSTS, `nosniff`, strict-origin referrer policy, and a CSP with `frame-ancestors 'none'`.
- One HTTP/2 client sent 80 simultaneous harmless `/api/auth/status` requests with one forwarded client IP: **40 returned 200 and 40 returned 429**. The limited responses contained **`Retry-After: 1`**. Observed allowance: **40 requests per client per second**.

Evidence: [`live-health.json`](verification-artifacts-11/live-health.json), [`asset-hashes.txt`](verification-artifacts-11/asset-hashes.txt), [`live-identity.log`](verification-artifacts-11/live-identity.log), and [`live-rate-limit-http2.json`](verification-artifacts-11/live-rate-limit-http2.json).

## Accessibility, mobile, and performance

- `/`, `/demo`, `/privacy`, `/terms`, and the unknown-route recovery view each had `lang=en`, one `main`, one h1, route-specific title/canonical metadata, and zero serious/critical Axe findings.
- No page or console errors occurred in the complete desktop/mobile flow.
- At 390 px there was no horizontal overflow and no interactive target below 44 x 44 px.
- The first action's keyboard focus ring was `3px solid rgb(201, 76, 59)`; its contrast is 4.07:1 on the light background and 3.86:1 on dark.
- With reduced motion requested, no active animation or transition remained.
- A 200% text-size check at 390 px retained all header links without clipping or horizontal overflow.
- Fresh mobile Lighthouse: **100 performance, 100 accessibility, 100 best practices, 100 SEO**; FCP 0.96 s, LCP 1.71 s, TBT 6 ms, CLS 0, total transfer 174,129 B.

Evidence: [`live-demo-mobile-390.png`](verification-artifacts-11/live-demo-mobile-390.png), [`live-mobile-200pct.json`](verification-artifacts-11/live-mobile-200pct.json), and [`lighthouse-summary.json`](verification-artifacts-11/lighthouse-summary.json).

## Release-blocking defects

### Critical — production state is ephemeral and inconsistent when scaled

Fresh Azure control-plane output for `sf-agent-diff-gate` shows:

```json
{
  "scale": { "minReplicas": 1, "maxReplicas": 3 },
  "volumes": null,
  "containers": [{ "name": "app", "env": [{ "name": "PORT", "value": "8080" }], "volumeMounts": null }]
}
```

The process defaults to `sqlite:/data/diff-gate.db?mode=rwc`. With no mounted volume, `/data` is container-local. A revision replacement loses the database; scale-out gives each replica a different database. This affects authentication sessions, team boundaries, repository policies, packets, evidence, approvals, GitHub App credentials, and audit history. A request routed to another replica may appear signed out or see missing state. The promised retained, accountable review record therefore does not work reliably in production.

The repository already contains the correct deployment contract: one replica plus an Azure Files mount at `/data`. Running `./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in` against current production exits 1. This is fresh evidence of the deployment-only failure, not an inference from a prior report.

Evidence: [`live-containerapp-config.json`](verification-artifacts-11/live-containerapp-config.json) and [`live-deployment-contract-result.txt`](verification-artifacts-11/live-deployment-contract-result.txt).

**Required fix:** apply the repository's production deployment configuration so `/data` is durable and shared, keep SQLite at exactly one replica, then run the replacement check in `verify-live-deployment.sh ... --replace` and confirm the storage id survives.

### High — unknown routes return HTTP 200 instead of a real 404

`GET /not-a-real-route` returns the styled “Page not found” recovery UI with `X-Diff-Gate-Route: not-found` and `X-Robots-Tag: noindex`, but its HTTP status is **200**. `/404` also returns 200. This violates the acceptance contract's real-404 requirement and tells crawlers, caches, monitors, and API clients that nonexistent resources are valid.

The static deployment config specifies a 404 rewrite, but the Axum fallback deliberately changes the status to 200. The recovery page itself is usable and accessible; only its protocol status is wrong.

**Required fix:** return the styled recovery document with HTTP 404 and retain the noindex header and working route back home.

Evidence: [`live-unknown-route-headers.txt`](verification-artifacts-11/live-unknown-route-headers.txt).

## Scope limitations

No test member account or private GitHub organization was supplied, so this verification did not submit a real Entra login or install/import from a private GitHub repository. The live tenant/PKCE redirect and unauthenticated boundaries were checked, while the authenticated team, GitHub, approval, audit, retention, persistence, and concurrency behavior was exercised by the passing isolated tests.

This is a web-with-backend product, not a library or CLI. It does not claim PWA offline reload, and no service worker is registered; the loaded demo's offline interaction test passes.
