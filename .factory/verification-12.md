# Independent verification 12 — FAIL

**Candidate:** `d150b3243f60c12f3c477aa778fae94b5df7c02a`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC from the clean candidate checkout

## Release decision

**FAIL.** The deployed image is the requested candidate, and its UI, sample workflow, claims, source tests, accessibility, privacy behavior, Entra redirect, request allowance, and performance pass. The real signed-in product is nevertheless unsafe: production currently runs up to three replicas with no durable volume, while every process uses its own SQLite database under `/data`.

This is active, not theoretical. A fresh 100-request concurrent `/health` smoke returned three different `storage_id` values from the same build. Sessions, team policies, packets, evidence, approvals, and audit history can therefore disappear or vary between requests. The repository's own live deployment verifier exits 1 against production.

## Release-blocking defect

### Critical — production serves three independent ephemeral packet stores

Fresh Azure control-plane output for `sf-agent-diff-gate`:

```json
{
  "image": "sociobotregistry.azurecr.io/sf-agent-diff-gate:d150b3243f60",
  "scale": { "minReplicas": 1, "maxReplicas": 3 },
  "volumes": null,
  "volumeMounts": null,
  "env": [{ "name": "PORT", "value": "8080" }]
}
```

The process defaults to `sqlite:/data/diff-gate.db?mode=rwc`; without a mounted volume, `/data` is container-local. The concurrency smoke proved that all three configured replicas are serving:

```json
{
  "requests": 100,
  "statusCounts": { "200": 100 },
  "builds": ["d150b3243f60c12f3c477aa778fae94b5df7c02a"],
  "storageIds": [
    "576c6921-7300-4605-93f4-737c7d777f05",
    "6f7615ae-236c-4e39-b103-a74eeb80c66e",
    "476dc9ad-aa91-4445-aafa-6b4405efd9bb"
  ]
}
```

This breaks the real job-to-be-done: an accountable approval packet cannot be trusted if the reviewer, owner, and audit read different stores. Entra PKCE state and sessions are affected too.

Evidence: [`live-containerapp-config.json`](evidence/verification-12/live-containerapp-config.json), [`live-load-smoke.json`](evidence/verification-12/live-load-smoke.json), and [`live-deployment-trace.log`](evidence/verification-12/gates/live-deployment-trace.log).

**Required repair:** run the repository's stateful deployment path so the existing Azure Files share is mounted at `/data`, set `DATABASE_URL` to that durable path, and hold SQLite to exactly one replica. An authorized repairer must then run `./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in --replace` and prove the same `storage_id` survives a revision replacement. This verifier did not mutate deployment state.

## Mandatory first checks

### Cold first read — PASS

A fresh 1440 × 1000 browser context showed:

- Job: **“Review agent-authored changes before merge.”**
- Audience/outcome: small software teams needing a required owner and test evidence.
- First action: visible **“Try it with sample data”**, with the explanation that it opens changed files, test evidence, and owner checks.

The action was fully inside the first viewport. The cold load made only same-origin requests and logged no console or page errors.

Evidence: [`live-first-read.json`](evidence/verification-12/live-first-read.json) and [`live-cold-desktop.png`](evidence/verification-12/live-cold-desktop.png).

### Claims gate — PASS after the documented install

`.factory/claims.json` exists with 20 entries. As explicitly requested, every command was first invoked before any dependency installation. All 13 Rust/runtime commands passed; the seven Playwright commands could not start because the clean clone did not yet contain `@playwright/test`. After the required `npm ci`, every exact browser claim command passed. The pre-install and installed transcripts are both retained.

| Claim | Result |
| --- | --- |
| `sample-sandbox` | PASS, 1/1 after install |
| `packet-export` | PASS, 1/1 after install |
| `demo-query-path` | PASS, 1/1 after install |
| `mobile-first-action` | PASS, 1/1 after install |
| `no-merge-action` | PASS, 1/1 after install |
| `team-packet-boundary` | PASS, 1/1 |
| `named-approval` | PASS, 1/1 |
| `entra-team-installation` | PASS, 1/1 |
| `github-complete-import` | PASS, 1/1 |
| `github-revision-refresh` | PASS, 1/1 |
| `github-app-provisioning` | PASS, 1/1 |
| `repository-policy` | PASS, 1/1 |
| `retention-deletion` | PASS, 1/1 |
| `audit-history` | PASS, 1/1 |
| `audit-export` | PASS, 1/1 after install |
| `no-third-party-runtime` | PASS, 1/1 after install |
| `github-file-limit` | PASS, 1/1 |
| `retention-limits-and-cleanup` | PASS, 1/1 |
| `runtime-port-health` | PASS |
| `durable-store-replacement` | PASS, 1/1 |

Evidence: [`claims/`](evidence/verification-12/claims/). The live landing/legal copy and README were cross-checked against the registry; no unlisted reliance-worthy claim was found.

## Clean-checkout gates and production builds

All available source gates pass:

```text
npm ci                                      58 packages, 0 vulnerabilities
npm test                                    24 Playwright tests passed
npx tsc --noEmit                            passed
npm run build                               passed; dist/ produced
cargo fmt --all -- --check                  passed
cargo test --all                            20 passed
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release                       passed
./scripts/verify-runtime-contract.sh        passed with PORT-only startup
```

