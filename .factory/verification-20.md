# Verification 20 — FAIL

**Candidate:** `a1eaeea89db9be13f74d8ec5ff137e104b753551`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC
**Method:** independent clean-checkout QA; no product code changed

## Release decision

**FAIL — release blocking.** The live static frontend is demonstrably this
candidate, but the deployed backend has deliberately failed closed. It returns
`503 unsafe_configuration`, has no configured Sociobot Entra sign-in, and
rejects `/auth/entra` with 503. A real team therefore cannot sign in, connect
its GitHub App, create/import a review packet, record evidence, or approve it.
That misses the researched brief's smallest useful product, despite a working
local/demo experience.

## Mandatory first gates

### Claims — PASS locally

`.factory/claims.json` exists and contains 20 claims. From the clean candidate,
after `npm ci`, every listed command passed using the product's demo entry
point where applicable:

| Claim group | Evidence |
| --- | --- |
| Browser demo claims: `sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action`, `audit-export`, `no-third-party-runtime` | Each exact `npm run test:browser -- --grep @claim:<id>` command passed; a combined independent repeat was **7 passed**. |
| Backend claims: team boundary, named approval, Entra/team installation, GitHub pagination/revision/provisioning, policy, retention/deletion, concurrent audit, file limit, retention limits, durable-store replacement | Each of the twelve exact named `cargo test <name>` commands passed. |
| `runtime-port-health` | `./scripts/verify-runtime-contract.sh` passed: a release binary started with only `PATH`, `PORT`, and `BUILD_SHA`, and `/health` returned the supplied build and a durable-store identity. |

### Cold first read — PASS

A fresh desktop visit states **“Review agent-authored changes before merge”**,
then says it is for small software teams that need a required owner and test
evidence before an agent-authored change lands. Its first action is **“Try it
with sample data”** and the adjacent copy says it opens a packet with changed
files, test evidence, and owner checks. This answers what it does, for whom,
and what to click in plain words.

At 390×844 the complete action measured `x=20`, `y=542.2`, `207.6×46.3` CSS
pixels: it is visible and operable in the first screen. The sample opens in one
click with the persistent “Demo — sample data, nothing is saved” banner,
reset, and real-work exit controls.

## Release-blocking defect

### Critical — deployed real-team workflow is unavailable

Fresh live evidence:

```text
GET /health → 503
{"status":"unsafe_configuration",
 "build":"a1eaeea89db9be13f74d8ec5ff137e104b753551",
 "storage_id":"1dd80a94-842c-490f-8fed-617ba8b62116"}

GET /api/auth/status → 200
{"service_ready":false,"authenticated":false,
 "entra_sign_in_configured":false,"github_app_setup_available":false,
 "github_app_configured":false,...}

GET /auth/entra → 503
{"error":"Diff Gate is waiting for its durable production storage configuration. Try again shortly."}
```

The landing page accurately exposes this as “Team workspace is temporarily
unavailable,” but the contract requires the real end-to-end job, not only the
sample. The service has no usable Sociobot Entra authority on the deployed
instance, so the required Entra-only sign-in and the team-scoped GitHub review
workflow cannot be exercised or accepted.

## Candidate/deployment identity

The backend health response names the tested commit. The two deployed hashed
assets also byte-match this checkout after its production build:

```text
aff40057e8e6dbf60c104628ab39f89ca302f19fd6d23ccd8b755defefd26487  index-r2CfzYxf.js
f17d5315eaddad9a13022bb9a75d3a64898058ce08639db68f6651b7234c2c65  index-rQ7R4Jb-.css
```

This is fresh evidence against the candidate's actual deployment, not a stale
frontend or a different commit.

## What passed

- `npm test`: **9 Node unit tests + 25 Playwright tests passed**.
- `npx tsc --noEmit`, `npm run build`, `cargo fmt --check`, `cargo test` (21
  backend tests), and `cargo clippy -- -D warnings` all passed.
- Production frontend build: JS **22.86 kB / 7.28 kB gzip**; CSS **12.23 kB /
  3.62 kB gzip** — within the stated static bundle budget. The release binary
  was built by the runtime-contract check.
- Live desktop plus 390px mobile: no console/page errors, one `h1`, `main`,
  `lang=en`, correct route titles, no mobile horizontal overflow, visible
  keyboard focus, and reduced motion reported no running hero animation.
- Axe scans of `/`, `/demo`, `/privacy`, and `/terms` reported **zero serious
  or critical findings**. The full local suite also checks light/dark public
  routes and the demo with axe.
- Privacy: Playwright recorded only the product origin during cold landing,
  demo, state change, and export. No analytics or third-party runtime request
  occurred. The demo's required no-third-party claim passed.
- Headers/caching: HSTS, `nosniff`, strict-origin referrer policy, and a
  self-only CSP with response-header `frame-ancestors 'none'` are present.
  Documents use `no-cache`; hashed JS is `public, max-age=31536000,
  immutable`; the WebP image is one-hour `must-revalidate`; the designed 404
  is status 404 with `X-Diff-Gate-Route: not-found` and `X-Robots-Tag: noindex`.
- The documented backend allowance is now enforced live: the supplied
  single-client 100-request probe to `/api/auth/status` saw **40 HTTP 200**,
  then **60 HTTP 429**, every throttled response with `Retry-After: 1`.
- Lighthouse desktop run: Performance **98**, Accessibility **100**, Best
  Practices **100**, SEO **100**; FCP **0.90 s**, LCP **1.65 s**, CLS **0**,
  TBT **139 ms**.

## Test limitation

`docker` is not installed in this verifier container, so the exact local
`docker build` could not be run (`docker: command not found`). The local
release build and clean PORT-only runtime contract did pass. This limitation
does not affect the FAIL decision, which is based on the live service.

## Required remediation and re-verification

Deploy the candidate with the documented one-replica durable SQLite topology
(`/data` Azure Files mount and durable database configuration) and the required
Sociobot Entra and GitHub App configuration. Then independently re-run the
live deployment and identity checks. Acceptance requires `/health` 200 with
this build and `service_ready:true`, an Entra redirect rooted at
`sociobotcustomers.ciamlogin.com`, a completed signed-in team packet/import
flow, and continued 40/60 rate-limit behavior with `Retry-After`.
