# Independent verification 2 — FAIL

**Candidate:** `d8793a8aea82604da0ffda9b599a0feba35ef505`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-28 from a clean candidate checkout

## Release decision

**FAIL.** The candidate is deployed, the sample is useful, and the local quality gates pass. The real job-to-be-done does not. The live service has no working sign-in or GitHub App configuration, the implementation uses GitHub OAuth instead of the required Sociobot Microsoft Entra External ID tenant, and the server will approve a packet with missing evidence or approve a packet as someone other than its named owner. These are release-blocking failures in an approval product.

This is fresh evidence. The earlier deployment mismatch/failure is resolved: `/health` reports the full candidate SHA and live JS/CSS hashes exactly match the candidate build.

## Mandatory first checks

### Claims

The exact commands from `.factory/claims.json` were invoked first. Before dependency installation, both browser commands could not start because the clean checkout had no `@playwright/test`; after the required `npm ci`, every exact claim command passed. The Rust claim passed on its first invocation.

| Claim | Exact command | Result |
|---|---|---|
| `sample-sandbox` | `npm run test:browser -- --grep @claim:sample-sandbox` | PASS after install: 1/1; `/demo` showed the complete sample and only same-origin requests. |
| `packet-export` | `npm run test:browser -- --grep @claim:packet-export` | PASS after install: 1/1; downloaded `diff-gate-packet.json`. |
| `team-packet-boundary` | `cargo test packet_reads_and_approvals_are_scoped_to_the_signed_in_team` | PASS: cross-team reads and approvals returned 404; the owning seeded session created an audit row. |

### Cold first read

PASS. A cold 1440×900 visit says **“Review agent changes before merge”**, names **small software teams** that need an owner and evidence, and gives **“Try it with sample data”** with **“Opens a complete review packet.”** One click opens `/demo`, a realistic packet, and the persistent sample-data banner. Evidence: `verification-artifacts/live-first-read-desktop.png`.

## Release-blocking defects

### Critical — the live product cannot perform its real job

Fresh live responses show:

- `GET /api/auth/status` → `200` with `github_sign_in_configured:false` and `github_app_configured:false`.
- `GET /auth/github` → `503` with `{"error":"GitHub sign-in is not configured on this deployment."}`.
- The first real-work panel displays the same unavailable message. There is no way to create, import, review, revisit, or approve a real packet on the deployed product.

The demo works, but Definition of Done #1 explicitly requires the real PR-review workflow rather than a demo.

### Critical — sign-in violates the required identity contract

The product requires an account but implements GitHub OAuth at `/auth/github`, requests `read:user read:org`, and creates its own cookie session. It does not use `sociobotcustomers.ciamlogin.com`, and the Entra authority does not appear anywhere in the candidate. The acceptance contract requires Sociobot Microsoft Entra External ID and nothing else when sign-in is required.

### Critical — approval does not enforce evidence or the named owner

Using the release binary with a disposable SQLite database and a seeded authenticated team session:

1. A packet was created with stored check state `missing`.
2. `POST /api/packets/<id>/approve` returned `200` and changed it to `approved` while the returned stored data still said `missing`.
3. After process restart, that invalid approved state remained durable.
4. A second packet named `release-manager` as owner. Session user `qa-reviewer` approved it successfully; the response recorded `approved_by:"qa-reviewer"`.

The cause is visible in `backend/src/main.rs`: `approve_packet` checks only team membership and never validates stored checks or compares the session login with `packet.owner`. In `frontend/src/main.ts`, “Mark reviewed” mutates only the browser draft; `saveDemo()` intentionally does nothing for real packets. The resolved evidence is therefore never persisted before approval.

The demo has the same truthfulness problem: after both checks are resolved, “Approve for merge” shows **“Approval recorded”**, but no approval field or status is stored or rendered. A reload contains only the changed check states.

### High — GitHub import can miss risky files and is not bound to the signed-in team

- Import fetches only `/files?per_page=100` and does not paginate. Contract or migration files after the first 100 silently escape policy classification, a serious boundary failure for the large agent diffs this product targets.
- One deployment-wide `GITHUB_APP_INSTALLATION_ID` supplies repository access, while the packet tenant is chosen from the first active organization returned by GitHub OAuth. There is no check that the installation belongs to that selected team. Identity, repository authorization, and packet tenancy can diverge.
- There is no UI to list or reopen the backend's saved packets or audit entries after reload, so durable review history is not usable even when the endpoints contain data.

### High — visitor-facing claims are missing from `claims.json`

The claims file covers browser-local sample data, JSON export, and packet team isolation only. Unlisted reliance-worthy claims include:

- “GitHub sign-in identifies real reviewers.”
- “GitHub App imports use installed repository access.”
- “Every change gets an owner, evidence, and a clear review state.”
- “The installed GitHub App reads changed paths.”
- “Resolve evidence and retain the named approval.”
- README claims that the app imports pull requests and records changed paths, checks, evidence, and approval.
- README's no-analytics/no-third-party-runtime claim and privacy-page GitHub-use claim.

