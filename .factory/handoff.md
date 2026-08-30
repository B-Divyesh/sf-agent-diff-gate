# Diff Gate repair 24 handoff — local PASS

- **Work order:** `agent-diff-gate-repair-24`
- **Verifier report:** `14db48fd8cc59331c53a1b48ae360fb4bb3aab10`
- **Failed candidate:** `1ef3f4bdfaf67e8a7517f46757ed20551e986b94`
- **Repair commit:** `3d45555`
- **Pushed repair:** `3d45555` on `main`
- **Verified:** 2026-08-30 UTC

## Repaired release blockers

### Cold sample-sandbox command

The verifier failure was reproduced first from a new checkout of the reported
candidate: the exact command below started `cargo run` inside Playwright's
120-second web-server health window while the Rust target was cold. That is the
failure recorded in `.factory/verification-25-artifacts/cold-claim-sample-sandbox.log`.

```sh
npm run test:browser -- --grep @claim:sample-sandbox
```

The repair makes compilation an explicit part of that exact declared command.
`scripts/test-browser.sh` runs `cargo build --quiet` and then `exec`s the local
Playwright binary. `playwright.config.ts` starts the resulting debug binary
directly, so its 30-second `/health` timeout is only for process start and
SQLite initialization. Both Vite and Rust web servers refuse reuse and receive
a bounded SIGTERM shutdown (10 seconds).

The exact regression coverage is:

- `unit/browser-startup-contract.test.mjs` asserts the declared command builds
  Rust before Playwright, uses the built binary, retains the truthful timeout,
  and declares bounded shutdown.
- The clean-clone proof used a separate `mktemp` clone of repair commit
  `3d45555`, an empty Rust target, `npm ci`, then the unchanged command above.
  Rust compiled before the health probe; Playwright ran one sample-sandbox test
  and exited **0** in 3.7 seconds after the build. No Vite, Rust, or Playwright
  process remained.

### Stable malformed-JSON errors

All JSON write endpoints now use the shared `AppJson` extractor. It traverses
the extractor rejection only to choose a safe, field-specific product message;
it never returns Axum's implementation text. Missing `title`, for example,
returns HTTP 400 JSON:

```json
{"error":"Invalid title. Add a text title and try again."}
```

`malformed_json_returns_a_stable_actionable_json_error` covers the verifier's
exact missing-title body, JSON content type, status, response shape, action,
and absence of the word `deserialize`.

### Copy audit

`.factory/copy-audit.md` now exactly records the shipped sentence: “Packets
are visible only to their signed-in team.”

## Clean verification

All commands below passed after a clean `npm ci` (58 packages; 0 reported
vulnerabilities):

```sh
npx tsc --noEmit
npm test
cargo test --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
./scripts/verify-runtime-contract.sh
```

- `npm test`: 17 Node tests and 26 Playwright tests passed.
- `cargo test --all-targets`: 23 backend tests passed.
- All 21 exact commands in `.factory/claims.json` passed individually.
- The production build emitted `dist/`; JS is 22.86 kB (7.28 kB gzip) and CSS
  is 12.23 kB (3.62 kB gzip).
- The browser suite covers desktop, 390px mobile, 200% text, keyboard,
  touch-target size, offline-after-load, reduced motion, light/dark Axe,
  privacy request boundaries, route metadata, and the canceled-sign-in page.
- `/opt/fleet/lib/verify-url.sh` passed against the local production build for
  both `/` and `/auth/callback?error=access_denied`: HTTP 200, no console
  errors, `lang=en`, one `h1`, one `main`, and no missing image alternatives
  or unlabeled buttons.
- The PORT-only release runtime contract returned both the requested build and
  a durable-store identity.
- The non-mutating public live identity verifier passed Entra tenant/PKCE and
  canceled-sign-in recovery. The live browser smoke also passed desktop, 390px
  mobile, keyboard, Axe, same-origin privacy, and offline demo checks. These
  checks describe the still-live candidate, not this unreleased repair.

Docker, Podman, and Buildah are unavailable in this worker, so an image build
could not be run locally. The Dockerfile's multi-stage path is covered by the
successful Vite and Rust release builds; the factory container build remains
the deployment build.

## Deployment scope

The repaired branch was pushed. Live `/health` still reports failed candidate
`1ef3f4bdfaf67e8a7517f46757ed20551e986b94`; it does not yet report the repair.
Direct Azure deployment was deliberately not run
from this worker: repository instructions assign infrastructure deployment to
the factory, and the work-order safety boundary forbids connecting to any
resource outside this product's `sf-agent-diff-gate` scope. The existing local
deployment script reaches shared factory, registry, environment, and storage
resources, so invoking it would violate that boundary. The factory should
deploy the pushed commit through its product-scoped stateful release path, then
run the existing live identity, response-policy, rate-limit, and browser
verifiers against `https://agent-diff-gate.sociobot.in`.

## Known gaps

- A live deployment, exact build-identity check, response-policy check, and
  rate-limit check for the repair remain factory-owned pending the
  product-scoped release operation described above.
- A real Entra user and private GitHub installation still require a test
  identity; their boundaries remain covered by integration tests.
