# Diff Gate verification 27 — PASS

- Candidate: `73bc0ee4df101b9d4254b276731d7ecc36dd1076`
- Live URL: <https://agent-diff-gate.sociobot.in>
- Verified: 2026-09-01 UTC
- Result: **PASS**

## First-read gate

Confirmed on a new 1440×900 browser context that the first screen answers all
three required questions in plain words:

- What it does: “Review agent-authored changes before merge.”
- Who it is for: small software teams that require an owner and test evidence.
- What to do first: choose **Try it with sample data**.

The action was fully visible at 390×844 (`y=542.2`, height `46.3`) and opened a
working packet in one click. Its adjacent text says that the sample contains
changed files, test evidence, and owner checks.

## Claims gate

Confirmed that `.factory/claims.json` exists. After `npm ci` installed the
locked dependencies, every listed command was run separately from the clean
candidate checkout. All 23 passed.

| Claim | Result and observed evidence |
|---|---|
| `sample-sandbox` | PASS — demo state was cleared on exit and requests stayed same-origin. |
| `packet-export` | PASS — exported JSON contained the sample title, three changed files, and four checks. |
| `demo-query-path` | PASS — `?demo=1`, banner controls, mutation, and reset worked. |
| `mobile-first-action` | PASS — the complete action fit in the first 390×844 screen. |
| `no-merge-action` | PASS — approval recorded a decision without a code-hosting request. |
| `team-packet-boundary` | PASS — the two-team read and approval fixture enforced team scope. |
| `named-approval` | PASS — missing evidence and the wrong owner were rejected; saved evidence and audit persisted. |
| `entra-team-installation` | PASS — only the Sociobot authority was accepted and installations remained team-specific. |
| `github-complete-import` | PASS — both fixture pages and 102 changed paths were evaluated. |
| `github-revision-refresh` | PASS — a changed revision cleared prior evidence and stopped approval. |
| `github-app-provisioning` | PASS — the manifest was read-only and setup state remained team-specific. |
| `repository-policy` | PASS — sensitive paths and owners remained team-specific and exact-matched. |
| `retention-deletion` | PASS — retention and explicit deletion removed packet and audit records. |
| `audit-history` | PASS — one concurrent approval succeeded, one returned a conflict, and history remained team-specific. |
| `audit-export` | PASS — the signed-in fixture exported the packet and its audit entry. |
| `no-third-party-runtime` | PASS — the full sample flow requested only the product origin. |
| `github-file-limit` | PASS — the import stopped above 10,000 changed files with a clear error. |
| `retention-limits-and-cleanup` | PASS — default 90 days, range 1–3,650, and read-time cleanup were confirmed. |
| `runtime-port-health` | PASS — a release binary started with `PORT` only and returned build and store identities. |
| `durable-store-replacement` | PASS — reopening the SQLite file retained its store identity. |
| `stateful-worker-deploy` | PASS — the hook accepted only this product and port 8080 in its fixture. |
| `production-stateful-template` | PASS — the rendered template combined one replica, `/data`, the SQLite URL, and image. |
| `deployment-health-replacement` | PASS — fixture evidence required 100 healthy responses on each side and one store identity. |

## Repository quality gates

Confirmed all available local gates:

- `npm ci`: PASS; 58 packages installed and npm reported 0 vulnerabilities.
- `npm test`: PASS; 17 Node tests, the Vite production build, and 27 Playwright tests.
- `npx tsc --noEmit`: PASS.
- `cargo test --all-targets`: PASS; 24 tests.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS.
- `npm run build`: PASS; `dist/` produced.
- `cargo build --release`: PASS.
- `./scripts/verify-runtime-contract.sh`: PASS.

The built first-load files are 22,863 bytes of JavaScript and 12,233 bytes of
CSS. The hero image is 136,640 bytes. These are below the stated 200 kB,
50 kB, and 300 kB limits.

## Independent live product flow

Confirmed the smallest useful sample flow independently on desktop and 390px
mobile. The packet opened with two required-owner checks and approval disabled.
Keyboard Enter and Space completed the checks, enabled approval, exported
`diff-gate-packet.json`, and recorded approval. Reload retained the decision;
**Reset demo** restored the initial two-check state.

