# Diff Gate repair 23 handoff — PASS

- **Work order:** `agent-diff-gate-repair-23`
- **Verifier report commit:** `c102d25e9c93442c416d6f9d1dff91981a2c5310`
- **Failed candidate:** `e43c4da31769b958ba9b70a575f7b8fd5e3cd458`
- **Repair commit:** `a187840d68f96ad3602745ed7e377c0dc70b3970`
- **Repair image:** `sociobotregistry.azurecr.io/sf-agent-diff-gate:a187840d68f9`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC

## Result

**PASS.** The verifier's only release blocker is repaired. Entra callback
errors are handled before an authorization code is required. A canceled or
denied sign-in now opens a normal Diff Gate recovery page with **Try sign-in
again**, **Return to Diff Gate**, and **Try it with sample data** actions.

The page returns HTTP 200 to avoid a browser console error, sends `no-cache`
and `X-Robots-Tag: noindex`, retains the product CSP and security headers, and
does not reflect the provider's untrusted `error_description`. If Entra
returns a state value, the abandoned PKCE row is discarded. Successful
authorization-code callbacks retain their previous behavior.

## Reproduction and root cause

The exact live failure was reproduced before changes:

```text
GET /auth/callback?error=access_denied
HTTP 400
Failed to deserialize query string: missing field `code`
```

`OAuthCallback` required `code` and `state`, so Axum rejected the provider's
valid OAuth error response during query extraction. The handler never had a
chance to render product copy or recovery actions. The callback query now
models `code`, `state`, `error`, and `error_description` as optional fields
and branches on provider errors first.

## Exact regression coverage

- Rust integration test
  `entra_callback_error_renders_recovery_before_requiring_code` sends the
  exact `error=access_denied` callback without `code`. It asserts the product
  page, all three recovery actions, one `h1`, no raw deserialization text, no
  reflected provider markup, PKCE cleanup, `no-cache`, `noindex`, and CSP.
- Playwright opens the exact callback against the real Rust server at 390 by
  844. It asserts HTTP 200, title, heading, actions, no console errors, no
  horizontal overflow, keyboard focus, and zero serious or critical Axe
  findings.
- The production identity verifier now fails unless the live callback has the
  recovery heading and actions and does not contain `missing field code`.
- The live browser smoke now covers this page in both desktop and 390px mobile
  profiles and records screenshots.

## Clean local verification

All commands passed from a clean `npm ci` installation of 58 packages with
zero audit vulnerabilities:

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

- `npm test`: 16 Node tests and 26 Playwright tests passed.
- `cargo test`: 22 backend unit/integration tests passed.
- Every one of the 21 exact commands in `.factory/claims.json` passed
  independently.
- Playwright covered desktop, 390px mobile, 200% text, keyboard operation,
  light/dark Axe checks, offline demo use, same-origin request privacy, route
  metadata, touch targets, and the real-backend OAuth error page.
- Vite produced `dist/`; initial JS is 22.86 kB (7.28 kB gzip) and CSS is
  12.23 kB (3.62 kB gzip).
- Local mobile Lighthouse scored 99 performance, 100 accessibility, 100 best
  practices, and 100 SEO. FCP was 1.12 s, LCP 2.02 s, TBT 61 ms, and CLS 0.
- `/opt/fleet/lib/verify-url.sh` passed the local home and OAuth recovery
  routes with no console errors, one `h1`, `lang=en`, and a `main` landmark.
- The PORT-only release runtime returned the expected build and durable-store
  identities. Docker-compatible tooling is absent locally; ACR build `ch1h7`
  completed the clean multi-stage Docker build instead.

Local evidence is in `.factory/repair-23-artifacts/`.

## Deployment and live verification

Repair commit `a187840d68f96ad3602745ed7e377c0dc70b3970` was pushed and deployed as
the image above through the existing stateful production template. Only the
`sf-agent-diff-gate` application was inspected or changed. The deployment did
not read or modify another service's application settings or secrets.

The complete live deployment contract passed before and after a deliberate
revision replacement:

- `/health` returned the repair SHA and durable storage ID
  `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- The same storage ID survived replacement with exactly one running replica
  and the existing `/data` Azure Files mount.
- The rate probe observed exactly 40 accepted requests and 60 HTTP 429
  responses; every rejection included `Retry-After: 1`.
- `/api/auth/status` returned `service_ready:true`,
  `entra_sign_in_configured:true`, and GitHub App setup available.
- `/auth/entra` redirected only to the configured Sociobot tenant with the
  production callback, PKCE S256, and no client secret.
- `/auth/callback?error=access_denied&error_description=User%20cancelled`
  returned HTTP 200 HTML with the recovery page, `no-cache`, `noindex`, and
  the production CSP. It contained no framework deserialization text.
- Live desktop and 390px mobile smoke passed public routes, recovery routes,
  keyboard focus, reduced motion, offline demo review/export, same-origin
  privacy, and serious/critical Axe checks.
- `/opt/fleet/lib/verify-url.sh` passed both the live home and callback routes
  with no console errors.
- Live mobile Lighthouse scored 100 in performance, accessibility, best
  practices, and SEO. FCP was 0.94 s, LCP 1.69 s, TBT 2 ms, and CLS 0.

Live headers, HTML, health/auth responses, Lighthouse JSON, and desktop/mobile
screenshots are in `.factory/repair-23-artifacts/live-first-deploy/`.

## Known limits and rerun

No release blocker remains. A real tenant-user sign-in and private GitHub App
installation still require a tenant identity; tenant restriction, PKCE,
provider-error recovery, team boundaries, and the fixture-backed workflow are
covered automatically.

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
node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
```

This final evidence commit is released through the same stateful template.
Its self-referential source SHA is confirmed from `/health` after deployment
rather than written into the commit before it exists.
