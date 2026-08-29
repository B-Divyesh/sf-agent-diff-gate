# Diff Gate verification 15 handoff — FAIL

**Tested candidate:** `43c2f38a2e95be07377fd2938018466a990c2cf7`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC

## Result

**FAIL — do not release.** The running image reports the tested candidate SHA,
but production is configured as up to three replicas with no Azure Files
volume, `/data` mount, or required production environment contract. A fresh
100-concurrent-request health check returned two store identities:
`7d5a7304-a848-4414-8493-f6a8a5dc10f5` and
`18e5872c-80ab-461c-8c17-489a324834a3`.

This is a critical persistence defect: authenticated team data can be split
between ephemeral SQLite databases. The product cannot reliably retain or
scope its packets, required-owner approvals, audit history, policies, sessions,
or GitHub App setup.

## What passed

- All 20 `.factory/claims.json` commands passed from the clean checkout.
- `npm test` (4 Node + 24 Playwright), `npm run build`, `cargo fmt --check`,
  `cargo test` (20), `cargo clippy -- -D warnings`, `cargo build --release`,
  and the PORT-only runtime contract passed.
- Live first-read, one-click sample demo, full sample approval/export/reset
  flow, desktop and 390px mobile, keyboard, reduced motion, offline demo,
  axe serious/critical, console/page errors, privacy request log, headers,
  cache policy, Entra-only PKCE redirect, and API rate limit all passed.
- Live `/health` reports the requested candidate SHA. A single generic
  response is therefore not sufficient evidence of a safe deployment.

## Required next step

Run `scripts/deploy-production.sh` from a clean committed tree. Then run:

```sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in '' \
  43c2f38a2e95be07377fd2938018466a990c2cf7
```

It must pass with exactly one replica, the `agent-diff-gate-data-v4` Azure
Files volume mounted at `/data`, the committed environment contract, and one
unchanged store identity under 100 concurrent health requests.

See [verification-15.md](verification-15.md) for exact command output,
evidence, severity, and the local Docker-tooling limitation.
