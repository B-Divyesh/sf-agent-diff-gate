# Diff Gate independent verification 24 — FAIL

- **Candidate:** `e43c4da31769b958ba9b70a575f7b8fd5e3cd458`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC
- **Verdict:** **FAIL — do not release.**

The deployment-only problem reported previously is repaired: fresh evidence
shows the live service is this exact candidate and is configured safely. The
candidate nevertheless fails the product contract because an ordinary failed
sign-in flow has no human-readable recovery path.

## Release-blocking defect

### Medium — canceled Entra sign-in exposes a raw server error, not a recovery path

Diff Gate requires Entra sign-in before a team can use the real review
workflow. A user who cancels or is denied at Entra is returned with an OAuth
`error` query parameter rather than an authorization `code`. Fresh request:

```text
GET /auth/callback?error=access_denied
HTTP 400
Failed to deserialize query string: missing field `code`
```

This is framework error text. It neither says that sign-in was cancelled nor
offers a way to retry, return to Diff Gate, or use the sample. It violates the
factory error-state requirement that errors say what happened and the next
action, on the mandatory route into the real team product. Handle provider
`error`/`error_description` callbacks explicitly and render a normal Diff Gate
recovery page with **Try sign-in again** and **Try sample data** actions.

No other release-blocking defect was found.

## First read and demo

**PASS.** A cold, new desktop browser received HTTP 200 with no console or page
errors. Its first screen says:

- **What:** “Review agent-authored changes before merge.”
- **For whom:** small software teams needing a required owner and test
  evidence before an agent-authored change lands.
- **What to click:** **Try it with sample data**, immediately followed by an
  explanation that it opens changed files, test evidence, and owner checks.

At 390×844 the entire sample button was visible at `x=20`, `y=542.2`,
`207.6×46.3` CSS px. One click opened the isolated packet and the persistent
“Demo — sample data, nothing is saved” banner.

Independent live demo exercise passed: approval was disabled until both
required checks were marked reviewed, then enabled; the packet exported as
`diff-gate-packet.json` containing the expected title, three changed paths,
and four checks; approval became retained; reset restored the sample; and
Start for real cleared `demo:diff-gate` session storage. The complete flow had
five requests, all same-origin, with no GitHub request and no console error.

## Claims gate — all required commands passed

`npm ci` from the clean checkout installed 58 packages with zero reported
vulnerabilities. All 21 exact commands in `.factory/claims.json` passed:

| Claim | Result |
| --- | --- |
| `sample-sandbox` | PASS |
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

## Local quality gate

All passed from this clean candidate:

```text
npm test                                      16 Node tests + 25 Playwright tests
npx tsc --noEmit
npm run build                                 dist/ produced
cargo fmt --check
cargo test                                    21 tests
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
./scripts/verify-runtime-contract.sh
```

The production web build is 22.86 kB JS (7.28 kB gzip) and 12.23 kB CSS
(3.62 kB gzip), below the static budget. The container build could not be run
locally because Docker, Podman, and Buildah are absent from this worker; the
live build identity and byte-for-byte static asset checks below compensate for
that verification limitation, not for the reported defect.

## Fresh live deployment evidence

**PASS — the prior deployment-only failure is not present.**

- `GET /health` returned HTTP 200, `status:"ok"`, build
  `e43c4da31769b958ba9b70a575f7b8fd5e3cd458`, and durable storage ID
  `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- `./scripts/verify-live-deployment.sh ... '' e43c4da...` passed. It confirmed
  the stateful SQLite production control plane, exactly one running replica,
  Azure Files `/data`, one concurrent durable storage identity, and the
  expected build.
- `GET /api/auth/status` reports `service_ready:true`,
  `entra_sign_in_configured:true`, and GitHub App setup available.
- `/auth/entra` redirects only to
  `sociobotcustomers.ciamlogin.com/.../oauth2/v2.0/authorize`, with the
  Diff Gate callback and PKCE `code_challenge_method=S256`.
- The live rate probe observed exactly **40** accepted requests from one
  client and **60** HTTP **429** responses; every rejection contained
  `Retry-After: 1`. Health is exempt as documented.
- SHA-256 values for live `/`, hashed JS, hashed CSS, and
  `change-control.webp` exactly match `dist/` at this commit.

## Browser, accessibility, privacy, and HTTP

- Four independent cold loads, desktop 1440px and 390×844 mobile, all public
  routes, and the designed 404 passed with no page/console errors. The factory
  `verify-url.sh` also passed: title, `lang=en`, exactly one `h1`, one `main`,
  complete image alt text, and labeled buttons.
- Axe found zero serious or critical issues. Keyboard focus was visible as a
  3px coral outline. At 390px, 200% root text size produced no horizontal
  overflow. Under reduced motion, the demo had zero running animations.
- The live smoke used fresh browser contexts, approved/exported a loaded demo
  offline, and confirmed no off-origin request or third-party runtime script.
  CSP is delivered as a response header with `frame-ancestors 'none'`; HSTS,
  `nosniff`, and strict-origin referrer policy are present.
- Hashed JS/CSS are `public, max-age=31536000, immutable`; the WebP is
  `max-age=3600, must-revalidate`; documents are `no-cache`. An unknown route
  returns HTTP 404 with `X-Diff-Gate-Route: not-found` and
  `X-Robots-Tag: noindex`.
- Fresh mobile Lighthouse: **Performance 99, Accessibility 100, Best
  Practices 100, SEO 100**; FCP 1.0 s, LCP 1.7 s, TBT 120 ms, CLS 0.

The actual tenant-user sign-in and private GitHub installation cannot be
completed without a factory test identity. Tenant restriction, PKCE, service
readiness, live access boundaries, and the corresponding local integration
claims were verified. There is no library/CLI consumer or PWA service-worker
surface for this web-with-backend product.

## Evidence

Supporting live headers, downloaded asset hashes, Lighthouse output, and
desktop/mobile screenshots are in
`.factory/verification-24-artifacts/`. The artifact set records the candidate
served by the URL above.
