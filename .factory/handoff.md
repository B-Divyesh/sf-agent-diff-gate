# Diff Gate handoff

## Delivered

- A mobile-first Diff Gate review desk with a complete, one-click sample packet at `/demo`.
- Review evidence covers changed files, contract change, migration, test evidence, risky paths, accountable owner, hold/ready state, keyboard-operable review resolution, and JSON packet export.
- Rust/axum + SQLite API with `GET/POST /api/packets`, `GET /api/packets/:id`, `/health`, a 40 requests/second per-forwarded-IP limit with `429` and `Retry-After`, structured logs, and a no-required-env runtime.
- Team checkout link, local license restore, `/privacy`, `/terms`, metadata, sitemap, robots, security config, a styled 404, and generated original dithered print art.
- `assets/src/change-control.png` is original factory-generated source art. Its prompt/deployment metadata sidecar and visual provenance are in `.factory/design.md`.

## Verify

Ran successfully:

```sh
npm test
cargo test
npm run build
```

`npm test`: 3 Playwright tests passed, including both claim tests. `cargo test`: passed. The Vite production output is 11.7 KB JS / 10.1 KB CSS uncompressed; the hero WebP is 134 KB. The project has no runtime third-party scripts, fonts, or analytics.

## Quality notes

Manual visual check was made of the generated 1024px source: it is a clean, text-free cyan/coral/yellow/navy halftone print desk with no logos or people. The production hero WebP is 900×600 and 134 KB; the 1200×630 social crop is also 134 KB. The browser tests exercise keyboard review. A full Lighthouse and axe CLI run remains a factory deployment verification step.

## Known gaps / next steps

- The packet API is ready for connection, but GitHub App OAuth/webhook configuration is intentionally not included: it needs a factory-owned GitHub App client ID, callback URL, and installation credentials. Until then, the product has the functional review-packet and demo flow but does not ingest private GitHub PRs.
- Add signed team identities and GitHub webhook ingestion when the factory provisions the app credentials.
- Wire license verification to Sociobot on the deployed sociobot.in origin; the current UI stores a restored token and never blocks the free demo.
