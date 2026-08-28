# Independent verification — FAIL

**Candidate:** `c185dbf7fd0ea475761eef0c011294252fe12950`  
**Live URL:** https://agent-diff-gate.sociobot.in  
**Verified:** 2026-08-28 (fresh checkout at the candidate commit)

## Release decision

**FAIL.** The visual demo, metadata, basic accessibility, and local builds are solid, and the live server identifies itself as the candidate commit. However, this is not an end-to-end implementation of the researched job: it cannot connect to or ingest a GitHub PR, has no team/auth boundary, and its "approval" is an in-memory alert. The unauthenticated packet API would also disclose every saved packet. The deployed rate limit did not enforce its documented allowance.

## Mandatory first checks

### Claims from `.factory/claims.json`

Both required claim commands were run first, after `npm ci`, against the product's Playwright demo entry point:

| Claim | Command | Result |
|---|---|---|
| `sample-sandbox` — sample data stays in this browser | `npm run test:browser -- --grep @claim:sample-sandbox` | PASS: 1/1; `/demo` loads the complete packet and the test recorded same-origin requests. |
| `packet-export` — exports the review packet as JSON | `npm run test:browser -- --grep @claim:packet-export` | PASS: 1/1; download filename was `diff-gate-packet.json`. |

### Cold first read of the live site

PASS. A fresh browser visit showed: **“Review agent changes before merge”**, then **“For small software teams who need an owner and evidence before an agent-made change lands.”** The first screen contains **“Try it with sample data”** and immediately says **“Opens a complete review packet.”** It answers what it does, for whom, and what to click first in plain words. The click navigates to `/demo` in one action.

## What passed

- Clean install: `npm ci` completed with 0 reported vulnerabilities.
- Frontend quality: `npx tsc --noEmit`, `npm test`, and `npm run build` passed. `npm test` ran all 7 browser tests, including keyboard resolution, demo reset/isolation, offline review, 390px overflow, and axe.
- Rust quality: `cargo fmt --check`, `cargo test` (3/3), `cargo clippy -- -D warnings`, and `cargo build --release` passed. The production binary was produced.
- Runtime smoke: the release binary started with only `PORT=18080` plus a clean process environment. It generated its default SQLite configuration, returned `{"status":"ok","build":"dev"}`, and locally enforced 40 requests/second followed by `429 Too Many Requests` with `retry-after: 1`.
- Local persistence smoke: a packet POSTed to a disposable SQLite database was listed successfully and remained after restart.
- Deployment identity: live `/health` returned `{"status":"ok","build":"c185dbf7fd0ea475761eef0c011294252fe12950"}`. SHA-256 checks of the live JS/CSS exactly matched the candidate `dist` assets.
- Live routes `/`, `/demo`, `/privacy`, `/terms`, metadata, canonical/title updates, demo reset, JSON export, and the desktop/390px layouts worked. There were no page errors or console errors.
- Live privacy request log: cold home/demo/privacy/terms visits made only same-origin requests (document, JS, CSS, and self-hosted WebP); no analytics, CDN fonts, or third-party scripts were observed.
- Accessibility: live axe scans on `/`, `/demo`, `/privacy`, `/terms`, and the client 404 had zero serious/critical findings. At 390px there was no horizontal overflow. Keyboard operation resolved both flagged checks and enabled approval; visible focus was a 3px coral outline. Reduced-motion emulation had no active animations/transitions.
- Static bundle: JS 12.35 KB (4.69 KB gzip), CSS 10.34 KB (3.26 KB gzip), hero image 136.64 KB.

## Release-blocking defects

### Critical — the promised product workflow is absent

The brief calls for a least-privilege **GitHub app** that turns an agent-authored PR into a policy-tagged packet (contracts, migrations, tests, risky paths, owner sign-off). The candidate only ships a fixed sample packet and a client-side blank card. There is no GitHub App/OAuth flow, PR import, GitHub API integration, policy configuration, owner identity, team boundary, or retained approval history.

Fresh live browser evidence: clicking **Create a blank packet** creates a fixed local draft with no editable title/owner/evidence fields: `Untitled change`, `Add responsible owner`, and two missing checks. Those missing checks have no control to resolve, so **Approve for merge** stays disabled. The browser recorded no `/api/` request during this flow. The frontend contains no `fetch` or API call for real packet creation/approval, so no approval can be recorded. This fails the brief's smallest useful product and Definition of Done #1.

### Critical — saved review packets have no authentication or tenant isolation

`GET https://agent-diff-gate.sociobot.in/api/packets` returned `200 []` unauthenticated, and an unauthenticated invalid POST returned the application's `400` validation response. The server routes `GET/POST /api/packets` without any authentication or tenant predicate; its SQL list query returns all packets. If any team saved data, any internet user could list it and retrieve it by ID. This violates the product's accountable review/audit purpose and its privacy promises.

### High — live rate limiting is not enforced

The documented/server-tested allowance is 40 requests/second per forwarded IP. The deployed server accepted **100 concurrent** unauthenticated requests from this verifier to `/api/packets`, all with `200`; no `429` or `Retry-After` was observed. Locally, the same 41-request test gave 40×200 then 1×429 with `retry-after: 1`. The deployment therefore fails the mandatory backend rate-limit verification even though the local unit test passes.

### High — paid access is not implemented according to the required Sociobot unlock contract

The page offers a $99/month team plan and stores a pasted `sb_license:agent-diff-gate` token, but it never calls the required Sociobot `/verify?license=` endpoint, has no daily cached verdict, no invalid/revoked handling, and does not actually gate or unlock a product capability. The UI claim “Connect GitHub review packets, retained audit history, and owner rules” is especially misleading because those features do not exist.

### High — Dockerfile violates the required build-image contract

The build stage is `FROM rust:1.88-alpine`. The backend-service contract explicitly requires `rust:1-slim` or `rust:1-alpine` and says never pin a minor release. This is a deployment-contract failure despite the local Rust release build passing. A local Docker engine was unavailable in this verifier container, so the image itself was not rebuilt here.

### Medium — public claims are not all represented in `claims.json`

The claims file covers only demo browser locality and JSON export. Live landing copy additionally makes reliance-worthy claims including “Repository code is not used to train this product,” “No GitHub install needed to try it,” advisory security treatment, pricing/checkout, and functionality promised by the paid plan. No observable sandbox test is listed for these claims, contrary to the claims contract.

### Medium — cache/404 deployment gaps

The hashed JS/CSS and WebP responses have no `Cache-Control` header, rather than long-lived immutable caching. Also, a nonexistent URL returns HTTP 200 with a client-side 404 screen; the site-structure contract calls for a real 404 response/route.

## Required remediation before a re-verification

1. Implement an authenticated, least-privilege GitHub App PR ingestion flow and real packet creation/editing, policy/risk evaluation, evidence capture, named owner sign-off, and durable audit history.
2. Add Sociobot Entra External ID authentication and tenant-scoped authorization. Do not expose packet list/get/create routes without it.
3. Fix deployed per-client rate limiting and prove a single client receives `429` and `Retry-After` past the allowance in the live environment.
4. Implement the required Sociobot license verification/restore/expiry lifecycle or remove the paid-plan UI until it exists.
5. Use the required unpinned Rust major Docker image, add cache headers and a real HTTP 404, and add tests for every visitor-facing claim (or remove it).
