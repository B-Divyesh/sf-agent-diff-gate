# Diff Gate repair-7 handoff

## Release status

**Deployed and verified:** <https://agent-diff-gate.sociobot.in> is serving
`2c877542ae9b694f2494c8cf93048fc940f4d042` (Container App revision
`sf-agent-diff-gate--0000028`). The release repairs every verification-7
finding without changing the product class, demo, real review flow, or visual
system.

## Repairs

- Entra now defaults to the production public base when `PUBLIC_BASE_URL` is
  absent or blank. The deployment config explicitly supplies
  `https://agent-diff-gate.sociobot.in`; live `/auth/entra` emits exactly
  `redirect_uri=https%3A%2F%2Fagent-diff-gate.sociobot.in%2Fauth%2Fcallback`
  with S256 PKCE.
- `scripts/deploy-production.sh` now rebuilds the stateful template after the
  generic container helper: Azure Files `agent-diff-gate-data-v4` is mounted at
  `/data`, SQLite uses its single-replica `unix-none` VFS URL, and min/max
  replicas are both one. It strips unsupported read-only scale fields before
  the ARM patch.
- `/health` includes a non-secret durable-store UUID. Live revision replacement
  from `0000027` to `0000028` retained storage id
  `1da0c91d-ce8d-4ea1-983d-665beebfbe13`, proving the mounted database survives
  replacement. Sessions, policies, packets, audits, PKCE state, and generated
  GitHub App credentials use that same SQLite database.
- The mobile header changes to a two-column wrapping navigation below 700px.
  At 390px and 200% root text size it remains 390px wide and Privacy ends at
  370.5px; nothing is clipped.
- Invalid license restore now removes the cached token, announces that the
  license is inactive, retains an open restore form, and keeps the Sociobot
  purchase path available.
- Added executable claim coverage for the 10,000-file boundary, retention
  default/range/read cleanup, PORT-only runtime health, and durable database
  reopening. Existing public copy remains covered by the expanded
  `.factory/claims.json` inventory.
- Corrected disabled-control contrast found in the final light-mode Axe sweep;
  disabled controls now preserve legible ink instead of using low opacity.

## Verification

From a clean dependency install:

- `npm ci`, `npx tsc --noEmit`, `npm test` — pass (19 Playwright tests).
- `npm run build` — pass; output is 7.68 kB gzip JS and 3.62 kB gzip CSS.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` — pass (18 tests).
- `cargo build --release && ./scripts/verify-runtime-contract.sh` — pass;
  clean-environment runtime starts on `PORT` and health returns build/store ids.
- Every exact command in `.factory/claims.json` was run independently and
  passed (five Playwright and twelve Rust/runtime commands).
- `node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in` —
  pass: desktop, 390px mobile, dark/reduced motion, keyboard focus, Axe on all
  public routes and 404, same-origin demo requests, and loaded-offline demo.
- Live 200% reflow probe passed (`scrollWidth: 390`, viewport 390); live
  `verify-url.sh` passed with no console errors. Evidence is under
  `.factory/repair-7-artifacts/verify-url/`.
- `npm run test:live-identity` and
  `./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in --replace`
  pass. The latter checks public callback, one replica, Azure Files mount, then
  forces and validates a replacement revision.

## Known gap / next step

No production Entra user credentials or private GitHub organization were
provided to this worker. The real credential-dependent sign-in → team GitHub
App install → private PR workflow therefore cannot be completed here. The live
provider redirect, configured tenant/client, PKCE, durable state replacement,
and all fixture-backed workflow tests are verified; perform that final
account-bound smoke with a designated Sociobot team before broad rollout.
