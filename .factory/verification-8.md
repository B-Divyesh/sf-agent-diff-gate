# Independent verification 8 — PASS

**Candidate:** `9d9104b0b72b502cb6e51b7bad204e4c19bce06f`

**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC

## Release decision

**PASS.** Live `/health` returned build `9d9104b0b72b502cb6e51b7bad204e4c19bce06f`, matching the candidate. No release-blocking product defects were found.

## Cold first read

A fresh desktop load says **“Review agent changes before merge”**, says it is **for small software teams** that need an owner and evidence before an agent-made change lands, and offers **“Try it with sample data”** with **“Opens a complete review packet.”** It answers what it does, for whom, and what to click first in plain words; that action opens the complete sample packet in one click. Screenshot: `verification-artifacts-8/live-first-read-desktop.png`.

## Required claims

After `npm ci`, I ran every exact `test` command in `.factory/claims.json` from this checkout. **All 17 claims passed:** `sample-sandbox`, `packet-export`, `team-packet-boundary`, `named-approval`, `entra-team-installation`, `github-complete-import`, `github-app-provisioning`, `repository-policy`, `retention-deletion`, `audit-history`, `audit-export`, `no-third-party-runtime`, `sociobot-billing`, `github-file-limit`, `retention-limits-and-cleanup`, `runtime-port-health`, and `durable-store-replacement`.

The complete suites were rerun: `npm test` passed 19 Playwright tests and `cargo test` passed 18 Rust tests. These cover sample isolation/reset/offline review/export, keyboard use, 390px touch targets/reflow, both color schemes and Axe, team isolation, owner/evidence enforcement, GitHub paging/file limits/App setup, retention/deletion/audit concurrency, Entra restriction, rate limiting, and durable storage.

## Local checks

All passed:

```text
npx tsc --noEmit
npm test                         # 19 Playwright passed
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                       # 18 Rust passed
npm run build                    # JS 7,679 B gzip; CSS 3,621 B gzip
cargo build --release
./scripts/verify-runtime-contract.sh
```

`verify-runtime-contract.sh` confirmed PORT-only startup and health build/store identities. Docker, Podman, Buildah, and Nerdctl are absent in this verifier container, so the exact Docker command could not run. This is an environment limitation: the Dockerfile's Vite and release-Rust stages were built locally; inspection confirms a multi-stage Dockerfile, `rust:1`, a non-root runtime user, `PORT`, and a default `BUILD_SHA`.

## Live exercise and accessibility

- At 390px, the live demo began with two owner checks. Resolving both enabled approval; approval succeeded; JSON export downloaded `diff-gate-packet.json` containing the approved sample and named owner. **Start for real** cleared `sessionStorage['demo:diff-gate']`.
- Desktop and 390px dark/reduced-motion sweeps covered `/`, `/demo`, `/privacy`, `/terms`, and 404. Each had exactly one `<h1>` and one `<main>`; mobile `scrollWidth` was 390. Axe found no serious/critical findings. The 3px solid keyboard focus ring was visible; reduced motion was active.
- Normal public routes had no console or page errors. The only console message in the full route sweep was the expected network error from deliberately requesting the unknown URL.
- Lighthouse reported Performance 99, Accessibility 100, Best Practices 100, SEO 100; FCP 1.4s, LCP 1.8s, CLS 0, TBT 70ms. Lighthouse produced its JSON report before a post-audit Chrome tab-crash warning.

Screenshots: `verification-artifacts-8/live-desktop.png` and `verification-artifacts-8/live-mobile.png`.

## Privacy, identity, headers, and rate limit

- Playwright request logging of landing/demo/public routes found only same-origin requests; no analytics, third-party scripts, or sample-data egress. The dedicated fresh-context no-third-party claim also passed.
- `/auth/entra` redirects only to `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650` with PKCE S256 and the production callback. No other sign-in authority appeared.
- Responses include CSP with `frame-ancestors 'none'`, HSTS, `nosniff`, and strict-origin referrer policy. Hashed JS/CSS are immutable for one year; the 136.6KB hero has a revalidating one-hour policy.
- One live client issued 120 concurrent harmless `GET /api/auth/status` requests: 40 returned 200, 80 returned `429`, all with `Retry-After: 1`. Observed allowance: **40 requests/client/second**.
- `/health` returned 200 with the candidate build and a durable store UUID. Protected reads returned 401 while malformed mutation payloads returned 422 validation errors.

## Limitation and defects

No production Sociobot credentials or private GitHub organization were available, so a real Entra login and real GitHub installation were not submitted. The live tenant/PKCE redirect, unauthenticated boundaries, and all fixture-backed team/GitHub workflow tests were verified.

**Defects: none found.**
