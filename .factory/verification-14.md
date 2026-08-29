# Independent verification 14 — FAIL

**Candidate:** `f3c84474e88f34683cad44624731e98b08c7acc5`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC, from a clean checkout

## Release decision

**FAIL — critical production persistence split.** The deployed backend and static assets are the candidate, but the current Azure Container App is running the generic, unsafe topology for SQLite. It can serve different authenticated team requests from different, ephemeral packet databases. That breaks the core product contract: review packets, required-owner approvals, test evidence, repository policies, audit history, sessions, and GitHub App setup cannot be reliably retained or consistently visible to a team.

No product code or deployment configuration was modified during this verification.

## Release-blocking finding

### Critical — live deployment uses multiple ephemeral SQLite stores

`./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in` failed its read-only production-contract assertion. Fresh Azure control-plane evidence for `sf-agent-diff-gate` was:

```json
{
  "latestRevision": "sf-agent-diff-gate--0000055",
  "activeRevisionsMode": "Single",
  "scale": { "minReplicas": 1, "maxReplicas": 3 },
  "volumes": null,
  "containers": [{
    "name": "app",
    "image": "sociobotregistry.azurecr.io/sf-agent-diff-gate:f3c84474e88f",
    "env": [{"name":"PORT","value":"8080"}],
    "volumeMounts": null
  }]
}
```

The verifier specifically rejected the missing one-replica limit, Azure Files volume/mount, `DATABASE_URL`, public URL, Entra configuration, and deployment-contract version. A fresh 240-concurrent-request `/health` probe returned the candidate build on every response but two distinct storage identities:

```text
74721323-1fbf-41d0-894c-416379f71570
6647f201-eb83-4450-ba3b-af320546d8a7
```

This is direct evidence of separate live stores. The correct source deployment script exists, but has not been reflected in the current control plane. Redeploy using `scripts/deploy-production.sh` and then rerun the non-mutating live deployment verifier; production must have exactly one replica, the `agent-diff-gate-data-v4` Azure Files volume mounted at `/data`, and the complete committed environment contract.

## Required gates first

`.factory/claims.json` exists with 20 claims. From the clean checkout, `npm ci` installed the pinned 58 packages with no vulnerabilities. Every listed command passed before broader QA:

| Claim IDs | Result |
| --- | --- |
| `sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action`, `audit-export`, `no-third-party-runtime` | PASS — 7 Playwright claim tests |
| `team-packet-boundary`, `named-approval`, `entra-team-installation`, `github-complete-import`, `github-revision-refresh`, `github-app-provisioning`, `repository-policy`, `retention-deletion`, `audit-history`, `github-file-limit`, `retention-limits-and-cleanup`, `durable-store-replacement` | PASS — each exact `cargo test <name>` command |
| `runtime-port-health` | PASS — `./scripts/verify-runtime-contract.sh` |

Cold live-page first read passed. The first screen says what it does (“Review agent-authored changes before merge”), who it is for (small software teams needing an owner and evidence), and what to click first (“Try it with sample data”), including what the sample opens. The primary action was visible at 390×844 and opened the realistic isolated packet in one click.

## Source quality and product exercise

- `npm test`: PASS — 3 unit tests and 24 Playwright tests.
- `npx tsc --noEmit`: PASS.
- `npm run build`: PASS; generated `dist/`.
- `cargo fmt --all -- --check`: PASS.
- `cargo test --all`: PASS — 20 tests.
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS.
- `cargo build --release`: PASS.
- `./scripts/verify-runtime-contract.sh`: PASS; a clean environment with only `PATH`, `PORT`, and build identity served `/health` with a generated durable-store identity.

The public sample was exercised live: changed files, test evidence, two required-owner checks, keyboard check resolution, approval enablement and immutable approved state, JSON export, reset, and start-for-real cleanup. Normal, boundary, recovery, retention, ownership, team-boundary, GitHub-pagination/revision, audit-conflict, and deletion paths are covered by the passing backend/browser tests. A live signed-in team and GitHub organization were not supplied, so those authenticated external integrations were not independently exercised against a real tenant.

## Live deployment, privacy, security, and accessibility

- `/health` reports build `f3c84474e88f34683cad44624731e98b08c7acc5`. Fresh SHA-256 checks matched local candidate JavaScript, CSS, hero image, and social image exactly.
- `/auth/entra` passed the supplied identity verifier: the only authority is `sociobotcustomers.ciamlogin.com`, with the production callback and PKCE S256.
- A 240-request concurrent burst from one forwarded client to `/api/packets` returned **41×401 and 199×429**; every 429 had `Retry-After: 1`. Observed allowance is approximately 40 requests per client per second (one response crossed a one-second window boundary). `/health` is correctly exempt.
- Browser request logging for the entire landing → sample → review → export flow observed no off-origin request. Console and page errors: none.
- Live `/`, `/demo`, `/privacy`, and `/terms` returned 200 with one h1 and no axe serious/critical violations in both light and dark 390×844 contexts. Mobile had no horizontal overflow and the primary action remained in the first viewport. The full live review flow approved successfully.
- HTML uses `no-cache`; hashed JS is immutable for one year; the hero revalidates hourly. HSTS, `nosniff`, strict-origin referrer policy, and CSP `frame-ancestors 'none'` are present. An unknown route returns the designed document with HTTP 404 and `X-Robots-Tag: noindex`.
- Production frontend is 22,499 B JS (7,219 B gzip) and 12,233 B CSS (3,617 B gzip), safely within the static bundle budgets. The self-hosted hero is 136,640 B.

No Docker-compatible executable was present in this worker, so a local Docker image build could not be performed. The constituent production frontend build, Rust release build, runtime-contract launch, and the running candidate container were all verified.
