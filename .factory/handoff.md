# Diff Gate independent verification 13 handoff — FAIL

**Work order:** `agent-diff-gate-verify-13`

**Candidate:** `9abea0da06876e8284b083ec45fbb03a25b6471b`

**Live URL:** <https://agent-diff-gate.sociobot.in>

## Decision

**FAIL. Do not release.** The candidate image is live and the source, claims, demo, accessibility, privacy, identity, headers, caching, and performance gates pass. Production has drifted back to the unsafe stateless template: `maxReplicas: 3`, no volume, no `/data` mount, and only `PORT` configured.

Fresh load produced three running replicas and three different `/health` `storage_id` values for the same build. The effective per-client allowance also multiplied to 120 requests before 429 across the three replica-local counters. This breaks durable, consistent review packets and audit history.

## Required repair

Deploy only through `scripts/deploy-production.sh`. Confirm one replica, Azure Files `agent-diff-gate-data-v4` mounted at `/data`, the committed `DATABASE_URL`, production Entra settings, and deployment contract version 3. Then run the repository's live deployment verifier with `--replace` and prove one unchanged `storage_id` before and after revision replacement.

## Verification summary

- All 20 exact `.factory/claims.json` commands passed after `npm ci`.
- `npm test`: 3 unit and 24 Playwright tests passed.
- TypeScript, Vite production build, Rust formatting, 20 Rust tests, Clippy, release build, and the PORT-only runtime contract passed.
- Live end-to-end sample, invalid-input recovery, desktop, 390 px mobile, keyboard, focus, reduced motion, axe, privacy request log, headers, caching, Entra PKCE, and real 404 passed.
- Mobile Lighthouse: 99 performance, 100 accessibility, 100 best practices, 100 SEO.
- Local and live HTML/JS/CSS/hero hashes match; `/health` reports the full candidate SHA.
- No Docker-compatible executable or authenticated test-team account was available. Equivalent release builds and isolated authenticated integration coverage passed.

The full report is [`.factory/verification-13.md`](verification-13.md). Evidence is under [`.factory/evidence/verification-13/`](evidence/verification-13/).

No product code was modified.
