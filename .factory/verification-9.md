# Independent verification 9 — FAIL

**Candidate:** `22eb3d32439685f5e2911553e3cb47fdf995ee6d`  
**Live URL:** <https://agent-diff-gate.sociobot.in>  
**Verified:** 2026-08-29 UTC

## Release decision

**FAIL.** The deployment is the requested candidate, and the functional, privacy, accessibility, performance, and rate-limit checks passed. It nevertheless has one release-blocking standards defect: an unknown URL renders the custom 404 screen with **HTTP 200**, not HTTP 404. This violates the required real-404-route contract and causes crawlers and clients to treat nonexistent pages as valid content.

## Cold first read

Fresh desktop and 390×844 mobile loads plainly say **“Review agent-authored changes before merge.”** They say it is for **small software teams** needing a required owner and test evidence, and put **“Try it with sample data”** in the first screen with **“Opens a complete review packet.”** The action opened the complete, isolated sample packet in one click. This first-read and demo gate passes.

## Required claims gate

`.factory/claims.json` exists. After `npm ci`, every exact declared command passed from this checkout:

| Claim | Command | Result |
| --- | --- | --- |
| sample-sandbox | `npm run test:browser -- --grep @claim:sample-sandbox` | PASS |
| packet-export | `npm run test:browser -- --grep @claim:packet-export` | PASS |
| demo-query-path | `npm run test:browser -- --grep @claim:demo-query-path` | PASS |
| mobile-first-action | `npm run test:browser -- --grep @claim:mobile-first-action` | PASS |
| no-merge-action | `npm run test:browser -- --grep @claim:no-merge-action` | PASS |
| team-packet-boundary | `cargo test packet_reads_and_approvals_are_scoped_to_the_signed_in_team` | PASS |
| named-approval | `cargo test approval_rejects_missing_evidence_and_wrong_owner_and_persists_saved_evidence` | PASS |
| entra-team-installation | `cargo test entra_and_github_installations_are_configured_per_team` | PASS |
| github-complete-import | `cargo test github_import_paginates_and_classifies_all_changed_paths` | PASS |
| github-app-provisioning | `cargo test github_app_manifest_is_read_only_and_bound_to_the_signed_in_team` | PASS |
| repository-policy | `cargo test repository_policy_is_team_scoped_and_requires_its_own_paths_and_owner` | PASS |
| retention-deletion | `cargo test retention_and_explicit_deletion_remove_packets_and_audit` | PASS |
| audit-history | `cargo test audit_history_is_team_scoped_and_concurrent_approval_reports_conflict` | PASS |
| audit-export | `npm run test:browser -- --grep @claim:audit-export` | PASS |
| no-third-party-runtime | `npm run test:browser -- --grep @claim:no-third-party-runtime` | PASS |
| github-file-limit | `cargo test github_import_rejects_more_than_10000_files` | PASS |
| retention-limits-and-cleanup | `cargo test retention_limits_default_and_read_cleanup_are_enforced` | PASS |
| runtime-port-health | `./scripts/verify-runtime-contract.sh` | PASS — PORT-only release startup returned build and durable-store identities |
| durable-store-replacement | `cargo test durable_storage_identity_survives_database_reopen` | PASS |

## Clean-checkout quality suite

All passed:

```text
npm ci
npx tsc --noEmit
npm test                         # 21 Playwright tests passed
cargo fmt --check
cargo test                       # 18 Rust tests passed
cargo clippy -- -D warnings
npm run build                    # 21.41 kB JS (7,018 B gzip), 12.23 kB CSS (3,623 B gzip)
./scripts/verify-runtime-contract.sh
```

The hero image is 136,640 B, below the 300 kB mobile budget. `docker` is not installed in this verifier container, so `docker build` could not be executed; the independently built Vite and Rust release artifacts passed. The deployment-management script could not be run because this container lacks the `az` CLI; this did not affect live black-box checks.

## Live deployment and product exercise

- `GET /health` returned `200` with `build: 22eb3d32439685f5e2911553e3cb47fdf995ee6d` and durable store id `0079cf67-0837-4e83-a152-141c66421d8c`.
- The live `index-C1WdzivD.js` SHA-256 was `37ea2c13ec18d541056cb6073ffd8e020275d07ad8401cd702edc35f1bdcd07d`, exactly matching the fresh candidate build.
- In a fresh live demo, keyboard Enter and click resolved both required checks, enabled approval, produced a valid `diff-gate-packet.json`, retained the approved state after reload, and **Start for real** cleared `sessionStorage['demo:diff-gate']`.
- At 390×844, the primary action ended at y=588.52, had no horizontal overflow, and the visible keyboard focus ring was `3px solid rgb(201, 76, 59)`. Reduced-motion mode was exercised.
- Live light and dark checks of `/`, `/demo`, `/privacy`, and `/terms` found exactly one h1, no console/page errors, and no Axe serious or critical findings.
- Lighthouse mobile: Performance **100**, Accessibility **100**, LCP **1.65 s**, CLS **0**.
- Authenticated workflows, concurrency conflict, team isolation, retention cleanup, and durable-store reopen are covered by the passing Rust tests. Anonymous live packet reads and valid mutation attempts returned 401; the live Entra entry point redirected only to `sociobotcustomers.ciamlogin.com/...` with PKCE S256 and the production callback.

## Privacy, headers, caching, and request allowance

- Playwright recorded only the product origin during the complete live demo flow (landing, sample, evidence changes, export, approval, reload, and exit). There were no third-party scripts, analytics requests, or sample-data egress.
- Responses include CSP with `frame-ancestors 'none'`, HSTS, `X-Content-Type-Options: nosniff`, and strict-origin referrer policy. Hashed JS/CSS are `public, max-age=31536000, immutable`; the hero is a revalidating one-hour asset; HTML is `no-cache`.
- A single client sent 55 concurrent harmless `GET /api/auth/status` requests with the same forwarded IP: **40 returned 200 and 15 returned 429**, each 429 carrying `Retry-After: 1`. Observed documented allowance: **40 requests per client per second**.

## Defects

### High — release blocking: unknown routes return 200 instead of 404

**Evidence:** `curl -D - https://agent-diff-gate.sociobot.in/does-not-exist` returned `HTTP/2 200` while serving the custom “This review desk is empty” page. The server implementation deliberately sets `StatusCode::OK` in `not_found_page`.

**Impact:** Crawlers, caches, monitoring, and API consumers cannot distinguish a nonexistent URL from a real page. This is not a real 404 route and violates the site-routing acceptance contract.

**Required repair:** Serve the styled fallback with `404 Not Found` (and retain a usable page/back link); update the console smoke check to regard an expected navigation 404 appropriately rather than changing its HTTP status.

No other defects were found. A real Entra login and a private GitHub App installation could not be submitted without a test team account, but their live redirect/boundaries and all local fixture-backed flows were verified.
