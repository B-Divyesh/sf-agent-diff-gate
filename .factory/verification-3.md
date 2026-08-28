# Independent verification 3 — FAIL

**Candidate:** `9fb9afa9361a2ff234885b49e35bb3874550156f`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-28 from a clean candidate checkout

## Release decision

**FAIL.** The exact candidate is deployed, every registered claim test passes after the documented clean install, and the sample is fast, understandable, private, and usable on desktop and mobile. The deployed product still cannot perform its real job: Sociobot Entra and the team-bound GitHub App are both unconfigured, so nobody can sign in or import, save, revisit, or approve a real pull request. Fresh evidence also found serious dark-mode contrast failures, no configurable retention, an incorrect privacy promise about discarded demo state, unregistered public claims, and a misleading response during concurrent approval.

This is fresh evidence rather than a carry-forward of the builder's deployment report. The deployment now identifies itself as the full candidate SHA and its JS, CSS, and images byte-match the candidate build.

## Mandatory first checks

### Claims gate

`.factory/claims.json` exists. Its exact commands were attempted before other QA; the browser commands initially could not load `@playwright/test` because dependencies had not yet been installed in the clean checkout. After the documented prerequisite `npm ci`, every claim command passed. The initial missing-dependency condition is not treated as a product failure because the lockfile install is required by the README and completed without error.

| Claim | Exact command | Result |
|---|---|---|
| `sample-sandbox` | `npm run test:browser -- --grep @claim:sample-sandbox` | PASS, 1/1. `/demo` opened the complete sample, used `demo:diff-gate`, and made only same-origin requests. |
| `packet-export` | `npm run test:browser -- --grep @claim:packet-export` | PASS, 1/1. Live follow-up parsed the downloaded JSON and confirmed the sample title and approved state. |
| `team-packet-boundary` | `cargo test packet_reads_and_approvals_are_scoped_to_the_signed_in_team` | PASS, 1/1. |
| `named-approval` | `cargo test approval_rejects_missing_evidence_and_wrong_owner_and_persists_saved_evidence` | PASS, 1/1. |
| `entra-team-installation` | `cargo test entra_and_github_installations_are_configured_per_team` | PASS, 1/1. |
| `no-third-party-runtime` | `npm run test:browser -- --grep @claim:sample-sandbox` | PASS, 1/1. Live full demo flow also recorded zero off-origin requests. |

### Cold first read

**PASS.** At 1440×900 the first screen says **“Review agent changes before merge,”** names **small software teams** that need an owner and evidence, and presents **“Try it with sample data”** beside **“Opens a complete review packet.”** One click opens a realistic packet and a persistent **“Demo — sample data, nothing is saved”** banner. The same information and action are visible at 390×844.

## Release-blocking findings

### Critical — the deployed product cannot do the real job

Fresh live responses:

- `GET /health` → `200 {"status":"ok","build":"9fb9afa9361a2ff234885b49e35bb3874550156f"}`.
- `GET /api/auth/status` → `200` with `entra_sign_in_configured:false`, `github_app_configured:false`, and no installation URL.
- `GET /auth/entra` → `503 {"error":"Sociobot Entra sign-in is not configured on this deployment."}`.

The live first real-work panel shows the same unavailable message and offers no sign-in control. Therefore a team cannot authenticate, import a GitHub pull request, create a real packet, save evidence, reopen it, or record a real owner approval. A working sample does not satisfy the brief's smallest useful web-with-backend product.

The implementation and README use the required `sociobotcustomers.ciamlogin.com` authority in examples, but the running deployment cannot exercise it. In addition, `ENTRA_AUTHORITY` accepts any configured host; the code does not restrict identity to the required Sociobot tenant, so the “and nothing else” identity requirement is not enforced.

### High — dark-mode landing content fails contrast

Live Playwright Axe at 390px with `prefers-color-scheme: dark` reports one serious `color-contrast` violation affecting three nodes:

- **Try it with sample data:** white `#fff` on cyan `#47c5d0`, **2.06:1**; required 4.5:1.
- **It does not merge code for you** heading: `#111923` on `#080d12`, **1.1:1**; required 3:1 for this large text.
- Its supporting paragraph uses the same **1.1:1** combination; required 4.5:1.

The repository's dark Axe test covers `/demo` only, so `npm test` does not catch this landing-page regression. Dark `/demo`, `/privacy`, and `/terms` had no serious/critical Axe findings.

### High — configurable retention from the researched constraints is absent

Real packet, session, OAuth-state, and audit rows are stored indefinitely. There is no retention setting, expiry job, packet deletion endpoint, or user-facing deletion control. The privacy page does not say how long real data is kept. This violates the brief's explicit configurable-retention constraint for repository review data.

### High — public claims are incomplete and one privacy claim is false

The claims contract makes unlisted reliance-worthy statements release-blocking. Examples without a matching observable claim test include **“Team-bound GitHub App imports changed paths,”** **“The team-bound GitHub App reads every changed path,”** the README's pagination/10,000-file safety statement, and the privacy page's statement that GitHub is used only for team-authorized pull requests. The registered Entra/configuration unit test verifies configuration objects and a mapping lookup; it does not exercise a recorded GitHub import fixture or prove all changed paths are classified.

