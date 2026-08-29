# Diff Gate verification 17 handoff — FAIL

**Candidate:** `cfdd80845d42ebe477b3b51664eb41a5ab48fc68`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC

## Release result

**FAIL.** The live site serves the exact candidate build and its user-facing demo passes, but the deployed backend has an unsafe stateful topology.

Azure revision `sf-agent-diff-gate--0000067` runs candidate image `...:cfdd80845d42`, permits `maxReplicas: 3`, has no volume or mount, and has only `PORT` configured. A fresh 100-request concurrent health probe returned three different storage identities. The service is consequently using separate ephemeral SQLite stores, so persisted team packets and audit history are not safe.

The exact evidence, full test record, claims result, privacy/accessibility checks, rate-limit result, and remediation are in [`.factory/verification-17.md`](verification-17.md).

## Required next step

Redeploy through `scripts/deploy-production.sh` (not the generic container template), then rerun:

```sh
./scripts/verify-live-deployment.sh \
  https://agent-diff-gate.sociobot.in '' \
  cfdd80845d42ebe477b3b51664eb41a5ab48fc68
```

Acceptance requires exactly one replica, Azure Files `agent-diff-gate-data-v4` at `/data`, the durable SQLite URL and production Entra/public configuration, and one shared health `storage_id` across 100 concurrent requests.

## What did pass

- All 20 required claims passed from a clean checkout.
- `npm test` (7 Node + 24 Playwright), TypeScript, production frontend build, Rust format/test/clippy/release build, and the PORT-only runtime contract passed.
- The live build identity and hashed frontend assets match the candidate.
- Cold first-read, one-click demo, 390 px/mobile/keyboard/reduced-motion, axe serious/critical, same-origin demo traffic, headers/caching, Entra-only redirect, and rate limiting (40 accepted, then 60 `429 Retry-After: 1`) passed.

Docker is not installed in this verification container, so the exact Docker image build was not run. No product code was modified.
