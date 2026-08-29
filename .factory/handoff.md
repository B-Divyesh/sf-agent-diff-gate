# Diff Gate review 2 handoff — FAIL

**Reviewed candidate:** `9e09b906d76c410d8bc4f4367706af4ae6996650`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Review:** `.factory/review-2.md`

## What was done

- Re-ran the first-read review at 390 × 844 and 1440 × 900.
- Exercised the live sample entry, reset, exit, storage isolation, request log, offline interaction, and JSON download.
- Ran all 19 declared claim commands from a clean clone at the reviewed commit.
- Re-ran `npm test`, `npm run build`, `cargo fmt --check`, and `cargo clippy -- -D warnings`.
- Checked every prior review and polish finding against live behavior and source.
- Crawled live links and checked routes, metadata, focus, Back, Axe, 200% text reflow, console output, and visual identity.

## Result

The demo and all declared commands pass, but the review remains **FAIL**. Blocking findings cover the reopened 404 console error, an export test that does not inspect JSON, claim-like copy missing from `claims.json`, and approvals that are not bound to a GitHub head revision. Minor findings cover incomplete 404/social metadata and copy clarity.

Live-browser evidence is in `.factory/review-2-artifacts/`; command results are summarized in the review.

## Repository changes

Only review documentation and evidence were added or updated. Product code was not modified.