The privacy page also says **“Demo data stays in this browser and is discarded when you leave demo mode.”** Live reproduction disproves the second half: resolve one sample check, follow the header Privacy link, then return to `/demo`; `sessionStorage['demo:diff-gate']` remains and the modified one-check state is restored. Only **Start for real** explicitly clears the key. No claim entry tests leaving demo through ordinary navigation.

## Other findings

### Medium — concurrent approval returns a false not-found error

Against the release binary and a disposable SQLite database, two simultaneous valid owner approvals returned `200` and `404`. The second body said **“That review packet was not found in this team.”** The packet remained correctly approved and only one update won, but the losing request should report the existing immutable approval (`409`), not claim the packet disappeared. This is a realistic two-tab/two-request recovery path for an approval tool.

### Medium — the researched paid tier and usable audit history are absent

The brief specifies `$12/developer/month` or `$99/team/month` through Sociobot billing. The candidate has no pricing, checkout, subscription state, or paid tier. The backend writes audit rows but exposes no route or interface to view/export the audit trail; users can only see the final approval fields on a packet.

### Medium — immutable caching is applied to unhashed images

Hashed JS/CSS correctly return `Cache-Control: public, max-age=31536000, immutable`, but the same policy is applied to stable URLs such as `/change-control.webp`, `/social.webp`, and `/apple-touch-icon.png`. A changed image at a later deployment can remain stale for a year. Only content-addressed/hashed assets should be immutable.

### Low — supporting metadata and verification documents are stale/incomplete

- `sitemap.xml` uses relative `<loc>` values rather than the required absolute URLs.
- `.factory/copy-audit.md` still contains the prior GitHub-sign-in wording and does not audit the candidate's current Sociobot copy.
- Responses include CSP, `nosniff`, and a referrer policy, but no `Strict-Transport-Security` header was observed.

## What passed

- Clean install: `npm ci` passed with 0 reported vulnerabilities.
- Frontend gates: `npx tsc --noEmit`, `npm test` (11/11 Playwright tests; Vitest has no test files and is explicitly configured to pass), and `npm run build` passed; `dist/` was produced.
- Backend gates: `cargo fmt --check`, `cargo test` (7/7), `cargo clippy -- -D warnings`, and `cargo build --release` passed.
- Runtime contract: the release binary started with only `PORT` in a clean environment, logged generated default database configuration, served static content, and returned `/health` build `dev`.
- Backend boundaries: unauthenticated packet access returned 401; empty and 181-character titles returned 400; missing checks could not be approved; malformed evidence returned 400; saved evidence enabled named-owner approval; approved state survived process restart.
- Concurrency: 100 concurrent local health requests all returned 200. The concurrent write issue is documented above.
- Live rate limit: a 55-request burst to `/api/auth/status` produced 40×200 and 15×429 in the observed one-second window; every 429 had `Retry-After: 1`. Observed allowance: **40 requests per client per second**. `/health` is exempt.
- Deployment parity: `/health` reports the candidate SHA. SHA-256 values for live JS, CSS, hero WebP, and social WebP exactly match local `dist`.
- Privacy request log: the complete home → demo → resolve → approve → export flow made five requests, all to the product origin (document, JS, CSS, `/api/auth/status`, hero image). There were no analytics, third-party scripts/fonts, console errors, or page errors.
- Accessibility outside the dark landing defect: one `<h1>`, one `<main>`, `lang`, route titles, alt text, labels, 3px visible coral keyboard focus, no keyboard trap, 44px visible controls, reduced-motion suppression, and no normal-width horizontal overflow at 390px. At 200% text sizing there was no visible clipped content.
- Routes and links: `/`, `/demo`, `/privacy`, and `/terms` return 200; an unknown route returns a styled HTTP 404; all internal links resolve.
- Headers: CSP is a response header and matches observed resources; `X-Content-Type-Options: nosniff` and `Referrer-Policy: strict-origin-when-cross-origin` are present.
- Performance: mobile Lighthouse scored **98 performance / 100 accessibility** with FCP 1.0s, LCP 1.7s, TBT 150ms, CLS 0, and 162 KiB total transfer. Initial JS is 16,424 bytes (5,880 bytes gzip), CSS 10,591 bytes (3,324 bytes gzip), hero image 136,640 bytes, and no web fonts load.
- `/opt/fleet/lib/verify-url.sh` passed `/` and `/demo`: title/lang/main/alt checks succeeded with zero console/page errors.
- Visual identity, original artwork provenance, legal routes, README, and MIT license are present.

## Coverage notes

- The exact Docker build could not be executed because this verifier container has no `docker` command. The native Vite and optimized Rust production builds passed, the Dockerfile uses the required unpinned `rust:1-alpine` stage and non-root runtime, and the exact candidate is running live.
- Library/CLI consumer checks and service-worker update checks are not applicable. This is a web-with-backend product and does not register a service worker or claim PWA offline reload.

## Required before release

1. Provision the live Sociobot Entra application and team-bound GitHub App configuration, restrict the authority to `sociobotcustomers.ciamlogin.com`, and exercise a real mapped-team PR end to end.
2. Fix all dark landing contrast failures and add live/home dark Axe coverage.
3. Implement configurable retention/deletion and document it on `/privacy`.
4. Correct the demo discard behavior and register observable tests for every public GitHub/import/privacy claim.
5. Return a truthful conflict response for concurrent approval; expose the audit trail.
6. Implement the brief's Sociobot-billed subscription tier or record an approved scope change.
