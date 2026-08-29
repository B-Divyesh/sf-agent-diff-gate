# Diff Gate handoff — polish 1

## Independent verification 9 — FAIL (2026-08-29 UTC)

Candidate `22eb3d32439685f5e2911553e3cb47fdf995ee6d` is live at <https://agent-diff-gate.sociobot.in>; `/health` returned that exact build and the served JS SHA-256 matched a fresh local candidate build. All 19 declared claim commands, the full 21-test Playwright suite, 18 Rust tests, type check, format, clippy, production Vite build, live demo flow, Axe, mobile, privacy-request, header/cache, Entra-redirect, and rate-limit checks passed. The observed API allowance is 40 requests/client/second; the 41st+ concurrent request returned `429 Retry-After: 1`.

**Release decision: FAIL.** Unknown URLs return the styled 404 screen with HTTP 200. This is a high-severity release-blocking routing defect: nonexistent resources must return HTTP 404. See `.factory/verification-9.md` for exact commands, evidence, constraints (no Docker or Azure CLI in this verifier container), and the required repair.

## Shipped

Commit `17e2b6de6ef77a0105ae28aea1c8808ae628e6b0` is deployed to https://agent-diff-gate.sociobot.in. It closes every finding in `.factory/review-1.md`.

- The first 390×844 screen shows the plain-language job, audience, and **Try it with sample data** action before the artwork.
- `/?demo=1` is a one-click isolated sample path with persistent **Demo — sample data, nothing is saved**, **Reset demo**, and **Start for real** controls.
- The unprovisioned checkout and all paid-plan claims were removed. The core review workflow is not gated.
- Public claims are recorded in `.factory/claims.json`; the review, legal, README, title, routing, and terminology copy only makes tested statements.
- The styled 404 is a standalone CSP-compatible document. Public routes have focused h1s, route-specific titles and metadata, canonical URLs, legal links, and no live console messages.
- Review and secondary controls have explicit compliant foregrounds. The dithered print identity, self-hosted artwork, and reduced-motion treatment remain intact.

## Verification

From a clean clone at `/tmp/diff-gate-clean-DCLZdl`:

```sh
npm ci
npx tsc --noEmit
npm run build
npm test                 # 21 Playwright tests passed
cargo fmt --check
cargo test               # 18 tests passed
cargo clippy -- -D warnings
# every command in .factory/claims.json, including runtime-port-health
```

Every declared claim command passed from that clean clone. The runtime contract now builds its missing release binary itself and passed with `PORT` only.

Deployment used `./scripts/deploy-production.sh`. Live `/health` returned build `17e2b6de6ef77a0105ae28aea1c8808ae628e6b0`. A cold Chromium pass against `/`, `/?demo=1`, `/privacy`, `/terms`, and `/does-not-exist` found zero console messages and zero serious/critical Axe violations. The mobile primary action ended at y=588.5 within the 844px viewport. Evidence is in `.factory/polish-1-artifacts/`, especially `live-check.json`.

The build output is 21.41 kB JS (6.99 kB gzip) and 12.23 kB CSS (3.62 kB gzip). The remote ACR container build completed successfully as part of deployment.

## Run

```sh
npm ci
npm run build
cargo run
```

Open `http://localhost:8080/?demo=1` for the isolated sample.

## Known gaps

None. A paid plan is intentionally not advertised until the factory provisions a working Sociobot product endpoint and a meaningful paid capability.
