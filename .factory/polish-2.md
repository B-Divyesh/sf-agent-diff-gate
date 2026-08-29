# Polish 2 — cumulative review closure

Repair commit: `a34f12d3a5f41c6eb86458f89a691e8c620b3b17`  
Live URL: <https://agent-diff-gate.sociobot.in>

| Finding | Change made | Evidence |
|---|---|---|
| F-1-1 | Kept the phone hero copy and sample action before the art. | `@claim:mobile-first-action`; `tests/diff-gate.spec.ts`; live mobile recheck after deploy. |
| F-1-2 | Kept the unprovisioned paid checkout removed. | Link crawl and `no-third-party-runtime` claim; live home recheck. |
| F-1-3 | Kept the runtime-contract script self-building when the release binary is absent. | Clean-clone `./scripts/verify-runtime-contract.sh`. |
| F-1-4 | Removed or listed every product promise; expanded the export claim to cover the visible sample contents. | Every command in `.factory/claims.json` from a clean clone; copy audit. |
| F-1-5 | Kept descriptive landing headings and product one-line footer copy. | `.factory/copy-audit.md`; live home recheck. |
| F-1-6 | Kept the product vocabulary consistent: agent-authored change, required owner, review packet, and test evidence. | `.factory/copy-audit.md`; README audit. |
| F-1-7 / F-2-1 | Replaced console-error suppression with a noindex recovery-navigation contract (`X-Diff-Gate-Route: not-found`), a real designed 404 document, and an unfiltered console test. | `unknown route renders the recovery view without console errors`; `scripts/live-browser-smoke.mjs`; live `/missing-release-check`. |
| F-1-8 | Preserved explicit high-contrast control colors and deterministic Axe checks. | `npm test` (24 Playwright tests); local live-browser smoke. |
| F-2-2 | The export claim now reads and parses the downloaded JSON, asserting its title, three changed files, and four checks. | `@claim:packet-export`. |
| F-2-3 | Rewrote the cited home and README promises into tested claims or neutral instructions; added the revision claim. | `.factory/claims.json`; clean-clone claim run; `.factory/copy-audit.md`. |
| F-2-4 | GitHub imports now store and display the PR head SHA. Refresh checks the revision; approval rechecks it, clears old evidence on change, and blocks approval. | `github_revision_change_refreshes_packet_and_blocks_approval`; `@claim:github-revision-refresh`. |
| F-2-5 | Added canonical, apple-touch, Open Graph, Twitter metadata, and the complete header navigation to the dedicated 404 document. | `dedicated 404 document keeps the public header and metadata`; live `/404`. |
| F-2-6 | Added absolute Open Graph image URLs and complete Twitter title, description, and image fields on every SPA route. | `public routes declare complete absolute social metadata`; live home/demo/privacy/terms checks. |
| F-2-7 | Replaced “complete review packet,” the 404 metaphor, and the flagged README wording with concrete product language. | `.factory/copy-audit.md`; live home, 404, and README review. |

## Verification artifacts

- Local desktop and mobile browser smoke screenshots: `.factory/repair-6-artifacts/live-desktop.png` and `.factory/repair-6-artifacts/live-mobile.png`.
- The deployment recheck uses the same unfiltered browser smoke at the live URL and records the durable-service check in `.factory/handoff.md`.
