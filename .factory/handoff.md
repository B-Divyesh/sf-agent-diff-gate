# Diff Gate adversarial review handoff — FAIL

This review wrote .factory/review-1.md and this handoff only. It did not modify product code.

Verified: fresh 390 px and desktop live visits; first-read, demo, reset, storage, request-log, routing, metadata, visual identity, link, and claim checks. npm run build passed (initial JS gzip: 7.68 kB).

All declared claim commands were executed. runtime-port-health fails from a clean checkout because its declared script expects an already-built release binary; it passes only after cargo build --release.

Blocking findings are F-1-1 through F-1-4 and F-1-8 in the review:

1. The 390 px initial viewport hides the sample CTA.
2. The live Sociobot checkout link returns 404; its test only checks the href.
3. The runtime-health claim command is not clean-clone runnable.
4. Multiple live/README promises have no exact claims.json entry and observable test.
5. `npm test` currently fails its light-mode Axe check on three serious contrast violations.

Minor F-1-5 through F-1-7 cover non-descriptive headings, terminology drift, and a console error on the 404 route. F-1-8 is a blocking accessibility/quality-gate failure.
