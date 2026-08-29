# Diff Gate verification 14 handoff — FAIL

**Candidate:** `f3c84474e88f34683cad44624731e98b08c7acc5`
**Live URL:** <https://agent-diff-gate.sociobot.in>

## Outcome

**FAIL. Do not release this deployment.** The candidate code, its 20 claims, public demo, source tests, build, live assets, browser QA, privacy behavior, Entra redirect, rate limiter, headers, caching, mobile layout, keyboard flow, and accessibility checks pass. The live persistence deployment is unsafe for the product's core accountable-review workflow.

The current Container App is candidate image `f3c84474e88f`, but has `maxReplicas: 3`, no volume, no `/data` mount, and only `PORT` configured. A fresh 240-request `/health` probe returned two different database storage IDs. The service can therefore split or lose packets, evidence, approvals, audit history, policies, and sessions across replica-local SQLite files.

## Required repair and re-verification

Run `scripts/deploy-production.sh` from a clean committed checkout. It must install the source's stateful contract: exactly one replica, Azure Files volume `agent-diff-gate-data-v4` mounted at `/data`, `sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none`, the public URL and Sociobot Entra variables, and deployment contract version 3. Then run:

```sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in
```

The command must pass and its 100 concurrent health requests must return one unchanged storage identity. No code change is required for this particular finding; a correct stateful deployment is required.

## Verification completed

- Clean install: `npm ci` (58 packages, zero vulnerabilities).
- All 20 exact `.factory/claims.json` commands passed.
- `npm test` (3 unit + 24 browser tests), TypeScript, frontend production build, Rust format/test/clippy/release build, and PORT-only runtime contract passed.
- Fresh live build identity and JS/CSS/image hashes matched the candidate.
- Live sample passed cold first read, one-click sample, keyboard review/approval, export/reset, 390px mobile, no off-origin sample requests, no console/page errors, and axe serious/critical checks in light/dark public routes.
- Live rate test: 41×401 plus 199×429 from a 240 request same-client burst; all 429 responses carried `Retry-After: 1`.

See [verification-14.md](verification-14.md) for exact evidence and scope limits. No product code was modified.
