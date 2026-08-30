# Diff Gate repair 22 handoff — PASS

- **Work order:** `agent-diff-gate-repair-22`
- **Failed candidate:** `6082c6d49f621e39d2917091242aafbfd9be365d`
- **Repair commit (pushed and deployed):** `92447a3aed4b2a08dfd922d1c7243df7c4164767`
- **Deployed image:** `sociobotregistry.azurecr.io/sf-agent-diff-gate:92447a3aed4b`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC

## Result

**PASS — the stateful web-with-backend artifact remains unchanged and the
production deployment contract now runs in the deployment-hook environment.**
The live build is `92447a3aed4b2a08dfd922d1c7243df7c4164767`, revision
`sf-agent-diff-gate--0000096`, with durable store identity
`1da0c91d-ce8d-4ea1-983d-665beebfbe13`.

## Reproduction, cause, and repair

The failure reproduced against the live candidate with the same restricted
tool PATH used by the deployment hook:

```sh
env PATH=/usr/bin:/bin ./scripts/verify-live-deployment.sh \
  https://agent-diff-gate.sociobot.in '' \
  6082c6d49f621e39d2917091242aafbfd9be365d \
  sociobotregistry.azurecr.io/sf-agent-diff-gate:6082c6d49f62
```

It reached the live 404 assertions and exited with:

```text
./scripts/verify-live-deployment.sh: 76: rg: not found
```

`rg` was an undeclared runtime dependency in two required 404-header checks.
The hook image provides POSIX tools but not ripgrep. The repair keeps both
checks intact and moves them to
[`scripts/assert-not-found-headers.sh`](../scripts/assert-not-found-headers.sh),
which uses POSIX `awk` to assert, case-insensitively:

- `X-Diff-Gate-Route: not-found`
- `X-Robots-Tag: noindex`

The HTTP 404 status assertion remains immediately before the header assertion;
no live check was removed or relaxed.

Focused regression coverage in `unit/production-deployment.test.mjs` runs the
actual helper under `PATH=/usr/bin:/bin`, accepts mixed-case CRLF headers,
rejects a missing header, and asserts that the live verifier calls the helper
and contains no `rg` runtime dependency. It passed as part of the unit and
clean test suites.

## Clean local verification

From a clean `npm ci` (58 packages; zero audit vulnerabilities), all commands
below passed:

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

Evidence:

- `npm test`: 16 Node/unit tests and 25 Playwright tests passed. This includes
  keyboard review, mobile 390px and 200% text reflow, visible focus, offline
  demo use, request privacy, route semantics, light/dark Axe checks, and
  desktop/mobile touch targets.
- Every one of the 21 exact commands in `.factory/claims.json` was run
  independently and passed, including the sandbox, packet export, real-team
  boundaries, named approval, retention/deletion, audit export, no third-party
  runtime, PORT-only health, durable reopen, and stateful worker-deploy claims.
- `cargo test`: 21 backend unit/integration tests passed. Formatting and
  warning-free Clippy passed, as did the release build.
- Vite produced `dist/`; initial JS is 22.86 kB (7.28 kB gzip) and CSS is
  12.23 kB (3.62 kB gzip).
- The PORT-only runtime contract passed with both build and durable-store
  identities.

## Deployment and live verification

The repair was pushed, then deployed only through the work-order stateful
release configuration:

```sh
env PATH=/usr/bin:/bin ./scripts/deploy-production.sh
```

ACR build `ch1fr` succeeded from a `.git`-free source archive. The release
rendered the existing durable SQLite/Entra contract into the image above and
ran its built-in full `verify-live-deployment.sh --replace` contract plus the
live browser smoke suite. The replacement preserved the same durable storage
identity.

I then reran the exact full contract explicitly, still without ripgrep:

```sh
env PATH=/usr/bin:/bin ./scripts/verify-live-deployment.sh \
  https://agent-diff-gate.sociobot.in --replace \
  92447a3aed4b2a08dfd922d1c7243df7c4164767 \
  sociobotregistry.azurecr.io/sf-agent-diff-gate:92447a3aed4b
```

It passed. The final constrained-PATH rerun reported:

```text
Production control-plane configuration is safe for SQLite.
Live identity configuration is ready and redirects only to Sociobot Entra with PKCE.
Live rate limit passed: 40 accepted, 60 returned 429, and every rejection sent Retry-After: 1.
Live deployment contract passed: expected build, one concurrent storage identity,
global 40-request allowance with Retry-After, public Entra callback, one replica,
Azure Files /data, and durable replacement identity 1da0c91d-ce8d-4ea1-983d-665beebfbe13.
```

Additional live evidence:

- `/health` returns HTTP 200, `status:"ok"`, the deployed repair SHA, and the
  durable store identity above.
- `/api/auth/status` returns `service_ready:true`,
  `entra_sign_in_configured:true`, and `github_app_setup_available:true`.
- Control-plane contract assertion passed: Single revision mode, exactly one
  replica, Azure Files `agent-diff-gate-data-v4` mounted at `/data`, production
  SQLite URL, production public base URL, Entra values, and expected image.
- The deliberate revision replacement advanced the app to revision `0000096`
  without changing the storage identity. The live rate test again observed 40
  accepted requests and 60 `429` responses, each with `Retry-After: 1`.
- `node scripts/live-browser-smoke.mjs` passed on live desktop and 390px mobile:
  cold-load console/resource checks, public routes, 404 response and recovery
  view, keyboard focus, serious/critical Axe checks, reduced motion, offline
  sample review/export, and same-origin-only demo requests.
- `/opt/fleet/lib/verify-url.sh` passed in 563 ms with no console errors. It
  recorded title `Diff Gate — Review agent-authored changes before merge`,
  `lang=en`, one `h1`, a `main` landmark, no images missing alt text, and no
  unlabeled buttons. Its artifacts are in `.factory/repair-22-artifacts/`.

The standalone `npx @axe-core/cli` launcher could not run because this worker
has no Chrome binary. This does not leave an accessibility gap: the repository
uses the preinstalled Playwright Chromium and `@axe-core/playwright` 4.11.0 in
both the full browser suite and the final live smoke; all public routes and the
404 page passed with zero serious or critical violations.

## Known limits and re-run

No release blocker remains. Interactive Sociobot Entra sign-in and private
GitHub App installation still require a real tenant user; the live tenant-only
PKCE redirect and service readiness are verified automatically.

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
env PATH=/usr/bin:/bin ./scripts/deploy-production.sh
```
