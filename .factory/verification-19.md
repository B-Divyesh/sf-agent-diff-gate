# Verification 19 — FAIL

**Candidate:** `9df61fc1e555984da087af7596c9a8b397897492`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC
**Verifier:** independent clean-checkout QA

## Release decision

**FAIL — release blocking.** The deployed frontend and backend identify as the candidate, but the production backend is deliberately fail-closed because its durable single-replica configuration is absent. Consequently no real team can sign in, create/import/review a packet, or use the GitHub App workflow. The live rate limit is also multiplied across replicas, so its documented 40-request single-client allowance is not enforced globally.

## Mandatory first gates

### Claims gate — PASS locally

`.factory/claims.json` exists and contains 20 claims. From this clean checkout, after `npm ci` and `cargo fetch`, every exact listed command exited zero. The seven browser claims were run individually through the Vite demo entry point:

| Claim IDs | Result |
| --- | --- |
| `sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action`, `audit-export`, `no-third-party-runtime` | PASS (one Playwright test each) |
| `team-packet-boundary`, `named-approval`, `entra-team-installation`, `github-complete-import`, `github-revision-refresh`, `github-app-provisioning`, `repository-policy`, `retention-deletion`, `audit-history`, `github-file-limit`, `retention-limits-and-cleanup`, `durable-store-replacement` | PASS (one named Cargo test each) |
| `runtime-port-health` | PASS (`./scripts/verify-runtime-contract.sh`; clean `PORT`-only release startup) |

### Cold first read — PASS

A fresh desktop browser opening `/` said: “Review agent-authored changes before merge,” for “small software teams” needing “a required owner and test evidence,” and presented **Try it with sample data** first. Its adjacent explanation says it opens a packet with changed files, test evidence, and owner checks. This is plain, specific, and the action is one click. At 390×844 the action measured `x=20, y=542.2, 207.6×46.3`, fully within the first screen.

The first read also exposed the failure: the real-work panel says “Team workspace is temporarily unavailable.”

## Release-blocking defects

### Critical — real product workflow unavailable in production

Fresh live evidence at 2026-08-29 21:58 UTC:

```text
GET /health                  → 503
{"status":"unsafe_configuration","build":"9df61fc1e555984da087af7596c9a8b397897492", ...}

GET /api/auth/status         → 200
{"service_ready":false,"authenticated":false,
 "entra_sign_in_configured":false,"github_app_setup_available":false, ...}

GET /auth/entra              → 503
GET /api/packets             → 503
```

`scripts/verify-live-identity.sh` was run against the URL and could not satisfy its required Sociobot Entra readiness condition. The live UI accurately exposes this condition rather than producing a console error, but it does not make the smallest useful product work end to end. It violates the acceptance contract's required Sociobot Entra sign-in, scoped team packets, GitHub App setup/import, durable persistence, and health readiness.

### Critical — rate limit is not enforced per single client across live service

The documented allowance is 40 requests per client per second (`deploy/live-rate-limit.mjs`). A fresh single-client 100-request live probe to `/api/auth/status` observed:

```json
{"200":80,"429":20}
```

All 20 throttled responses did contain `Retry-After: 1`, but requests 41–80 were wrongly accepted. The provided live probe independently reported “accepted 79 requests; expected exactly 40.” This is consistent with two independent replicas/limit windows and is release-blocking for the backend-service contract.

## Candidate/deployment identity

The deployed hashed assets exactly match this checkout's production output:

```text
aff40057e8e6dbf60c104628ab39f89ca302f19fd6d23ccd8b755defefd26487  dist/assets/index-r2CfzYxf.js  (also fetched live)
f17d5315eaddad9a13022bb9a75d3a64898058ce08639db68f6651b7234c2c65  dist/assets/index-rQ7R4Jb-.css (also fetched live)
```

`/health` also reports build `9df61fc1e555984da087af7596c9a8b397897492`. The defects are therefore fresh evidence against the candidate's actual deployment, not a stale frontend deployment.

## What passed

- Full frontend test suite: `npm test` — 8 Node tests + 25 Playwright tests passed.
- Backend suite: `cargo test --all` — 21 passed.
- Type/quality checks: `npx tsc --noEmit`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets --all-features -- -D warnings` passed.
- Production builds: `npm run build` passed (JS 22.86 kB / 7.28 kB gzip; CSS 12.23 kB / 3.62 kB gzip) and `cargo build --release` passed. Docker is not installed in this verifier container, so a local image build could not run; this does not alter the FAIL decision.
- Live demo: `/demo` loaded a realistic packet, reset/re-entry/export/approval behavior passed locally via the required claim tests; an independent live 390px run resolved both checks and retained the demo approval. Demo traffic was same-origin only, with no console or page errors.
- Live accessibility smoke: light and dark 390×844 demo scans with reduced motion had zero axe serious/critical findings, no horizontal overflow, and a visible 3px focus outline. Desktop cold load also had no console/page errors.
- Live response policy: HTML/API uses `no-cache`; the hashed JS has `public, max-age=31536000, immutable`; image cache is one hour `must-revalidate`; HSTS, `nosniff`, strict-origin referrer policy, CSP with `frame-ancestors 'none'`, and designed 404/noindex headers are present.
- Privacy: recorded cold/demo flows requested only `https://agent-diff-gate.sociobot.in` (document, local assets, `/api/auth/status`, and artwork). No third-party runtime or analytics request was observed.

## Required remediation and re-verification

Deploy the candidate only with the documented stateful topology: exactly one replica, one durable Azure Files `/data` mount, the durable SQLite URL, and all production Entra/GitHub configuration. Then re-run `scripts/verify-live-deployment.sh` and `scripts/verify-live-identity.sh`; they must show `/health` 200 with this build, `service_ready:true`, Sociobot Entra PKCE redirect, one unchanged storage ID across replacement, and exactly 40 accepted/60 HTTP 429 responses with `Retry-After: 1` from one client. Finally repeat a signed-in team packet and GitHub App import flow live.
