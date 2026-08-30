# Diff Gate independent verification 25 — FAIL

- **Candidate:** `1ef3f4bdfaf67e8a7517f46757ed20551e986b94`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC
- **Verdict:** **FAIL — do not release.**

The live service is the exact candidate and its product flow is healthy. The
release is nevertheless blocked because a declared claim test does not run
successfully from a clean checkout. The repository gives Playwright 120
seconds to compile and start the Rust server; a cold build exceeds that limit.
The exact `sample-sandbox` command fails before its assertion runs.

## Release-blocking defect

### Medium — a declared claim test times out from a clean checkout

After `npm ci`, the first exact command in `.factory/claims.json` was run:

```text
npm run test:browser -- --grep @claim:sample-sandbox
Error: Timed out waiting 120000ms from config.webServer.
```

The failure was independently reproduced with a brand-new
`CARGO_TARGET_DIR`; it again timed out at 120 seconds. Evidence is in
[`cold-claim-sample-sandbox.log`](verification-25-artifacts/cold-claim-sample-sandbox.log).
`playwright.config.ts` starts `cargo run --quiet` but permits only 120 seconds
for `/health`. Once Rust has been compiled separately, the same claim passes
in 18.9 seconds. This proves that the claim behavior works, but it does not
repair the required clean-clone command.

The acceptance contract says any failing claim test is release-blocking.
Increase the cold server timeout or prebuild the backend as part of the claim
test command, then prove the exact command in a fresh target directory.

## Other defects

### Low — malformed API JSON exposes framework error text

Unauthenticated requests with valid JSON shapes correctly return HTTP 401 and
the plain message “Sign in with Sociobot before opening team packets.”
Malformed write bodies are rejected earlier by the Axum extractor and return
plain-text HTTP 422 messages such as:

```text
Failed to deserialize the JSON body into the target type: missing field `title`
```

No data is read or changed, so this is not an authorization bypass. It is an
error-copy and response-consistency defect. Map JSON extraction failures to a
stable JSON error that says which input is invalid and what to correct.

### Low — the required landing copy audit is stale

`.factory/copy-audit.md` records “Packets are visible only to that reviewer’s
team.” The candidate and live page say “Packets are visible only to their
signed-in team.” The current sentence is plain and covered by the
`team-packet-boundary` claim, but the audit is no longer an exact extraction as
required.

## First read and demo

**PASS.** A cold 1440×900 browser immediately answers all three questions:

- **What:** “Review agent-authored changes before merge.”
- **For whom:** small software teams needing a required owner and test
  evidence before an agent-authored change lands.
- **First click:** **Try it with sample data**, next to the explanation that it
  opens changed files, test evidence, and owner checks.

One click opens an opinionated sample for organization-level retention. The
persistent banner says “Demo — sample data, nothing is saved” and provides
**Reset demo** and **Start for real**.

The fresh live flow confirmed:

- three changed paths and four evidence checks load immediately;
- approval is disabled while two required-owner checks remain;
- both checks work by keyboard, after which approval becomes enabled;
- JSON export contains the title, owner, three files, and four checks;
- approval is retained in `demo:diff-gate` session storage;
- reset restores the two unresolved checks;
- Start for real removes the demo storage key and returns to sign-in.

The complete request log is same-origin and contains no analytics or
third-party runtime request. See
[`live-e2e.json`](verification-25-artifacts/live-e2e.json).

## Claims gate

All 21 commands listed in `.factory/claims.json` were run individually. The
gate is **FAIL** because the cold execution of the first command failed.

| Claim | Result |
| --- | --- |
| `sample-sandbox` | **FAIL cold** — server timeout; PASS after precompile |
| `packet-export` | PASS |
| `demo-query-path` | PASS |
| `mobile-first-action` | PASS |
| `no-merge-action` | PASS |
| `team-packet-boundary` | PASS |
| `named-approval` | PASS |
| `entra-team-installation` | PASS |
| `github-complete-import` | PASS |
| `github-revision-refresh` | PASS |
| `github-app-provisioning` | PASS |
| `repository-policy` | PASS |
| `retention-deletion` | PASS |
| `audit-history` | PASS |
| `audit-export` | PASS |
| `no-third-party-runtime` | PASS |
| `github-file-limit` | PASS |
| `retention-limits-and-cleanup` | PASS |
| `runtime-port-health` | PASS |
| `durable-store-replacement` | PASS |
| `stateful-worker-deploy` | PASS |

The runtime-contract command performed a cold release compilation in 5m14s,
then passed PORT-only startup, build identity, and durable-store identity.

