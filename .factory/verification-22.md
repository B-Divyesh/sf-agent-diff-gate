# Diff Gate independent verification 22 — FAIL

- **Candidate:** `52b389fd8f0b4886021b8fa46dc196dfc3addaf0`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC
- **Verdict:** **FAIL — do not release.**

## First-read result

A cold desktop load plainly says “Review agent-authored changes before merge,”
names small software teams as the audience, and presents **Try it with sample
data** with an immediate explanation of the sample packet. The action is also
inside the initial 390×844 viewport. It opens a usable, isolated review packet
with changed files, test evidence, risky paths, reset, export, and Start for
real controls. This first-read/demo gate **passes**.

## Release blocker — real workflow is unavailable in production

The public deployment is serving the candidate build, but it is not deployed
with the stateful SQLite contract required by the product. The application
correctly fails closed, which means the actual job-to-be-done (team sign-in,
team-scoped packets, policy, evidence, approvals, audit, retention, and GitHub
setup/import) cannot be used.

Fresh public evidence:

- `GET /health` returned **503** with
  `{"status":"unsafe_configuration","build":"52b389fd8f0b4886021b8fa46dc196dfc3addaf0",...}`.
- `GET /api/auth/status` returned 200 but `service_ready:false`, no Entra
  configuration, and no GitHub App setup. `GET /auth/entra` and
  `GET /api/packets` both returned **503** with “waiting for its durable
  production storage configuration.” An actual Sociobot Entra login therefore
  could not be completed or independently verified.
- A fresh 100-request concurrent `/health` probe returned **100×503**,
  `unsafe_configuration`, the expected candidate build, and **two distinct
  storage IDs**. That is direct evidence of more than one live process/store.
- Read-only Azure control-plane evidence for revision
  `sf-agent-diff-gate--0000084` confirms image
  `sociobotregistry.azurecr.io/sf-agent-diff-gate:52b389fd8f0b`, only
  `PORT=8080`, `minReplicas:1`, `maxReplicas:3`, `volumes:null`, and
  `volumeMounts:null`.

This violates the backend contract (exactly one replica, Azure Files `/data`,
production `DATABASE_URL`, and complete Entra configuration). It is a
**critical release blocker**, even though the demo correctly remains usable.

## Claims gate (run first from the clean checkout)

`npm ci` completed with 58 packages and zero audit vulnerabilities. Every
command listed in `.factory/claims.json` was run against the shipped local demo
entry point or its specified backend fixture; all passed.

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

## Local quality gates

- `npm test`: **PASS** — 11 Node tests and 25 Playwright tests.
- `npx tsc --noEmit`, `cargo fmt --check`, and
  `cargo clippy --all-targets --all-features -- -D warnings`: **PASS**.
- `cargo build --release`: **PASS**.
- `./scripts/verify-runtime-contract.sh`: **PASS**; clean `PORT` startup
  returned the supplied build identity and a durable-store identity.
- `npm run build`: **PASS**; `dist/` produced. Initial JS is 22,863 B
  (7.28 kB gzip), CSS 12,233 B (3.62 kB gzip), and hero WebP 136,640 B — all
  within the applicable budgets.
- `docker build ...`: **not run** because this verification worker has no
  `docker`, `podman`, `buildah`, or `nerdctl` executable. This is an
  environment limitation, not a product finding.

## Browser, accessibility, privacy, and HTTP checks

- Fresh deployed desktop (1440×1000) and mobile (390×844, dark,
  reduced-motion) checks passed for `/`, `/demo`, `/privacy`, `/terms`, and
  the designed HTTP 404 route. No console or page errors; one `h1`, one
  `main`, `lang=en`, no horizontal overflow, and visible keyboard focus.
- Axe found **no serious or critical findings** on those routes. Local browser
  coverage also passed keyboard-only review, 200% text, 44px targets, reset,
  offline demo interaction, export, approval, invalid/blocked states, and
  recovery navigation.
- Request logs for cold landing, demo launch, review mutation, and export were
  same-origin only. No analytics or third-party runtime request was observed.
  The loaded demo remained usable after `context.setOffline(true)`.
- Live headers include `X-Content-Type-Options: nosniff`, strict-origin
  referrer policy, HSTS, and a self-only CSP with `frame-ancestors 'none'`.
  Documents use `no-cache`; hashed JS is one-year immutable; the hero image is
  one-hour `must-revalidate`; unknown routes are HTTP 404 with
  `X-Diff-Gate-Route: not-found` and `X-Robots-Tag: noindex`.
- The server-side rate limiter passed from one client: **40** accepted
  `/api/auth/status` requests, then **60** responses with HTTP **429** and
  `Retry-After: 1`.

Screenshots from the independent live browser sweep are retained in
`.factory/verification-22-artifacts/`.

## Required remediation and re-verification

Deploy this same image only through the stateful production contract: one
replica, Azure Files volume `agent-diff-gate-data-v4` mounted at `/data`, the
production SQLite URL, public base URL, and Sociobot Entra environment values.
Then re-run the live deployment contract, including health concurrency,
durable replacement, real Entra redirect, and signed-in team workflow checks.
