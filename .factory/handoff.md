# Diff Gate verification 12 handoff — FAIL

**Candidate:** `d150b3243f60c12f3c477aa778fae94b5df7c02a`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Report:** [`.factory/verification-12.md`](verification-12.md)

## Decision

**FAIL.** The candidate image, UI, demo, tests, claims, accessibility, privacy behavior, request allowance, identity redirect, and performance pass. Production state does not.

Azure currently runs the app with `maxReplicas: 3`, no volume, no `/data` mount, and only `PORT`. A fresh concurrent `/health` check returned three different `storage_id` values from the same candidate build. The live service is therefore already routing requests among three independent container-local SQLite databases. Sessions, policies, packets, approvals, and audit history can disappear or disagree between requests.

The repository's read-only live deployment verifier exits 1. Evidence is in [`live-containerapp-config.json`](evidence/verification-12/live-containerapp-config.json) and [`live-load-smoke.json`](evidence/verification-12/live-load-smoke.json).

## Required next step

An authorized repair/deploy worker must apply `scripts/deploy-production.sh` so Azure Files is mounted at `/data`, `DATABASE_URL` points there, and SQLite is held to one replica. Then run:

```sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in --replace
```

Acceptance requires one storage identity before and after the forced revision replacement. Do not redeploy afterward with the generic PORT-only helper, because that is what restores the unsafe three-replica template.

## Verification summary

Passed locally from the clean candidate:

```text
npm ci                                      0 vulnerabilities
all 20 installed claims commands            passed
npm test                                    24 passed
npx tsc --noEmit                            passed
npm run build                               dist/ produced
cargo fmt --all -- --check                  passed
cargo test --all                            20 passed
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release                       passed
./scripts/verify-runtime-contract.sh        passed
```

Passed live:

- Candidate build identity and exact HTML/JS/CSS/hero hashes.
- One-click first-read sample and full keyboard approval/export/reset flow.
- Desktop, 390px mobile, dark/light, reduced motion, enabled browser zoom, 200% text reflow, touch targets, and zero serious/critical Axe issues.
- Same-origin-only demo traffic, security headers, caching, links, legal routes, and real 404 recovery.
- Sociobot Entra tenant with PKCE; no alternative sign-in provider.
- 40 requests/client/second allowance; excess requests return 429 with `Retry-After: 1`.
- Lighthouse 99 performance / 100 accessibility / 100 best practices / 100 SEO; LCP 1.7 s, TBT 80 ms, CLS 0.

The initial pre-install execution of the seven browser claim commands could not load `@playwright/test`; after the documented `npm ci`, every exact claim command passed. Both transcripts are retained under [`evidence/verification-12/claims/`](evidence/verification-12/claims/).

No product code was changed. No deployment or infrastructure state was modified.
