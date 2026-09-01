# Polish 3 — cumulative review closure

Repair commit: `6095165c7993ff7d3c63de894d8ad74ac16f37d4`  
Built-routing follow-up: `71bf3ad7a4a00d6cf753980cf478cb22620f3cc7`  
Public URL: <https://agent-diff-gate.sociobot.in>

The source repair and its clean-clone evidence are complete. At the time this
file was written, the public URL still reported the previous build
`155e6c200f3cffa3a98f904337b695571f5ba78d`; the work-order container release
must promote `71bf3ad` before the final live recheck. The repairer did not run
the legacy repository deployment script because it reaches shared factory
resources outside this product's allowed boundary.

| Finding id | Change made | Evidence |
|---|---|---|
| F-1-1 | Kept the plain-language headline and sample action above the art at phone width. | `@claim:mobile-first-action`; clean-clone `mobile-first-action.log`; [`local-demo-mobile.png`](polish-3-artifacts/local-demo-mobile.png). |
| F-1-2 | Kept the unprovisioned checkout and plan copy removed. | `@claim:no-third-party-runtime`; clean-clone `no-third-party-runtime.log`; source link scan. |
| F-1-3 | Kept `verify-runtime-contract.sh` self-building when the release binary is absent. | `@claim:runtime-port-health`; clean-clone `runtime-port-health.log`. |
| F-1-4 | Kept public promises either claim-backed or instructional, and added claim-backed deployment statements. | All 23 clean-clone claim logs; [claims.json](claims.json); [copy-audit.md](copy-audit.md). |
| F-1-5 | Kept descriptive section headings and the product description in the footer. | [copy-audit.md](copy-audit.md); `npm test`. |
| F-1-6 | Kept the terms agent-authored change, required owner, review packet, and test evidence consistent. | [copy-audit.md](copy-audit.md); README audit. |
| F-1-7 / F-2-1 / F-3-1 | Removed console filtering. Browser navigations receive a 200 noindex recovery document with `X-Diff-Gate-Route: not-found`; non-navigation requests retain HTTP 404. | `container recovery navigation is noindex, explicit, and silent`; two Rust not-found tests; [`local-404-mobile.png`](polish-3-artifacts/local-404-mobile.png); local curl returned navigation 200 and non-navigation 404. |
| F-1-8 | Kept deterministic, high-contrast demo controls and Axe checks. | `npm test`: both light and dark Axe route suites pass. |
| F-2-2 | Kept the export claim parsing the downloaded JSON and asserting its packet contents. | `@claim:packet-export`; clean-clone `packet-export.log`. |
| F-2-3 | Kept cited promises removed, narrowed, or covered by exact claims. | All 23 commands in [claims.json](claims.json) passed from the clean clone. |
| F-2-4 | Kept GitHub head-revision binding, refresh, evidence invalidation, and stale-approval blocking. | `@claim:github-revision-refresh`; clean-clone `github-revision-refresh.log`. |
| F-2-5 | Kept the complete recovery-page header, metadata, icons, and links. | `dedicated 404 document keeps the public header and metadata`; local recovery screenshot. |
| F-2-6 | Kept absolute Open Graph and Twitter image metadata on public routes. | `public routes declare complete absolute social metadata`. |
| F-2-7 | Kept concrete action notes, the plain 404 heading, and direct README wording. | [copy-audit.md](copy-audit.md); `npm test`. |
| F-3-2 | Replaced broad deployment prose with three exact claims. The release template, factory hook, and two 100-response replacement probes each have a direct fixture test. | `@claim:production-stateful-template`, `@claim:stateful-worker-deploy`, and `@claim:deployment-health-replacement`; their clean-clone logs. |
| F-3-3 | Renamed “Try it” to “Try the sample review” and rewrote deployment language in plain operational terms. | [copy-audit.md](copy-audit.md); README review. |

## Verification completed before deployment

- Fresh clone: `/tmp/agent-diff-gate-clean-F4JRl4` at `71bf3ad`; `npm ci`, then all **23/23** `claims.json` commands separately.
- `npm test`: 17 Node tests, build at 7.28 kB gzip JavaScript, and 27 Playwright tests.
- `cargo test --all-targets`: 24 tests.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `npx tsc --noEmit`, and `./scripts/verify-runtime-contract.sh` all passed.
- Docker is unavailable in this worker (`docker: command not found`); the factory container build remains the deployment-time Docker verification.

## Required work-order live recheck

After the configured container deployment reports build `71bf3ad`, run the
unfiltered `scripts/live-browser-smoke.mjs` and verify:

1. <https://agent-diff-gate.sociobot.in/round-3-live-check> navigates silently,
   shows **Page not found**, and returns the noindex recovery headers.
2. The same URL returns HTTP 404 for a non-navigation request.
3. `/`, `/?demo=1`, `/demo`, `/privacy`, and `/terms` have no console or serious/
   critical Axe issue; the demo reset, export, offline path, and storage boundary work.
