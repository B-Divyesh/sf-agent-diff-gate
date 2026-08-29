# Independent verification 6 — FAIL

**Candidate:** `eb8a164db197462ae1f62a942933ca52e095301a`  
**Live URL:** <https://agent-diff-gate.sociobot.in>  
**Verified:** 2026-08-29 UTC from this clean checkout

## Decision

**FAIL — deployment configuration blocks the real job.** The live HTML, JS, and CSS SHA-256 hashes match the production build from this candidate, and `/health` reports this exact SHA. The sample product, quality gates, accessibility checks, privacy checks, and rate limiting pass. But live `/api/auth/status` reports both `entra_sign_in_configured:false` and `github_app_configured:false`; the real-work panel has no sign-in action and says Entra is not configured. A team cannot authenticate, set a policy, import a PR, retain a real packet, or record a real owner approval. That fails the researched brief's smallest useful product and the end-to-end Definition of Done.

## Required first checks

### Claims gate — PASS

`.factory/claims.json` exists. After `npm ci` (58 packages; 0 reported vulnerabilities), every one of its 12 exact commands passed from the demo-capable clean checkout:

| Claim | Result |
|---|---|
| `sample-sandbox`, `packet-export`, `audit-export`, `no-third-party-runtime`, `sociobot-billing` | each exact Playwright grep command: 1/1 pass |
| `team-packet-boundary`, `named-approval`, `entra-team-installation`, `github-complete-import`, `repository-policy`, `retention-deletion`, `audit-history` | each exact `cargo test <name>` command: 1/1 pass |

### Cold first read — PASS

The cold 1440px live home screen says **“Review agent changes before merge”**, names **small software teams** needing an owner and evidence, and provides **“Try it with sample data”** with **“Opens a complete review packet.”** One click opens `/demo` with a realistic PR, changed contract/migration paths, test evidence, risky owner checks, and the persistent **“Demo — sample data, nothing is saved”** banner. Evidence: `verification-artifacts-6/live-home-cold-desktop.png` and `live-demo-desktop.png`.

## Release-blocking finding

### Critical — live production cannot review a real pull request

Fresh live evidence:

- `GET /health` → `200 {"status":"ok","build":"eb8a164db197462ae1f62a942933ca52e095301a"}`.
- `GET /api/auth/status` → `200 {"authenticated":false,"entra_sign_in_configured":false,"github_app_configured":false,"install_url":null,...}`.
- The live home says: **“Sociobot Entra sign-in is not configured on this deployment. The sample demo still works without an account.”**

The supplied sample is not a substitute for the real, team-bound GitHub review workflow. Provision the approved Sociobot Entra External ID settings and a team-mapped GitHub App installation, then reverify a production sign-in, repository-policy save, PR import, server-recorded test evidence, named-owner approval, audit export, retention, and deletion. The code's Entra-only authority and team-scoping claim tests pass, but this disabled deployment cannot establish that user-facing flow.

## Independent product exercise

- **Demo normal/recovery:** resolved both risky checks, exported `diff-gate-packet.json`, approved the sample, then reset it. Export contains the expected owner, three changed paths, and four checks. Reset restored `HOLD` and both unresolved checks.
- **Invalid input recovery:** an empty restored-license submission focused the labelled token field and showed the native required-field message; no request was made.
- **Desktop/mobile/keyboard:** 390px dark reduced-motion demo had `scrollWidth === clientWidth === 390`, no active animations, no console/page errors, and visible solid focus on keyboard targets. Tab reached every demo control and header navigation.
- **Accessibility:** live Axe scans at `/`, `/demo`, `/privacy`, `/terms`, and an HTTP 404 route found zero serious/critical violations. Each route has one h1, main landmark, route title, `lang=en`, and visible focus.
- **Privacy:** fresh demo `/demo` → resolve → export → approve → reset requested only same-origin HTML, JS, and CSS. Cold home additionally requested only same-origin `/api/auth/status` and the self-hosted hero image. No third-party scripts, analytics, page errors, or console errors were observed.

## Quality, runtime, headers, and deployment parity

- `npm test`: PASS — 16/16 Playwright tests. Vitest intentionally has no files and exits successfully with `--passWithNoTests`.
- `npx tsc --noEmit`, `cargo fmt --check`, `cargo test` (12/12), and `cargo clippy -- -D warnings`: PASS.
- `npm run build` and `cargo build --release`: PASS — JS 22.58 kB (7.56 kB gzip), CSS 12.00 kB (3.58 kB gzip); below the static budget. Container build is unavailable because this verifier image has no `docker` or `podman` executable.
- Native service with only `PORT` and `BUILD_SHA` served `/health`, created its default `/data` SQLite database, and identified the requested SHA.
- Candidate/live parity: live `index-5gOcXlLD.js` and `index-DP1-EDly.css` SHA-256 exactly equal freshly built `dist` assets; live `/health` has the candidate SHA.
- Rate limit: 100 concurrent HTTP/2 requests to `/api/packets` with one forwarded client identity yielded **40×401 and 60×429**; each 429 had `Retry-After: 1`. Observed allowance: **40 requests per client per one-second window**. Local native runtime reproduced the same result.
- Response headers: CSP is a response header and includes `frame-ancestors 'none'`; HSTS, `nosniff`, and strict-origin referrer policy are present. HTML is `no-cache`; hashed JS has `public, max-age=31536000, immutable`; hero WebP is 136,640 bytes and one-hour revalidated.
- Lighthouse live home: performance **94**, accessibility **100**, LCP **1.7 s**, CLS **0** (report: `verification-artifacts-6/lighthouse-live-home.json`).

## Scope notes

This is a web-with-backend, not a library/CLI/PWA; consumer-package and service-worker update checks do not apply. Docker could not be run because the executable is absent from the verifier container, not because the candidate's Dockerfile failed. The Dockerfile was reviewed and uses the required unpinned Rust-major builder, build argument, non-root user, and port 8080.

## Required before release

1. Factory administrators must provision the live Entra client/authority/team claim and team-bound GitHub App mapping through the approved secret path.
2. Re-run this verification through a real production account and a representative PR, including the state-changing audit/retention paths.
