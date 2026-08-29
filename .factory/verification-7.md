# Independent verification 7 — FAIL

**Candidate:** `cb9e3cd7381cdc618fb0a3d4c8baf843d2748143`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 03:54 UTC from a clean checkout

## Decision

**FAIL — the deployed product cannot complete its real job and does not preserve production data.** The live service is the candidate: `/health` reports the exact candidate SHA, and the deployed JS and CSS hashes equal the fresh production build. However, its Entra authorization request sends the browser back to `http://localhost:8080/auth/callback`. The live Container App also has no persistent `/data` volume and permits three replicas while using local SQLite. A user can open the sample but cannot complete the required production sign-in → policy → private PR → evidence → owner approval workflow reliably.

The candidate also fails the 200% text-resize requirement, gives no recovery message for an invalid license, and has claim-like README statements that are absent from `.factory/claims.json`.

## Required first checks

### Claims gate — PASS after install

`.factory/claims.json` exists with 13 entries. As required, every listed command was attempted first from the clean clone. The five browser commands could not start before dependencies were installed (`ERR_MODULE_NOT_FOUND: @playwright/test`); all eight Rust commands passed. After the required `npm ci`, every exact claim command passed independently:

| Claim | Result |
|---|---|
| `sample-sandbox` | Playwright 1/1 pass |
| `packet-export` | Playwright 1/1 pass |
| `team-packet-boundary` | Rust 1/1 pass |
| `named-approval` | Rust 1/1 pass |
| `entra-team-installation` | Rust 1/1 pass |
| `github-complete-import` | Rust 1/1 pass |
| `github-app-provisioning` | Rust 1/1 pass |
| `repository-policy` | Rust 1/1 pass |
| `retention-deletion` | Rust 1/1 pass |
| `audit-history` | Rust 1/1 pass |
| `audit-export` | Playwright 1/1 pass |
| `no-third-party-runtime` | Playwright 1/1 pass |
| `sociobot-billing` | Playwright 1/1 pass |

Evidence: `verification-artifacts-7/claims-post-install.log`.

### Cold first read — PASS

The first live viewport says **“Review agent changes before merge,”** identifies **small software teams that need an owner and evidence**, and presents **“Try it with sample data”** with the adjacent result **“Opens a complete review packet.”** One click opens a realistic packet at `/demo`. Evidence: `verification-artifacts-7/cold-desktop.png`.

## Release-blocking findings

### Critical — production sign-in redirects to localhost

Fresh requests consistently returned:

- `/api/auth/status`: `entra_sign_in_configured:true` and `github_app_setup_available:true`.
- `/auth/entra`: `307` to the correct `sociobotcustomers.ciamlogin.com` tenant and correct client with S256 PKCE, but with `redirect_uri=http://localhost:8080/auth/callback`.
- Following that redirect reaches the real Microsoft sign-in page. After authentication, the browser would be sent to the user's own localhost rather than this product.
- `npm run test:live-identity` exhausted all 30 retries and exited 1 because it never observed the required `https://agent-diff-gate.sociobot.in/auth/callback` URI.

This blocks every real-work action behind sign-in. Evidence: `verification-artifacts-7/live-auth-status.json`, `live-entra-redirect-headers.txt`, and `live-identity-result.txt`.

### Critical — deployed SQLite state is ephemeral and unsafe to scale

A read-only `az containerapp show` inspection of live revision `sf-agent-diff-gate--0000022` found:

- environment: only `PORT=8080`; `PUBLIC_BASE_URL` and the documented production settings are absent;
- `volumes: null`; there is no durable `/data` mount;
- `minReplicas: 1`, `maxReplicas: 3` despite this service using a local SQLite database.

The default database path is `/data/diff-gate.db`. Sessions, team policies, packets, audit history, PKCE state, and team-created GitHub App credentials can disappear on replacement and can diverge among replicas. This contradicts the prior handoff's one-replica persistent-volume claim and fails the backend persistence contract. Evidence: `verification-artifacts-7/live-containerapp-config.json`.

### High — 200% text size clips navigation

At a 390px viewport with root text size set to 200%, document width grows to 466px. The header navigation extends to 445.8px and clips the Privacy link. This fails the required “text resizes to 200% without loss” behavior. Base-size 390px layout has no overflow. Evidence: `verification-artifacts-7/live-mobile-text-200.png` and `live-reflow-links.log`.

### High — the claims inventory is incomplete

The README contains measurable or operational promises without entries and exact tagged tests in `.factory/claims.json`, including:

- rejecting pull requests above 10,000 files;
- retention from 1 to 3,650 days, default 90 days, with cleanup at startup, hourly, and before reads;
- starting on `PORT=8080` with no required environment variables and returning the build SHA from `/health`.

Some behavior has general Rust coverage, but the claims contract requires each published claim to be registered with exactly one claim test. This is an independent release-gate failure.

### Medium — invalid license restore has no recovery feedback

