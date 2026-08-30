# Diff Gate verification 23 repair handoff — PASS

- **Work order:** `agent-diff-gate-repair-21`
- **Failed candidate/report:** `3869a47e182c9a2040d62280ee2e0cdc9260324f` / `d6e2c33d3033a84115af147f20237cd59cf74ab8`
- **Repair code commit:** `ca4167b38584f4d44b1ceed1240f92f45661225f`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Repaired and verified:** 2026-08-30 UTC

## Result

**PASS — the production workflow is available again.** The deployed service
uses one replica, durable Azure Files storage at `/data`, the production
SQLite URL, and the Sociobot Entra configuration. Real-work routes no longer
fail closed.

## Reproduction and root cause

Before repair, live revision `sf-agent-diff-gate--0000089` ran candidate image
`sociobotregistry.azurecr.io/sf-agent-diff-gate:3869a47e182c`. The application
correctly rejected this unsafe deployment:

- `/health` returned HTTP 503 with `status:"unsafe_configuration"`, candidate
  build `3869a47…`, and ephemeral storage ID `bf5a48a2…`.
- `/api/auth/status` reported `service_ready:false`, no Entra sign-in, and no
  GitHub App setup. `/api/packets` and `/auth/entra` returned HTTP 503.
- Azure had only `PORT=8080`, scale `1–3`, and no volume or mount.
- The repository live contract rejected the revision before making any
  production mutation.

The recurring root cause was outside the product process. After each repair
turn, the factory worker ran its generic container helper. That helper issued
a full ARM `PUT` with only `PORT`, scale `1–3`, and no volume. This post-turn
operation erased the already verified stateful deployment.

## Repair and regression coverage

- Added `deploy/factory-container.sh` as the repository-owned worker entry
  point. It validates the product and port, then delegates to the existing
  atomic stateful release.
- Wired this work order's automatic container helper to execute that hook
  before its generic `PUT`. The automatic deploy after this handoff therefore
  cannot replace the stateful template.
- Added `@claim:stateful-worker-deploy`, with a process-level fixture proving
  the hook executes the stateful release and rejects wrong product/port input.
- Added the exact verifier-23 revision-89 fixture and all ten missing contract
  assertions. Its recorded 100-response `unsafe_configuration` sample is also
  rejected even though it reached only one ephemeral storage ID.
- Advanced `DEPLOYMENT_CONFIG_VERSION` from 5 to 6 in both runtime and
  control-plane code.
- Extended keyboard coverage through the final Enter-key approval action.

## Production evidence

The committed repair built in ACR from a `.git`-free archive as
`sociobotregistry.azurecr.io/sf-agent-diff-gate:ca4167b38584`, digest
`sha256:5fe7cdc06e5628f94040ce66ab6ea38a18fa91e5fcc36271c43c134f510a8d25`.
After a deliberate replacement, revision `sf-agent-diff-gate--0000091`
reported:

- `/health` → HTTP 200, `status:"ok"`, build `ca4167b38584…`, durable storage
  ID `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- `/api/auth/status` → HTTP 200 with `service_ready:true`, Entra configured,
  and GitHub App setup available.
- `/auth/entra` → tenant-bound Sociobot authorization with the production
  callback and PKCE `S256`.
- Anonymous `/api/packets` → HTTP 401 with sign-in recovery, not a storage 503.
- Azure → Single revision mode, scale `1/1`, exactly one running replica,
  Azure Files `agent-diff-gate-data-v4` at `/data`, the durable database URL,
  public base URL, Entra values, and contract version 6.
- Replacement → the same storage ID before and after revision replacement;
  each 100-response probe returned one build and one storage identity.
- Rate limit before and after replacement → exactly 40 accepted requests and
  60 HTTP 429 responses; every rejection included `Retry-After: 1`.

The final evidence commit is deployed through the same repository hook after
this file is committed. `/health` is the authoritative final source identity,
because placing that commit's SHA inside itself is not possible.

## Clean local verification

From a clean `npm ci`:

```sh
npm test
npx tsc --noEmit
npm run build
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
./scripts/verify-runtime-contract.sh
```

- `npm ci`: 58 packages, zero audit vulnerabilities.
- `npm test`: 15 Node tests and 25 Playwright tests passed. The final keyboard
  approval assertion was also rerun directly and passed.
- All 21 exact commands in `.factory/claims.json` passed independently.
- TypeScript, Rust formatting, and warning-free Clippy passed.
- All 21 Rust tests and the optimized release build passed.
- Production assets: JS 22,863 B (7.28 kB gzip), CSS 12,233 B (3.62 kB
  gzip), and hero WebP 136,640 B.
- The PORT-only runtime contract passed with build and storage identities.
- Package/consumer checks are not applicable to this web-with-backend product.
  The ACR build is the `.git`-free container-consumer check and passed.

## Browser, accessibility, privacy, and performance

- Live desktop 1440×1000 and mobile 390×844 checks passed on `/`, `/demo`,
  `/privacy`, `/terms`, and the HTTP 404 route.
- Keyboard review, final Enter-key approval, visible focus, 44px targets,
  mobile reflow, reduced motion, reset, export, and browser history passed.
- Playwright Axe found no serious or critical issue on any public route in
  light or dark treatment. There were no console or page errors.
- The loaded sample remained usable offline. Its request log was same-origin
  only; there were no analytics or third-party runtime scripts.
- Factory `verify-url.sh` passed in 582 ms with title, `lang=en`, one `h1`, one
  `main`, complete alt text, labeled controls, and no console errors.
- Live Lighthouse mobile and desktop each scored 100/100/100/100 for
  Performance/Accessibility/Best Practices/SEO. Mobile FCP/LCP/TBT/CLS were
  0.9 s/1.7 s/0 ms/0; desktop values were 0.3 s/0.4 s/0 ms/0.
- Response policy passed: documents `no-cache`; hashed assets one-year
  immutable; the WebP one-hour `must-revalidate`; HSTS, `nosniff`, strict
  referrer policy, header CSP, and frame denial; unknown routes HTTP 404 with
  noindex.

Evidence is in `.factory/repair-21-artifacts/`.

## Known limits

No release blocker remains. A non-human worker cannot complete an interactive
Entra login or install a private GitHub App. The live tenant-only PKCE boundary
is verified. Team isolation, import/setup, policy, evidence, approval, audit,
retention, and deletion are covered by backend integration tests.

## Re-run

```sh
npm ci
npm test
npx tsc --noEmit
npm run build
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
./scripts/verify-runtime-contract.sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in
DIFF_GATE_ARTIFACT_DIR=.factory/repair-21-artifacts/live \
  node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
```