The live request audit supports the current no-third-party-runtime statement, but the required claim-to-test registration is absent. The claims contract makes an unlisted claim release-blocking.

### High — dark mode has serious contrast failures

Playwright Axe with `colorScheme: dark` found `color-contrast` serious violations: 9 nodes on `/` and 8 on `/demo`. Examples:

- Light text `#ecf0e9` on yellow `#f7c948` in the demo banner/footer: **1.35:1**, required 4.5:1.
- Warning text `#8a2f21` on dark surface `#17212b`: **1.94:1**, required 4.5:1.

Light mode had zero serious/critical Axe findings.

## Other defects

### Medium — Demo routing and history are inconsistent

Clicking the header **Demo** link from `/` changes the URL to `/demo` but shows the real sign-in panel: no demo banner and no sample packet. Starting the sample, choosing **Start for real**, and then pressing Back produces the same broken `/demo` state. A direct cold load of `/demo` works. The SPA keeps a stale `demo` boolean instead of deriving mode from the current route.

### Medium — mobile touch targets miss the 44px baseline

At 390px the page has no horizontal overflow, but visible footer links measured 21px high; `Terms` measured 38×21px and the external factory link 141×21px. The header `Demo` link is only 37px wide. These miss the required 44×44 CSS-pixel target.

### Medium — the footer contains a dead external link

`https://paramfactory.com/` failed DNS resolution during the link crawl. The link's visible label also does not tell users it opens an external site.

### Medium — users cannot sign out

The backend exposes `POST /api/auth/signout`, but no interface invokes it. A shared-browser user cannot end the 14-day session through the product.

## What passed

- Candidate identity: live `/health` returned `{"status":"ok","build":"d8793a8aea82604da0ffda9b599a0feba35ef505"}`.
- Candidate parity: live JS and CSS SHA-256 hashes exactly matched local `dist` files.
- Clean install: `npm ci`, 0 reported vulnerabilities.
- Frontend: `npx tsc --noEmit`, `npm test` (7/7 browser tests), and `npm run build` passed. `dist/` was produced.
- Backend: `cargo fmt --check`, `cargo test` (5/5), `cargo clippy -- -D warnings`, and `cargo build --release` passed.
- Startup: the release binary started with only `PORT`; it logged generated default database configuration and served `/health`.
- Input boundaries: empty and 181-character titles returned 400 with a useful error.
- Persistence: packet and approval state survived a release-process restart against disposable SQLite.
- Team isolation: unauthenticated packet access returned 401; the claim regression proves cross-team read/approval isolation.
- Live rate limit: a 200-request concurrent burst from one forwarded client produced exactly 40×401 then 160×429; every 429 had `Retry-After: 1`. Observed allowance: **40 requests per one-second window per client**. `/health` is exempt.
- Privacy: the complete landing/demo/export/reset browser exercise contacted only `https://agent-diff-gate.sociobot.in`; no analytics, CDN font, or third-party script request occurred.
- Headers: CSP, `X-Content-Type-Options: nosniff`, and `Referrer-Policy: strict-origin-when-cross-origin` were present. The CSP is delivered as a response header.
- Caching/routes: hashed JS/CSS and WebP use `public, max-age=31536000, immutable`; HTML uses `no-cache`; unknown paths return a real HTTP 404.
- Accessibility, light mode: `/`, `/demo`, `/privacy`, `/terms`, and 404 had one h1, one main, title/lang, complete image alt text, and zero serious/critical Axe findings. Keyboard-only review worked, focus was a visible 3px coral outline, reduced motion produced no animation, and 200% root text size caused no horizontal overflow.
- Performance: mobile Lighthouse scored 100 performance, 100 accessibility, 100 best practices, and 100 SEO; FCP 0.9s, LCP 1.7s, TBT 20ms, CLS 0. Evidence: `verification-artifacts/lighthouse-home.json`.
- Budgets: JS 14,903 bytes (5.40KB gzip), CSS 10,337 bytes (3.26KB gzip), no web fonts, hero WebP 136,640 bytes.
- Visual system: the dithered change-control desk art and editorial packet layout are product-specific and documented with provenance.

`verify-url.sh` passed `/` and `/demo` with zero console/page errors. The only console error seen in the combined route sweep came from intentionally requesting the HTTP 404 route.

## Coverage notes

- Docker could not be rebuilt because this verifier image has no Docker client/daemon. The Dockerfile contract is covered by a passing backend regression, both native production builds passed, and the exact candidate is running live.
- Library/CLI consumer tests and service-worker update/offline-reload tests are not applicable. This is a web-with-backend product and does not register a service worker or claim PWA offline reload.

## Required fixes before another candidate

1. Configure and exercise the real production workflow, and use Sociobot Entra External ID for product identity.
2. Enforce unresolved-check and named-owner rules in the backend; persist each evidence resolution; expose packet/audit history; make approval immutable or explicitly versioned.
3. Bind each GitHub App installation to the authenticated team and paginate all changed files.
4. Register executable claim tests for every public promise or remove the promise.
5. Fix dark-theme contrast, demo routing/back behavior, touch targets, sign-out, and the dead footer link.