Vitest reports no unit files; browser coverage is in Playwright. No lint script exists beyond TypeScript, Rust formatting, and Clippy. No Docker/Podman/Buildah/nerdctl executable is installed in this verifier container, so a local Docker wrapper build could not be attempted. The equivalent frontend and Rust release builds passed, the release binary passed its runtime contract, and the live ACR image identifies this candidate.

The frontend is 22,499 B JavaScript (7,223 B gzip), 12,233 B CSS (3,623 B gzip), and a 136,640 B hero image. These are below the stated budgets.

Evidence: [`gates/`](evidence/verification-12/gates/) and [`asset-identity-and-sizes.txt`](evidence/verification-12/asset-identity-and-sizes.txt).

## End-to-end product exercise

The public sample flow passes independently in a fresh live browser:

1. Keyboard Enter opened the sample from the first screen.
2. The packet showed owner Mira Chen, three changed files, four evidence checks, and two unresolved owner checks.
3. Approval was disabled with a specific explanation before evidence was resolved.
4. Keyboard Enter resolved both checks and enabled approval.
5. Approval became immutable and survived reload.
6. Export produced valid JSON with the title, three files, four checks, and approved status.
7. Reset restored the two unresolved checks.
8. Start for real removed `sessionStorage['demo:diff-gate']` and the demo banner.

The entire normal flow used only `https://agent-diff-gate.sociobot.in`. Anonymous protected packet, settings, policy, approval, and GitHub-import calls returned 401. An expired Entra callback returned 400 with “That Sociobot sign-in link expired. Start again.” Unknown routes return the designed recovery UI with HTTP 404, `X-Diff-Gate-Route: not-found`, and `X-Robots-Tag: noindex`.

Invalid/recovery checks also pass in the signed-in UI fixture: retention `0`, a malformed URL, and an empty title are blocked by native validation without an API call; valid retention `30` recovers; an incomplete policy gets a plain server error and succeeds after correction; an import without a GitHub App gives a specific next step.

Evidence: [`live-independent-e2e.json`](evidence/verification-12/live-independent-e2e.json), [`live-invalid-recovery-ui.json`](evidence/verification-12/live-invalid-recovery-ui.json), and [`live-api-boundaries.txt`](evidence/verification-12/live-api-boundaries.txt).

## Live identity, request allowance, privacy, and caching

- `/health` returns candidate build `d150b3243f60c12f3c477aa778fae94b5df7c02a`.
- Local and live SHA-256 hashes match exactly for HTML, JavaScript, CSS, and the hero image.
- `/auth/entra` redirects only to `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650` with the production callback and PKCE S256.
- An 80-request HTTP/2 burst from one forwarded client returned 40×200 and 40×429. Every 429 had `Retry-After: 1`. **Observed allowance: 40 requests per client per second.** `/health` is exempt as documented.
- The demo's full request log is same-origin only; there are no analytics or third-party runtime scripts.
- HTML/API responses are `no-cache`; hashed assets are immutable for one year; stable images revalidate hourly.
- Responses include HSTS, `nosniff`, strict-origin referrer policy, and CSP with `frame-ancestors 'none'`.
- All public links resolve; the sign-in link correctly returns a 307 to the Sociobot tenant.

Evidence: [`live-health.json`](evidence/verification-12/live-health.json), [`live-rate-limit.json`](evidence/verification-12/live-rate-limit.json), [`live-header-matrix.txt`](evidence/verification-12/live-header-matrix.txt), [`live-link-crawl.json`](evidence/verification-12/live-link-crawl.json), and [`live-assets/`](evidence/verification-12/live-assets/).

## Accessibility, mobile, motion, and performance

- `/`, `/demo`, `/privacy`, `/terms`, and the 404 recovery view each have `lang=en`, one main landmark, one h1, and route-specific titles.
- Axe found zero serious or critical issues in light and dark modes.
- The complete keyboard review flow works; focus uses a visible 3 px coral outline.
- At 390 × 844, the first action is fully visible, all rendered targets are at least 44 × 44 CSS px, body text is 16 px, and there is no horizontal overflow.
- Browser zoom is not disabled; the repository's 390 px, 200% text reflow test passes without overflow or clipped navigation.
- With reduced motion requested, no nonzero animation or transition remains.
- The standard URL verifier reports no console, title, language, landmark, alt-text, or unnamed-button failure.
- Fresh mobile Lighthouse: **99 performance, 100 accessibility, 100 best practices, 100 SEO**; FCP 0.9 s, LCP 1.7 s, TBT 80 ms, CLS 0, total transfer 170 KiB.

Evidence: [`live-mobile-accessibility.json`](evidence/verification-12/live-mobile-accessibility.json), [`live-smoke/`](evidence/verification-12/live-smoke/), [`verify-url/`](evidence/verification-12/verify-url/), and [`lighthouse-live-rerun.json`](evidence/verification-12/lighthouse-live-rerun.json).

## Scope notes

No test member account or private GitHub organization was supplied. The live Entra authority/callback/PKCE and anonymous boundaries were verified, while authenticated team isolation, GitHub pagination/import boundaries, approval, audit, retention, deletion, durable reopen, and concurrent approval are covered by passing isolated integration tests.

This is a web-with-backend product, not a library or CLI. It does not register a service worker or claim PWA/offline reload, so service-worker update testing is not applicable. The already-loaded demo remains usable offline.

No product code was modified during verification.
