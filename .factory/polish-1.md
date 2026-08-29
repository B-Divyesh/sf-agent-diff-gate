# Polish 1 — cumulative review closure

Repair commit: `17e2b6de6ef77a0105ae28aea1c8808ae628e6b0`  
Live URL: https://agent-diff-gate.sociobot.in

| Finding | Change made | Evidence |
|---|---|---|
| F-1-1 | On phones, the hero copy and primary action now precede the artwork; the action has a 390×844 viewport assertion. | `@claim:mobile-first-action`; live mobile action ends at y=588.5 in [`live-check.json`](polish-1-artifacts/live-check.json). |
| F-1-2 | Removed the unprovisioned checkout and paid-plan UI rather than linking users to a 404. Diff Gate remains a complete free review workflow. | No checkout URL remains in shipped UI/README; live cold home screenshot [`live-home-desktop.png`](polish-1-artifacts/live-home-desktop.png). |
| F-1-3 | Made `verify-runtime-contract.sh` build the release binary when missing, so its declared command is clean-clone runnable. | Clean-clone `runtime-port-health` claim passed; local script prints `Runtime contract passed`. |
| F-1-4 | Rewrote landing, legal, and README copy to only state tested behavior; added exact demo-query, mobile-first-action, and no-merge claims. | Every command in `.factory/claims.json` passed from `/tmp/diff-gate-clean-DCLZdl`; `npm test`, `cargo test`, and live cold checks passed. |
| F-1-5 | Replaced slogan headings with “Review a pull request”, “How review packets work”, and “What Diff Gate does not do”; replaced the footer slogan with the product description. | [`copy-audit.md`](copy-audit.md); live home screenshot. |
| F-1-6 | Standardized public language on “agent-authored change”, “required owner”, “review packet”, and “test evidence”. | [`copy-audit.md`](copy-audit.md); `npm test` routing/demo pass. |
| F-1-7 | Added a standalone, CSP-compatible styled 404 page with header/footer and no SPA boot. | Live `/does-not-exist`: title and h1 correct, zero console messages and zero serious/critical Axe issues in [`live-check.json`](polish-1-artifacts/live-check.json). |
| F-1-8 | Set explicit high-contrast ink for review and secondary buttons; Axe waits for final computed styles before scanning. | `npm test` 21/21 passes; live `/demo` has zero serious/critical Axe issues. |

## Additional required work

- `/?demo=1` now enters the isolated sample directly. The persistent banner offers **Reset demo** and **Start for real**; the first action uses this URL.
- Route titles, canonical URLs, descriptions, Open Graph metadata, focused `<h1>`, and the dedicated 404 route are verified in browser tests and live.
- `.factory/catalog-description.txt` is verb-first and 83 characters.
- The print/halftone visual system is preserved; no external font, script, analytics, or payment endpoint ships at runtime.

## Live evidence

- Desktop: [`live-home-desktop.png`](polish-1-artifacts/live-home-desktop.png), [`live-demo-desktop.png`](polish-1-artifacts/live-demo-desktop.png), [`live-missing-desktop.png`](polish-1-artifacts/live-missing-desktop.png)
- Mobile: [`live-home-mobile.png`](polish-1-artifacts/live-home-mobile.png), [`live-demo-mobile.png`](polish-1-artifacts/live-demo-mobile.png)
- Structured cold-browser results: [`live-check.json`](polish-1-artifacts/live-check.json)