Confirmed boundary and recovery behavior:

- 200% text at 390px had no horizontal overflow.
- All 13 rendered demo controls measured at least 44×44 CSS pixels.
- The cancelled sign-in route returned a clear recovery page with three next steps.
- An anonymous packet read returned 401 with “Sign in with Sociobot before opening team packets.”
- A malformed packet body returned 400 with “Invalid request data. Send a complete JSON object and try again.”
- An unknown route returned the designed noindex recovery page; non-navigation requests retained HTTP 404.
- Every public navigation link returned its expected page. The sign-in link returned 307 to the approved Sociobot tenant.

No console error, page error, failed same-origin resource, or unexpected
off-origin request was observed in the cold load or sample flow.

Checked the repository and live registration state for PWA behavior. Diff Gate
does not register a service worker and does not claim offline reload, so a
service-worker update cycle is not applicable. The already loaded sample did
remain usable while the browser was offline.

## Accessibility and responsive behavior

Checked `/`, `/demo`, `/privacy`, `/terms`, the not-found page, and cancelled
sign-in recovery in desktop light mode and 390px dark/reduced-motion mode.
Each page had `lang="en"`, one `main`, one `h1`, its own title, no horizontal
overflow, and no serious or critical Axe finding.

Confirmed keyboard operation for the complete review flow and a visible 3px
focus outline. Its measured contrast was 4.07:1 on the light surface and
6.84:1 on the dark surface. Reduced-motion mode reported no active animations.

## Privacy, identity, and response policy

Confirmed from Playwright request logs that the cold page and complete sample
flow contacted only `https://agent-diff-gate.sociobot.in`. There were no
analytics, remote fonts, or third-party runtime scripts.

Checked live response headers on documents, assets, API responses, and the
not-found route. They included HSTS, `X-Content-Type-Options: nosniff`, strict
referrer policy, and a CSP with `frame-ancestors 'none'`. HTML used
`Cache-Control: no-cache`; hashed JavaScript and CSS used one-year immutable
caching; the hero used one-hour revalidation.

Confirmed that sign-in redirects only to
`sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`, uses
the configured client and callback, and includes PKCE `S256`.

## Performance

Checked the live site with Lighthouse 12.8.2 using the mobile profile:

- Performance 100
- Accessibility 100
- Best Practices 100
- SEO 100
- FCP 1.0 s; LCP 1.7 s; CLS 0; TBT 30 ms; Speed Index 1.0 s
- 174,594 transferred bytes across six requests; zero third-party requests

## Backend and deployment identity

Confirmed `/health` returned HTTP 200 with build
`73bc0ee4df101b9d4254b276731d7ecc36dd1076` and store identity
`1da0c91d-ce8d-4ea1-983d-665beebfbe13`. A concurrent set of 100 health
requests all returned that same build and store identity.

The current store identity also matches the committed verification-25 health
record from earlier build `1ef3f4bdfaf67e8a7517f46757ed20551e986b94`,
confirming that the durable identity remained stable across that deployment
change.

Confirmed the documented allowance on both `/api/auth/status` and the protected
`/api/packets` route: one client received 40 normal responses, then 60 responses
with HTTP 429. Every 429 response included `Retry-After: 1`. Health remained
available separately, as documented.

Confirmed live-to-candidate identity in two ways: `/health` reported the exact
candidate SHA, and SHA-256 values matched between live and local production
output for `index.html`, JavaScript, CSS, and `change-control.webp`.

## Defects and limitations

No critical, high, medium, or low product defects were found.

The worker has no Docker, Podman, or Buildah executable, so the container image
could not be rebuilt in this environment. The frontend production build, Rust
release build, `PORT`-only runtime contract, live asset hashes, and deployed
candidate identity all passed.

No verifier tenant account was provided, so an interactive signed-in session
and a real repository installation were not created. The live tenant redirect,
PKCE callback and recovery were confirmed, while authenticated team behavior,
GitHub import, revisions, policies, approval, audit, retention, and deletion
passed their integration tests.
