# Independent verification 13 — FAIL

**Candidate:** `9abea0da06876e8284b083ec45fbb03a25b6471b`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC from the clean candidate checkout

## Release decision

**FAIL.** The candidate source, public demo, claims, accessibility, privacy behavior, identity redirect, security headers, caching, and performance pass. The candidate image is live. Production is nevertheless unsafe for the core job: it is configured for up to three replicas, has no durable volume, and gives every process its own SQLite database under `/data`.

This is active. Fresh load created three running replicas, and 240 concurrent `/health` requests returned three different `storage_id` values from the same candidate build. Reviewers, required owners, and audit readers can therefore reach different packet stores. Sessions, policies, packets, evidence, approvals, GitHub App configuration, and audit history can disagree or disappear when a replica is replaced.

## Release-blocking defect

### Critical — production serves three independent ephemeral packet stores

Fresh Azure control-plane evidence for `sf-agent-diff-gate` shows:

```json
{
  "activeRevisionsMode": "Single",
  "image": "sociobotregistry.azurecr.io/sf-agent-diff-gate:9abea0da0687",
  "scale": { "minReplicas": 1, "maxReplicas": 3 },
  "volumes": null,
  "volumeMounts": null,
  "env": [{ "name": "PORT", "value": "8080" }]
}
```

After load, Azure reported three running replicas. The request-level proof was:

```json
{
  "requests": 240,
  "builds": ["9abea0da06876e8284b083ec45fbb03a25b6471b"],
  "storage_ids": [
    "3835899b-fc72-4e28-b819-62011a5a6b11",
    "8d6aafd2-84dc-4907-aaa4-54a7bde406c1",
    "db8d4958-9f9d-4734-9c97-16315850b188"
  ]
}
```

The repository's non-mutating live verifier exited 1 and reported the missing one-replica limit, Azure Files volume, `/data` mount, durable `DATABASE_URL`, production identity settings, and deployment contract version. Evidence: [`live-containerapp-config.json`](evidence/verification-13/live-containerapp-config.json), [`live-replicas.json`](evidence/verification-13/live-replicas.json), [`live-health-after-scale.json`](evidence/verification-13/live-health-after-scale.json), and [`live-deployment-verifier.log`](evidence/verification-13/live-deployment-verifier.log).

The replica-local limiter also multiplied the effective allowance. One 240-request burst from one forwarded client produced 120×200 and 120×429; every 429 had `Retry-After: 1`. **Observed live allowance: 120 requests per client per second**, rather than the source's 40 per process. Evidence: [`live-rate-limit-240.json`](evidence/verification-13/live-rate-limit-240.json).

**Required repair:** deploy through `scripts/deploy-production.sh`, which applies the candidate image and stateful template atomically. Production must have exactly one replica, the existing `agent-diff-gate-data-v4` Azure Files volume mounted at `/data`, the committed durable database URL, and the production environment contract. Then run `./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in --replace 9abea0da06876e8284b083ec45fbb03a25b6471b sociobotregistry.azurecr.io/sf-agent-diff-gate:9abea0da0687` and prove one unchanged `storage_id` before and after replacement. This verifier did not mutate deployment state.

## Mandatory first checks

### Cold first read — PASS

A fresh 1440×900 browser context immediately answered all three questions:

- What: **“Review agent-authored changes before merge.”**
- For whom: small software teams that need a required owner and test evidence.
- First action: **“Try it with sample data”**, next to a plain explanation of the changed files, evidence, and owner checks it opens.

The action was fully visible at 390×844 and opened the realistic sample in one keyboard activation. Evidence: [`live-independent-qa.json`](evidence/verification-13/live-independent-qa.json) and [`screenshot-desktop.png`](evidence/verification-13/verify-url/screenshot-desktop.png).

### Claims gate — PASS after the required clean install

`.factory/claims.json` exists with 20 entries. As requested, all seven Playwright commands were first invoked before dependency installation; each stopped at the expected clean-clone precondition because `@playwright/test` was not installed. `npm ci` installed 58 packages with zero vulnerabilities. Every exact claim command then passed from the demo entry point or isolated backend sandbox.

| Claims | Result |
| --- | --- |
| `sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action` | PASS |
| `team-packet-boundary`, `named-approval`, `entra-team-installation` | PASS |
| `github-complete-import`, `github-revision-refresh`, `github-app-provisioning` | PASS |
| `repository-policy`, `retention-deletion`, `audit-history`, `audit-export` | PASS |
| `no-third-party-runtime`, `github-file-limit`, `retention-limits-and-cleanup` | PASS |
| `runtime-port-health`, `durable-store-replacement` | PASS |

