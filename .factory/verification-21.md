# Verification 21 — FAIL

**Candidate:** `ce5bf429b0b5bf119773fd50eee846ff69c97612`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-30 UTC
**Method:** independent clean-checkout verification; no product code changed

## Release decision

**FAIL — release blocking.** The live frontend and backend both identify as the
candidate, but the backend has failed closed. A real team cannot sign in with
Sociobot Entra, set up its private read-only GitHub App, import a pull request,
record evidence, or approve a packet. The working sample is not a substitute
for the researched brief's real accountable-review workflow.

## Mandatory gates

### Claims — PASS

`.factory/claims.json` exists and lists 20 claims. From the clean candidate,
after `npm ci`, every exact listed command passed:

| Claims | Evidence |
| --- | --- |
| Browser demo claims (`sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action`, `audit-export`, `no-third-party-runtime`) | Each exact `npm run test:browser -- --grep @claim:<id>` command passed. |
| Backend claims (team boundary, named approval, Entra/team installation, GitHub pagination/revision/provisioning, repository policy, retention/deletion, concurrent audit, file limit, retention limits, durable storage) | All 12 exact named `cargo test <test-name>` commands passed; the `set -e` sequence then reached the runtime claim. |
| `runtime-port-health` | `./scripts/verify-runtime-contract.sh` passed twice. The release binary starts with only `PATH`, `PORT`, and `BUILD_SHA`, then `/health` returns the supplied build and a non-empty durable store identity. |

### Cold first read — PASS

A fresh desktop load says **“Review agent-authored changes before merge.”** It
says it is for small software teams needing a required owner and test evidence
before an agent-authored change lands. Its first action is **“Try it with
sample data”** and the adjacent text says it opens changed files, test evidence,
and owner checks. This plainly answers what it does, for whom, and what to
click. The one-click demo opens an isolated review packet with the persistent
“Demo — sample data, nothing is saved” banner, Reset demo, and Start for real.

## Release-blocking defect

### Critical — real-team workflow unavailable in the live candidate

Fresh live responses on 2026-08-30 UTC:

```text
GET /health → 503
{"status":"unsafe_configuration",
 "build":"ce5bf429b0b5bf119773fd50eee846ff69c97612",
 "storage_id":"a438ce08-a123-4589-ab59-19f70c2058f3"}

GET /api/auth/status → 200
{"service_ready":false,"authenticated":false,
 "entra_sign_in_configured":false,"github_app_setup_available":false,
 "github_app_configured":false,...}

GET /auth/entra → 503
```

The product's page gracefully reports that the team workspace is temporarily
unavailable, and the demo remains usable. However, the acceptance contract is
a GitHub-app review-control surface for real small teams, not a demo-only
packet viewer. There is no usable Sociobot Entra redirect to
`sociobotcustomers.ciamlogin.com`, so the required signed-in end-to-end flow
cannot be exercised or accepted.

## Candidate/deployment identity

The live backend health body names the tested commit exactly. The two deployed
hashed frontend assets byte-match this checkout's production build:

```text
aff40057e8e6dbf60c104628ab39f89ca302f19fd6d23ccd8b755defefd26487  index-r2CfzYxf.js
f17d5315eaddad9a13022bb9a75d3a64898058ce08639db68f6651b7234c2c65  index-rQ7R4Jb-.css
```

## What passed

- `npm test`: **10 Node unit tests and 25 Playwright tests passed**.
- `npx tsc --noEmit`, `npm run build`, `cargo fmt --check`, `cargo test`, and
  `cargo clippy -- -D warnings` passed. The exact production build has 22,863 B
  JS (7,297 B gzip) and 12,233 B CSS (3,617 B gzip), within the initial static
  bundle budget.
- Independent live Playwright smoke checks passed on desktop and 390×844 mobile:
  `/`, `/demo`, `/privacy`, `/terms`, and a real 404 have one `main` and one
  `h1`, `lang=en`, route titles, no horizontal overflow, visible keyboard
  focus, no console/page errors, reduced-motion-safe demo behavior, and no
  serious or critical Axe findings. Evidence screenshots are in
  `.factory/verification-artifacts-21/`.
- Privacy: a cold load plus demo launch, review action, and export made requests
  only to `agent-diff-gate.sociobot.in`. No analytics or third-party runtime
  request was observed. The loaded demo remained usable offline.
- Headers/caching: documents use `no-cache`; hashed JS/CSS use
  `public, max-age=31536000, immutable`; the 136,640 B WebP uses one-hour
  `must-revalidate`. HSTS, `nosniff`, strict-origin referrer policy, and a
  self-only response-header CSP with `frame-ancestors 'none'` are present.
  `/404` is an HTTP 404 with `X-Diff-Gate-Route: not-found` and
  `X-Robots-Tag: noindex`.
- Rate limiting is enforced for the checked server endpoint: a single-client
  100-request probe to `/api/auth/status` observed **40 HTTP 200**, then **60
  HTTP 429**, with `Retry-After: 1` on every rejection.

## Limitation

`docker` is absent in this verifier container, so the repository's exact local
Docker build/run command could not be executed. The release binary and its
PORT-only runtime contract did pass. This is not the reason for the FAIL.

## Required remediation

Deploy this candidate with the required durable stateful topology and runtime
configuration: one replica with durable `/data` SQLite storage, a production
public base URL, Sociobot Entra authority/client/callback, and the team-bound
GitHub App configuration. Re-verification must show `/health` HTTP 200 for
this commit, `service_ready:true`, a PKCE redirect rooted at
`sociobotcustomers.ciamlogin.com`, and a completed signed-in team workflow.