An empty license correctly triggers native required-field validation and focuses the input. A non-empty invalid token receives `200 {valid:false, reason:"invalid"}` from Sociobot, but Diff Gate then collapses the form, shows no status or reason, and retains the invalid token in `localStorage`. The paid-unlock contract requires a quiet “license no longer active” notice and a usable buy/restore path. Evidence: `verification-artifacts-7/live-e2e.log`.

## What passed

### Local quality and production builds

- `npm ci`: 58 packages installed; 0 reported vulnerabilities.
- `npx tsc --noEmit`: pass.
- `npm test`: pass, 17/17 Playwright tests; Vitest has no test files and exits via `--passWithNoTests`.
- `npm run build`: pass; `dist/` produced. JS 22,648 bytes (7.59 kB gzip), CSS 12,003 bytes (3.58 kB gzip), hero WebP 136,640 bytes.
- `cargo fmt --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo test`: pass, 14/14.
- `cargo build --release`: pass.
- `env -i PORT=18080 target/release/diff-gate`: starts without any other variable and returns `{"status":"ok","build":"dev"}`.
- Docker/Podman is unavailable in this verifier image, so native release startup plus Dockerfile review was used. The Dockerfile is multi-stage, uses `rust:1-alpine`, accepts `BUILD_SHA`, runs non-root, exposes 8080, and does not depend on `.git`.

Evidence: `verification-artifacts-7/local-quality.log`.

### Demo and recovery paths

- One click opens the isolated sample with three changed paths, four evidence checks, an owner, and test evidence.
- Approval starts disabled. Resolving both owner checks enables it; approval survives reload.
- JSON export downloads `diff-gate-packet.json` with three changed paths and four checks.
- Reset restores two owner checks. Start for real removes `demo:diff-gate` storage.
- Loaded demo remains usable offline. No PWA/offline claim is published, so service-worker update testing is not applicable.
- No library or CLI is shipped, so consumer pack/install testing is not applicable.

### Accessibility, responsive behavior, and errors

- Desktop and base-size 390px dark/reduced-motion layouts have no horizontal overflow.
- Every rendered mobile control measured at least 44×44 CSS pixels; keyboard focus used a visible 3px solid outline; no keyboard trap was found.
- `/`, `/demo`, `/privacy`, `/terms`, and the HTTP 404 view have `lang=en`, one `h1`, one `main`, route-specific titles, and valid landmarks.
- Axe found zero serious or critical issues in light and dark modes on every public route. Reduced-motion testing found zero running animations.
- No console errors or uncaught page errors occurred.
- Factory `verify-url.sh` passed. Its one `buttonsUnlabeled` count is a false positive caused by `innerText` being empty on the text-labelled Restore button inside a closed `<details>`; Axe exposes the button name correctly.

The 200% resize failure remains as reported above.

### Privacy, security, links, and rate limits

- Fresh demo load → evidence change → export → reset made same-origin requests only. No analytics, third-party runtime script, or font request appeared.
- The explicit license-restore action contacted only `https://api.sociobot.in`; its response allowed the exact product origin and used `Cache-Control: no-store`.
- Live HTML sends CSP as a response header with `frame-ancestors 'none'`, plus HSTS, `nosniff`, and strict-origin referrer policy. HTML is `no-cache`; hashed assets are one-year immutable.
- All internal navigation links returned their expected statuses; the designed unknown route returns HTTP 404 and offers a path home. The checkout link is explicitly external.
- Product API burst: 100 concurrent HTTP/2 requests from one client produced 40×401 then 60×429; all 60 throttled responses included `Retry-After: 1`. Observed allowance: 40 requests per client per one-second window.
- Sociobot product verification burst: 120 concurrent requests produced 30×200 then 90×429; all 90 throttled responses included `Retry-After: 4`. Observed allowance: 30 requests per client per roughly five-second window.

### Deployment parity and performance

- `/health` returns `cb9e3cd7381cdc618fb0a3d4c8baf843d2748143`.
- Live/local JS SHA-256: `21dd4f80f06d79c60e3e2e9422c560f6cceb227b1ae8ab7b88483e93f908af1f`.
- Live/local CSS SHA-256: `63f5f23c91be14fa3b0b041b62c93a56a106c23a2c2ede962320f58394bb76d1`.
- Lighthouse 12.8.2 mobile: performance 99, accessibility 100, best practices 100, SEO 100; LCP 1.65 s, CLS 0, TBT 70 ms.

Evidence: `verification-artifacts-7/live-health.json`, response-header files, `live-rate-combined.headers`, `billing-rate-combined.headers`, and `lighthouse-live-home.json`.

## Required before release

1. Deploy with `PUBLIC_BASE_URL=https://agent-diff-gate.sociobot.in`, the approved Entra settings, a durable `/data` volume, and exactly one replica; then make the live-identity regression a deployment gate that cannot be bypassed.
2. Complete a real production account flow through sign-in, team GitHub App creation/installation, policy save, private PR import, evidence save, named-owner approval, audit export, retention, and deletion.
3. Make the mobile header reflow at 200% text size.
4. Show invalid/revoked license feedback and remove or replace invalid cached tokens.
5. Register every public README/page claim in `.factory/claims.json` with one exact tagged test, or remove the extra claims.
