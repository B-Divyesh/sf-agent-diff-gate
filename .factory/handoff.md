# Diff Gate verification 23 handoff — FAIL

- **Candidate:** `3869a47e182c9a2040d62280ee2e0cdc9260324f`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC
- **Result:** **FAIL — do not release.**

## Release blocker

The exact candidate is live, but its real backend is unavailable. Fresh
`/health` evidence returned HTTP 503 `unsafe_configuration` with the candidate
build. `/api/auth/status` reported `service_ready:false`, no Sociobot Entra,
and no GitHub App setup; `/api/packets` and `/auth/entra` returned HTTP 503.

Azure revision `sf-agent-diff-gate--0000089` has only `PORT=8080`, a 1–3
replica range, no Azure Files volume, no `/data` mount, and none of the durable
database, public-base, Entra, or deployment-version settings. The repository's
own `scripts/verify-live-deployment.sh` failed every corresponding assertion.
This is a fresh recurrence of deployment drift, not reliance on the prior
report.

## What passed

- The cold first-read and one-click sample gate passed on desktop and 390px
  mobile.
- All 20 `.factory/claims.json` commands passed after `npm ci`.
- `npm test` passed 12 Node and 25 Playwright tests.
- TypeScript, production Vite build, Rust formatting, 21 Rust tests, clippy,
  optimized build, and the PORT-only runtime contract passed.
- The live sample passed keyboard review, blocked-state recovery, approval,
  reset, sandbox exit, offline use, same-origin request logging, desktop/mobile
  layout, reduced motion, and Axe with no serious/critical findings.
- Isolated Lighthouse mobile and desktop each scored 100/100/100/100.
- Live rate limiting accepted 40 requests and returned 60×429 with
  `Retry-After: 1`.
- Candidate/live build identity and key frontend asset hashes match.

No product source was modified. Full commands, exact responses, limitations,
and remediation are in [`.factory/verification-23.md`](verification-23.md).
Browser and Lighthouse evidence is in `.factory/verification-23-artifacts/`.

## Required next step

Deploy through `scripts/deploy-production.sh` with exactly one replica, the
`agent-diff-gate-data-v4` Azure Files mount at `/data`, the durable SQLite URL,
public base URL, deployment contract version, and Sociobot Entra settings.
Re-run the live contract, including a deliberate revision replacement and
storage-identity check. Do not use a generic PORT-only container update.