Individual transcripts are in [`claims/`](evidence/verification-13/claims/). The live landing, legal pages, README, and claim registry were cross-checked; no unlisted reliance-worthy product claim was found.

## Clean-checkout gates and production builds

All available source gates pass:

```text
npm ci                                      58 packages, 0 vulnerabilities
npm test                                    3 unit + 24 Playwright tests passed
npx tsc --noEmit                            passed
npm run build                               passed; dist/ produced
cargo fmt --all -- --check                  passed
cargo test --all                            20 passed
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release                       passed
./scripts/verify-runtime-contract.sh        passed with PORT-only startup
```

The production frontend is 22,499 B JavaScript (7,223 B gzip), 12,233 B CSS (3,623 B gzip), and a 136,640 B hero image. Local and live SHA-256 hashes match for HTML, JavaScript, CSS, and the hero. No Docker-compatible executable is installed in this verifier container, so a local wrapper image build was unavailable; the constituent Vite and Rust release builds pass, and `/health` proves the deployed container was built from the full candidate SHA. Evidence: [`gates/`](evidence/verification-13/gates/), [`asset-sizes.txt`](evidence/verification-13/asset-sizes.txt), and [`live-local-hashes.txt`](evidence/verification-13/live-local-hashes.txt).

## End-to-end product exercise

The live sample workflow passes independently:

1. Keyboard Enter opened the one-click sample.
2. The packet showed owner Mira Chen, three changed files, four evidence checks, and two unresolved owner checks.
3. Approval was disabled before evidence resolution.
4. Keyboard-only actions resolved both checks and enabled approval.
5. Approval became immutable and survived reload.
6. Export produced valid JSON with the title, three files, four checks, and approved status.
7. Reset restored the two unresolved checks.
8. Start for real removed `sessionStorage['demo:diff-gate']` and the demo banner.

Invalid/recovery paths also pass in an authenticated browser fixture using the live candidate frontend: retention `0`, a malformed pull-request URL, and an empty title are blocked without API calls; retention `30` saves; an incomplete repository policy reports a specific error and succeeds after correction. Live anonymous packet, settings, policy, import, audit, manifest, and approval requests are denied. An expired Entra callback returns 400 with “That Sociobot sign-in link expired. Start again.” Evidence: [`live-independent-qa.json`](evidence/verification-13/live-independent-qa.json) and [`live-api-boundaries.txt`](evidence/verification-13/live-api-boundaries.txt).

## Privacy, identity, headers, accessibility, and performance

- The full live demo flow made ten requests, all to `https://agent-diff-gate.sociobot.in`; it logged no console or page errors. Browser-observed HTML headers include `no-cache`, HSTS, `nosniff`, strict-origin referrer policy, and a CSP with `frame-ancestors 'none'`.
- Hashed JavaScript is cached immutable for one year; the stable hero revalidates hourly. Unknown routes return the designed page with HTTP 404 and `X-Robots-Tag: noindex`.
- `/health` returns the full candidate SHA. `/auth/entra` redirects only to `sociobotcustomers.ciamlogin.com` with the production callback and PKCE S256.
- Desktop and 390×844 mobile pass keyboard operation, visible focus, 44 px targets, no horizontal overflow, dark mode, reduced motion, loaded-demo offline use, and axe with zero serious/critical findings.
- The standard URL verifier passes title, `lang=en`, one h1, one main landmark, image alt text, button names, and console checks.
- Fresh mobile Lighthouse: **99 performance, 100 accessibility, 100 best practices, 100 SEO**; FCP 0.98 s, LCP 1.73 s, TBT 74 ms, CLS 0, transfer 174,153 B.

Evidence: [`live-header-matrix.txt`](evidence/verification-13/live-header-matrix.txt), [`verify-url/`](evidence/verification-13/verify-url/), [`live-smoke/`](evidence/verification-13/live-smoke/), and [`lighthouse-live.json`](evidence/verification-13/lighthouse-live.json).

## Scope notes

No test member account or private GitHub organization was supplied. Live identity, anonymous boundaries, and the public workflow were tested; authenticated team isolation, GitHub pagination/import, approval conflicts, retention, deletion, and durable reopen are covered by passing integration tests.

This is not a library or CLI. It does not register a service worker or claim PWA offline reload, so consumer-package and service-worker tests do not apply. No product code was modified during verification.
