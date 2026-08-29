# Independent verification 15 — FAIL

**Candidate:** `43c2f38a2e95be07377fd2938018466a990c2cf7`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC, from a clean checkout

## Release decision

**FAIL — critical production persistence split.** The public site and
`/health` identify themselves as the requested candidate, but the deployed
Container App is not using the committed stateful SQLite topology. A fresh
100-way health probe received two durable-store identities. Team packets,
approvals, audit history, repository policies, sessions, and GitHub App setup
can therefore be routed to different ephemeral stores. This breaks the core
review-control product contract.

No product code or deployment configuration was modified during verification.

## Release-blocking defect

### Critical — candidate image is deployed with the unsafe generic topology

At verification time `GET /health` reported build
`43c2f38a2e95be07377fd2938018466a990c2cf7`, so the served binary is the
candidate. However, a fresh concurrent probe of that endpoint returned 100
HTTP 200 responses with these two `storage_id` values:

```text
7d5a7304-a848-4414-8493-f6a8a5dc10f5
18e5872c-80ab-461c-8c17-489a324834a3
```

The required non-mutating verifier failed before the traffic probe:

```text
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in '' \
  43c2f38a2e95be07377fd2938018466a990c2cf7

Unsafe production configuration:
- SQLite requires exactly one replica
- Azure Files volume data must use agent-diff-gate-data-v4
- Azure Files volume data must be mounted at /data
- DATABASE_URL, PUBLIC_BASE_URL, Entra values, and DEPLOYMENT_CONFIG_VERSION
  must match the production contract
```

Read-only Azure control-plane evidence for revision
`sf-agent-diff-gate--0000058` was:

```json
{
  "activeMode": "Single",
  "scale": {"minReplicas": 1, "maxReplicas": 3},
  "volumes": null,
  "containers": [{
    "name": "app",
    "image": "sociobotregistry.azurecr.io/sf-agent-diff-gate:43c2f38a2e95",
    "env": [{"name":"PORT","value":"8080"}],
    "volumeMounts": null
  }]
}
```

This repeats the deployment-only issue found in verification 14. Redeploy with
`scripts/deploy-production.sh`, then rerun the non-mutating live deployment
verifier. Do not release until it confirms one replica, the required Azure
Files `/data` mount and environment contract, and one store identity across
100 concurrent health responses.

## Mandatory claims — PASS

`.factory/claims.json` exists and contains 20 claims. After `npm ci` from the
clean checkout, every exact listed command passed before broader QA:

| Claims | Evidence |
| --- | --- |
| `sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action`, `audit-export`, `no-third-party-runtime` | Seven exact `npm run test:browser -- --grep @claim:...` commands passed. |
| `team-packet-boundary`, `named-approval`, `entra-team-installation`, `github-complete-import`, `github-revision-refresh`, `github-app-provisioning`, `repository-policy`, `retention-deletion`, `audit-history`, `github-file-limit`, `retention-limits-and-cleanup`, `durable-store-replacement` | Each exact `cargo test <test-name>` command passed. |
| `runtime-port-health` | `./scripts/verify-runtime-contract.sh` passed: PORT-only startup returned build and durable store identities. |

## Product, quality, and browser QA — PASS except release topology

- Cold live-page first read passed: it plainly says it reviews agent-authored
  changes before merge, names small software teams as the audience, and makes
  **Try it with sample data** the visible first action with an explanation of
  what opens.
- Live demo end to end passed: sample packet → resolve two owner checks → JSON
  export (`diff-gate-packet.json`) → approval becomes immutable → reset
  restores two checks → Start for real discards demo mode.
- `npm test` passed (4 Node tests and 24 Playwright tests); `npm run build`
  passed and produced `dist/`; `cargo test` passed (20); `cargo fmt --check`,
  `cargo clippy -- -D warnings`, and `cargo build --release` passed.
- The local Docker command could not run because this verifier image has no
  `docker` executable (`docker: command not found`), not because of a product
  build failure. The Dockerfile was inspected and its component frontend and
  Rust release builds passed.
- `node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in`
  passed: desktop and 390×844 mobile, dark/reduced-motion mode, keyboard
  focus, no console/page errors, no horizontal overflow, no axe
  serious/critical findings on `/`, `/demo`, `/privacy`, `/terms`, and the
  designed 404, offline demo behavior, and same-origin-only demo requests.
- Live request logging for landing and demo observed only the product origin.
  The browser response has CSP with `frame-ancestors 'none'`, HSTS, `nosniff`,
  strict-origin referrer policy, no-cache documents, and a one-year immutable
  hashed JavaScript asset. Build output is 22,499 B JavaScript (7,190 B gzip)
  and 12,233 B CSS (3,620 B gzip), below the static budgets.
- `./scripts/verify-live-identity.sh` passed: only
  `sociobotcustomers.ciamlogin.com` is used, with the production callback and
  PKCE S256. `/health` is intentionally exempt; a concurrent anonymous burst
  to `/api/auth/status` produced 429 responses with `Retry-After: 1` after the
  40-request-per-client-per-second allowance.

## Scope limits

No real Entra team or private GitHub organisation was supplied. The live
public flow, authentication redirect, rate limiter, and anonymous API boundary
were independently tested; signed-in team isolation, GitHub fixture imports,
approval conflicts, retention/deletion, and durable reopen are covered by the
passing integration claim tests.
