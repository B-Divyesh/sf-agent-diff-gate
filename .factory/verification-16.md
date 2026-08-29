# Independent verification 16 — FAIL

**Requested candidate:** `88c39207f693df8986a96fb0754d3925496d4b6c`

**Available checkout / remote main:** `88c392a7825d7f92d2b97f7c44415532ffe5deec`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC, from a clean checkout

## Release decision

**FAIL.** The live product has a critical persistence split, its per-client
rate allowance is multiplied by the replica count, and the requested candidate
cannot be resolved or matched to the deployed build.

No product code or deployment configuration was changed during verification.

## Release-blocking findings

### Critical — live team state is split across three ephemeral SQLite stores

Fresh Azure control-plane evidence for active revision
`sf-agent-diff-gate--0000061` showed three healthy replicas, while the template
allows one to three replicas. It has no volume or mount and supplies only
`PORT=8080`:

```text
image: sociobotregistry.azurecr.io/sf-agent-diff-gate:88c392a7825d
scale: minReplicas=1, maxReplicas=3
active replicas: 3
volumes: null
volumeMounts: null
environment: PORT=8080 only
```

The repository's non-mutating production verifier rejected this live state:

```text
./scripts/verify-live-deployment.sh \
  https://agent-diff-gate.sociobot.in '' \
  88c392a7825d7f92d2b97f7c44415532ffe5deec

Unsafe production configuration:
- SQLite requires exactly one replica
- Azure Files volume data must use agent-diff-gate-data-v4
- Azure Files volume data must be mounted at /data
- DATABASE_URL, PUBLIC_BASE_URL, all Entra values, and
  DEPLOYMENT_CONFIG_VERSION must match the production contract
```

This is observable from the public service, not just a control-plane risk. A
single HTTP/2 burst of 300 `GET /health` requests returned HTTP 200 with the
same build but three distinct durable-store identities:

```text
92ca6217-2d35-421b-b4e4-e177c0f72fb8
e0471c65-f9ce-4f3e-9cb1-d02767ee58eb
a98a372d-1c6a-47d4-bb40-47d2cd6cbb65
```

Team packets, repository policies, sessions, approvals, retention, GitHub App
setup, and audit history can therefore appear or disappear depending on which
replica receives a request. This breaks the core accountable-review job.

The same split breaks the documented 40-request-per-client-per-second
allowance. A 150-request HTTP/2 burst from one client received 120 HTTP 200
responses before 30 HTTP 429 responses. Every 429 had `Retry-After: 1`, but the
allowance was tripled because each replica kept an independent counter. Health
is intentionally exempt and returned 300/300 HTTP 200 responses.

### Critical — requested candidate identity does not exist or match live

The requested SHA `88c39207f693df8986a96fb0754d3925496d4b6c` is not an object
in the supplied clone. GitHub's commit endpoint returned HTTP 422, “No commit
found for SHA.” Both `origin/main` and the checked-out clean tree point to
`88c392a7825d7f92d2b97f7c44415532ffe5deec`.

Live `/health` reports build
`88c392a7825d7f92d2b97f7c44415532ffe5deec`, and the live hashed JS/CSS files
are byte-identical to that checkout. The deployment therefore matches the
work-order base, but it cannot match the candidate SHA named for this review.

## Mandatory claims

`.factory/claims.json` exists, is valid, and contains 20 claims. In the literal
first pre-install invocation required by the work order, all seven Playwright
claim commands failed to load `@playwright/test` because a clean clone does not
contain `node_modules`; all 13 Rust/runtime claim commands passed. After the
required `npm ci`, every exact manifest command was rerun and all 20 passed in
one recorded run (`CLAIM_GATE_EXIT 0`):

| Claims | Installed clean-checkout result |
| --- | --- |
| `sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action`, `audit-export`, `no-third-party-runtime` | PASS — each exact Playwright command |
| `team-packet-boundary`, `named-approval`, `entra-team-installation`, `github-complete-import`, `github-revision-refresh`, `github-app-provisioning`, `repository-policy`, `retention-deletion`, `audit-history`, `github-file-limit`, `retention-limits-and-cleanup`, `durable-store-replacement` | PASS — each exact Rust command |
| `runtime-port-health` | PASS — PORT-only startup returned build and durable-store identities |

The initial pre-install failures were missing-runner failures rather than claim
assertion failures. They are recorded here because the acceptance contract
explicitly required the claim commands before any other setup.

## First-read and end-to-end product QA