## Local quality gates

After the initial claim failure, all broader gates passed:

```text
npm ci                                      58 packages; 0 vulnerabilities
npm test                                    16 Node + 26 Playwright tests
cargo test --all-targets                    22 passed
cargo fmt --all -- --check                  passed
cargo clippy --all-targets --all-features -- -D warnings
npx tsc --noEmit                            passed
npm run build                               passed; dist/ produced
cargo build --release                       passed
./scripts/verify-runtime-contract.sh        passed
```

The production web build contains 22.86 kB JS (7.28 kB gzip) and 12.23 kB CSS
(3.62 kB gzip). Docker, Podman, and Buildah are unavailable in this worker, so
the Dockerfile itself was not rebuilt. The PORT-only release runtime and the
byte-identical live assets provide separate runtime and deployment evidence.

## Fresh live deployment evidence

The previously reported deployment-only failure is absent.

- `/health` returned HTTP 200 with build
  `1ef3f4bdfaf67e8a7517f46757ed20551e986b94` and storage identity
  `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- One hundred concurrent `/health` responses returned that same build and
  storage identity.
- Live `/`, hashed JS, hashed CSS, and `change-control.webp` are byte-for-byte
  identical to this candidate's `dist/` files.
- `/api/auth/status` returned `service_ready:true`,
  `entra_sign_in_configured:true`, and GitHub App setup available.
- `/auth/entra` redirected only to
  `sociobotcustomers.ciamlogin.com/.../oauth2/v2.0/authorize` with the product
  callback and PKCE S256.
- Canceled and missing-detail callbacks render a product recovery screen.
  A hostile `error_description` was not reflected. A stale state currently
  returns a concise JSON 400 message rather than the recovery page.
- Valid-shape anonymous packet, settings, and GitHub import writes all
  returned 401 without changing data.
- The live limiter allowed exactly **40 requests per second** from one client.
  Requests 41–100 returned HTTP **429**, each with `Retry-After: 1`. `/health`
  remained HTTP 200 after the probe, as documented.

Only public product endpoints were inspected. No external service resources,
settings, secrets, databases, or deployments were read or changed.

## Accessibility, mobile, privacy, and HTTP

- The factory URL verifier passed the home and canceled-sign-in routes: HTTP
  200, `lang=en`, title, one `h1`, one `main`, complete image alternatives,
  labeled buttons, and no console errors.
- Independent Axe scans found zero serious or critical issues on `/`, `/demo`,
  `/privacy`, `/terms`, the 404 view, and canceled-sign-in recovery in light
  and dark treatments.
- Keyboard activation resolved every demo check. Focus uses a visible 3px
  coral outline. The skip link and route focus behavior passed.
- At 390×844 the complete sample action is inside the first viewport at
  `x=20`, `y=542.2`, `207.6×46.3` CSS px. Minimum rendered interactive target
  size is 44px. There is no horizontal overflow at normal or 200% root text.
- Reduced-motion mode had zero running animations.
- The demo remained usable for review and export after the loaded context was
  taken offline. No service worker is registered or claimed; this is treated
  as a web-with-backend product, not a PWA.
- The browser recorded no page errors or console errors and only same-origin
  requests throughout the demo flow.
- CSP is a response header and includes `frame-ancestors 'none'`; HSTS,
  `nosniff`, and strict-origin referrer policy are present.
- Hashed JS/CSS use `public, max-age=31536000, immutable`; images use a
  one-hour revalidation policy; HTML and API responses use `no-cache`.
- Unknown routes return a designed HTTP 404 with `X-Diff-Gate-Route:
  not-found` and `X-Robots-Tag: noindex`.
- All expected public routes and assets returned their intended 200/404
  status. The social image is a real 1200×630 WebP.

Fresh mobile Lighthouse scored **99 performance, 100 accessibility, 100 best
practices, and 100 SEO**. FCP was 0.99s, LCP 1.74s, TBT 114ms, CLS 0, and total
transfer was 174,530 bytes. The 136,640-byte hero, JS, and CSS are within their
respective budgets.

## Scope limits and evidence

A real tenant-user sign-in and private GitHub installation could not be
completed without a test identity. Tenant restriction, PKCE, anonymous
boundaries, policy matching, pagination, revision invalidation, owner checks,
retention, deletion, audit export, and concurrent approval are covered by the
passing browser and backend integration tests.

Fresh screenshots, request/response logs, headers, hashes, rate evidence, URL
verification, and Lighthouse output are in
`.factory/verification-25-artifacts/`.
