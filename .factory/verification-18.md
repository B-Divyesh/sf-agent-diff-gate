# Independent verification 18 — FAIL

**Candidate:** `e262b9d3c038725f9f40a90705733f3cfb1c9cf6`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC
**Result:** **FAIL — the current production backend is fail-closed and the real review workflow is unavailable.**

## Release-blocking finding

### Critical — the deployed service has unsafe configuration and returns 503 for all real backend/auth routes

This is fresh, repeatable evidence, not an inference from the earlier report.
On three consecutive requests each, the live service returned:

| Route | Current response |
| --- | --- |
| `/health` | `503` JSON: `{"status":"unsafe_configuration","build":"e262b9d3c038725f9f40a90705733f3cfb1c9cf6","storage_id":"d9dbe9e6-7b0a-4ce4-9bbd-a00ce54e79b7"}` |
| `/api/auth/status` | `503` — “Diff Gate is waiting for its durable production storage configuration. Try again shortly.” |
| `/auth/entra` | the same `503`, rather than the required Sociobot Entra PKCE redirect |

The root document itself is HTTP 200, as reported, and it serves the requested
commit. That does not make the product usable: a cold browser load immediately
requests `/api/auth/status`, receives 503, and logs `Failed to load resource:
the server responded with a status of 503`. The real-team panel says sign-in is
not configured, and a team cannot authenticate, configure its GitHub App, import
a PR, persist a packet, or record an accountable approval. Those are the core
job in the researched brief.

The repository's exact live identity check also failed because both required
routes are 503. The required rate-limit verification cannot pass on a disabled
API: `node deploy/live-rate-limit.mjs https://agent-diff-gate.sociobot.in`
observed **0 accepted** requests where the documented allowance is 40 requests
from one client, and no 429/`Retry-After` response was observable. Restore the
durable production configuration and then rerun the identity and rate probes;
the repair must produce a healthy `/health`, a Sociobot Entra-only PKCE redirect,
and 40 HTTP 200 responses followed by 60 HTTP 429 responses with `Retry-After: 1`.

## Candidate identity and first read

- **Live candidate match: PASS.** The current `/health` body identifies build
  `e262b9d3c038725f9f40a90705733f3cfb1c9cf6`; the live JS/CSS asset names also
  match this checkout's `npm run build` output.
- **Cold first-read: PASS.** The first screen says “Review agent-authored changes
  before merge,” names small software teams as the audience, and presents a
  one-click **Try it with sample data** action with an adjacent explanation that
  it opens changed files, test evidence, and owner checks.

## Mandatory claim checks — PASS

After `npm ci` from this checkout, every command in `.factory/claims.json`
passed through the product's demo entry point or its named backend fixture:

`sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`,
`no-merge-action`, `team-packet-boundary`, `named-approval`,
`entra-team-installation`, `github-complete-import`, `github-revision-refresh`,
`github-app-provisioning`, `repository-policy`, `retention-deletion`,
`audit-history`, `audit-export`, `no-third-party-runtime`, `github-file-limit`,
`retention-limits-and-cleanup`, `runtime-port-health`, and
`durable-store-replacement`.

The explicit manifest run ended `CLAIM_RUN_EXIT=0`; it includes the PORT-only
runtime contract and the durable-store replacement test.

## Local quality gates — PASS (except unavailable Docker tool)

| Check | Evidence |
| --- | --- |
| `npm test` | PASS — 8 Node tests and 24 Playwright tests |
| `npx tsc --noEmit` | PASS |
| `npm run build` | PASS — JS 22.50 kB / 7.19 kB gzip; CSS 12.23 kB / 3.62 kB gzip |
| `cargo fmt --all -- --check` | PASS |
| `cargo test --all` | PASS — 21 backend tests |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `cargo build --release` | PASS |
| `./scripts/verify-runtime-contract.sh` | PASS — PORT-only local startup returned build and durable-store identities |
| Docker production image build | Not run: `docker` is not installed in this verification container |

## Live UI, privacy, accessibility, and response policy

- The live `/demo` workflow passed at desktop: it loaded the isolated banner,
  resolved both required checks, approved the packet, reset correctly, and
  remained usable after the already-loaded page was set offline.
- At 390×844, the sample action ended at y=588.5 (inside the first viewport),
  there was no horizontal overflow, keyboard focus was visible (`3px` coral
  outline), reduced-motion media was active, and the sample banner opened.
- Axe found **zero serious or critical violations** on `/`, `/demo`, `/privacy`,
  and `/terms`, in both light and dark modes.
- Request logs for the demo contained only `https://agent-diff-gate.sociobot.in`;
  no analytics or third-party runtime traffic appeared. The root's only console
  error is the reproducible same-origin API 503 described above.
- Documents are `no-cache`; hashed JS/CSS assets are
  `public, max-age=31536000, immutable`. The root sends HSTS, `nosniff`,
  strict-origin referrer policy, and a self-contained CSP with
  `frame-ancestors 'none'`. `/404` returns HTTP 404; robots and sitemap return
  HTTP 200.

## Severity summary

| Severity | Current reproducible finding |
| --- | --- |
| Critical | Production `/health`, identity, and every real backend route are fail-closed with 503 `unsafe_configuration`; real accountable review cannot start. |
| Verification gap | The exact Docker image build could not be run because Docker is absent from this container. |

No product source code was modified during this verification.