The cold first screen **passes**. It says “Review agent-authored changes before
merge,” names small software teams that need a required owner and test evidence,
and makes **Try it with sample data** visible without scrolling on desktop and
390×844 mobile. The adjacent text says what the click opens.

The live sample flow passed end to end using keyboard activation only:

1. Open the sample packet.
2. Reach and activate both **Mark reviewed** controls with Tab and Enter.
3. Reach and activate **Approve for merge** with Tab and Enter.
4. Observe “Approved by Mira Chen,” a disabled **Approved** control, and the
   retained approval after reload.
5. Export valid JSON containing the title, three changed files, and four checks.
6. Reset the demo and observe the two original owner checks return.

Focus outlines were visible and solid throughout. The banner remained present:
“Demo — sample data, nothing is saved,” with **Reset demo** and **Start for
real**. The flow produced no console or page errors. Its complete outgoing
request set used only `https://agent-diff-gate.sociobot.in`; no analytics,
third-party scripts, GitHub calls, or other data transfers occurred. A loaded
demo remained usable offline. The product does not claim to be a PWA and does
not register a service worker, so update/offline-reload testing does not apply.

Representative boundaries and recovery paths also passed in the local
integration suite: 1- and 3,650-day retention are accepted while 0 and 3,651
are rejected; imports stop beyond 10,000 files; pagination reads later GitHub
file pages; wrong-owner and evidence-free approval are rejected; a changed PR
revision clears stale evidence; duplicate approval conflicts; packet deletion
removes audit history; another team receives 404; and unknown browser routes
return a designed HTTP 404. Live unauthenticated packet access returned HTTP
401 with a plain recovery instruction.

## Local quality gates

```text
npm ci                                                   PASS — 58 packages, 0 vulnerabilities
npm test                                                 PASS — 5 Node + 24 Playwright tests
npx tsc --noEmit                                         PASS
npm run build                                            PASS — dist/ generated
cargo fmt --all -- --check                               PASS
cargo test --all                                         PASS — 20 backend tests
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release                                    PASS
./scripts/verify-runtime-contract.sh                     PASS
```

The production frontend is 22,499 B JavaScript (7,223 B gzip) and 12,233 B CSS
(3,623 B gzip), below the budgets. The complete live first load measured 170
KiB. Docker, Podman, and Buildah are unavailable in this verifier image, so a
local multi-stage container build could not be run; both component production
builds and the PORT-only release-binary runtime contract passed.

## Browser, accessibility, privacy, headers, and performance

- `/opt/fleet/lib/verify-url.sh` passed: HTTP 200, `lang=en`, one `h1`, one
  `main`, complete image alt text, and no console/page errors.
- The live browser smoke passed desktop and 390×844 mobile in light and dark
  treatments, reduced motion, all public routes, keyboard focus, 200% text
  coverage from the repository suite, no horizontal overflow, and no axe
  serious/critical findings. The designed 404 also passed.
- Live headers include HSTS, `X-Content-Type-Options: nosniff`,
  `Referrer-Policy: strict-origin-when-cross-origin`, and a self-contained CSP
  with `frame-ancestors 'none'`. Documents and API responses are `no-cache`;
  hashed JS/CSS are `public, max-age=31536000, immutable`.
- Live JS and CSS SHA-256 hashes exactly match the locally built checkout.
- Fresh mobile Lighthouse runs scored 96–100 performance; the complete run
  scored 100 accessibility, 100 best practices, and 100 SEO. The complete run
  measured FCP 0.9 s, LCP 1.7 s, CLS 0, TBT 10 ms, and 170 KiB transferred.
- `/auth/entra` returns HTTP 307 only to
  `sociobotcustomers.ciamlogin.com/<tenant>/...`, with the production callback,
  authorization-code flow, and PKCE S256. The committed identity verifier
  passed.

## Scope limits and required next step

No real Entra user or private GitHub organization was supplied. Public demo,
authentication redirect, unauthenticated boundary, live concurrency, headers,
and rate limiting were exercised directly; signed-in team and GitHub behavior
was exercised with isolated integration fixtures.

Do not release this revision. First resolve the candidate SHA discrepancy.
Then deploy the intended commit only through `scripts/deploy-production.sh`,
which applies one replica, the `agent-diff-gate-data-v4` Azure Files volume at
`/data`, the SQLite/public/Entra environment contract, and the deployment
configuration version. Rerun the non-mutating verifier and require one stable
store identity plus the 40-request allowance across concurrent traffic before
acceptance.
