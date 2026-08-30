# Diff Gate verification 26 — PASS

- **Candidate:** `155e6c200f3cffa3a98f904337b695571f5ba78d`
- **Live URL:** https://agent-diff-gate.sociobot.in
- **Verified:** 2026-08-30 UTC
- **Verdict:** **PASS**

## First-read result

A cold desktop visit returned HTTP 200 with no console or page errors. The first
screen says that Diff Gate reviews agent-authored changes before merge, names
small software teams as its audience, and gives a visible **Try it with sample
data** button. Its adjacent text explains that one click opens changed files,
test evidence, and owner checks. This meets the plain-language and one-click
demo gate.

## Mandatory claims gate

`.factory/claims.json` was present. Every listed command was run from this
clean candidate checkout after `npm ci` and passed:

| Claim IDs | Command class | Result |
|---|---|---|
| `sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action`, `audit-export`, `no-third-party-runtime` | exact `npm run test:browser -- --grep @claim:…` commands | PASS (one Playwright test each) |
| `team-packet-boundary`, `named-approval`, `entra-team-installation`, `github-complete-import`, `github-revision-refresh`, `github-app-provisioning`, `repository-policy`, `retention-deletion`, `audit-history`, `github-file-limit`, `retention-limits-and-cleanup`, `durable-store-replacement` | exact named `cargo test …` commands | PASS (one Rust test each) |
| `stateful-worker-deploy` | `npm run test:unit -- --test-name-pattern @claim:stateful-worker-deploy` | PASS |
| `runtime-port-health` | `./scripts/verify-runtime-contract.sh` | PASS: PORT-only startup returned build and durable-store identities |

The export claim was additionally observed live: the download was
`diff-gate-packet.json`, parsed as JSON, and contained the sample title, three
changed files, and four checks.

## Local quality gates

- `npm ci`: passed; 58 packages installed; npm reported 0 vulnerabilities.
- `npm test`: passed: 17 Node tests and 26 Playwright tests.
- `npx tsc --noEmit`: passed.
- `cargo test --all-targets`: passed: 23 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `npm run build`: passed and produced `dist/`.
- `cargo build --release`: passed.
- Built initial bundle: JS 22.86 kB / 7.28 kB gzip; CSS 12.23 kB / 3.62 kB gzip.

## End-to-end, accessibility, privacy, and deployment checks

- Live `/health` returned the candidate SHA and a non-empty durable storage ID.
- Fresh local and live SHA-256 values matched for `index.html`, the hashed JS,
  the hashed CSS, and the hero image.
- A live one-click demo at 390×844 began blocked with two owner checks; resolving
  both enabled approval. JSON export, approval, and Reset demo all succeeded.
  The observed request log remained same-origin and had no console/page errors.
- `scripts/live-browser-smoke.mjs` passed at desktop and 390px dark/reduced
  motion: cold-load request graph, keyboard focus, one `h1`/`main`, no overflow,
  offline demo, public routes, 404 recovery, cancelled Entra recovery, and Axe
  serious/critical findings.
- `scripts/verify-live-identity.sh` passed: only the Sociobot Entra authority,
  PKCE, and an accessible cancelled-sign-in recovery page were observed.
- Browser and curl response headers included HSTS, `nosniff`, strict referrer
  policy, CSP with `frame-ancestors 'none'`, and the designed 404 contract.
  Hashed JS was immutable for one year. The 136,640-byte hero image is within
  the 300 kB mobile budget.
- The live rate probe observed the documented allowance: **40** requests from
  one client accepted, then **60** requests returned **429** with
  `Retry-After: 1`.

## Defects

None found (no critical, high, medium, or low product defects).

## Verification limitation

The worker has no `docker`, `podman`, or `buildah` binary, so a container-image
build could not be independently executed here. The Vite production build,
Rust release build, PORT-only runtime contract, and deployed candidate were
all verified successfully.
