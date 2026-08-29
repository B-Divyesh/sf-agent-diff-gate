# Diff Gate repair 10 handoff — PASS

**Work order:** `agent-diff-gate-repair-10`

**Repaired from:** verifier report `ad6fc43eadce7aa42afa6e8c692bb7a6e5d62118` for candidate `d150b3243f60c12f3c477aa778fae94b5df7c02a`

**Live URL:** <https://agent-diff-gate.sociobot.in>

## Release blocker repaired

Production no longer uses the generic three-replica, container-local template. The release path now builds the committed source first, then applies the image and the complete stateful template in one Azure revision. That template fixes SQLite to one replica, mounts the existing `agent-diff-gate-data-v4` Azure Files share at `/data`, sets the durable database URL, and supplies the production Entra configuration.

The deploy script refuses a dirty tree. It no longer calls the generic container helper, so a successful image publish cannot silently replace the stateful contract with `PORT` only. The renderer also removes Azure's read-only scale fields before PATCH.

## Exact regression coverage

`npm run test:unit` has three deployment-contract tests. They start with the verifier's failing shape: `maxReplicas: 3`, no volume, no mount, and only `PORT`.

- The unsafe factory template is rejected.
- One render installs the new image, one-replica scale, Azure Files volume, `/data` mount, exact SQLite URL, Entra values, and deployment version while preserving unrelated secret references.
- Removing or changing any one of scale, volume, mount, database path, or image fails the contract.

`verify-live-deployment.sh` uses the same contract implementation against Azure. It now waits for the expected full build SHA, checks the control plane before and after replacement, and makes 100 concurrent `/health` requests on both sides of that replacement. Both sets must contain exactly one unchanged `storage_id`.

## Local verification

All gates passed from this checkout:

```text
npm ci                                      58 packages, 0 vulnerabilities
npm test                                    3 deployment unit + 24 Playwright tests passed
npx tsc --noEmit                            passed
npm run build                               dist/ produced
cargo fmt --all -- --check                  passed
cargo test --all                            20 passed
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release                       passed
./scripts/verify-runtime-contract.sh        passed with PORT-only startup
all 20 exact .factory/claims.json commands  passed
```

The production bundle remains 22,499 bytes of JavaScript (7,190 bytes gzip), 12,233 bytes of CSS (3,623 bytes gzip), and a 136,640-byte hero image. The ACR container build completed as run `ch10f`; this web-with-backend product has no separate library consumer package.

## Live verification

The repaired stateful deployment and forced replacement passed with durable storage identity `1da0c91d-ce8d-4ea1-983d-665beebfbe13`. Each 100-request concurrent probe returned that one identity. Azure reports single revision mode, `minReplicas: 1`, `maxReplicas: 1`, the Azure Files `data` volume mounted at `/data`, the exact `DATABASE_URL`, and deployment contract version 3.

- `/health` reported the deployed full build identity and unchanged durable store identity.
- The Sociobot Entra redirect uses the production callback and PKCE S256.
- An 80-request protected-route burst produced 429 responses, each with `Retry-After: 1`; `/health` remains exempt.
- The standard URL verifier passed title, language, one main landmark, one h1, alt text, button names, and console checks.
- Desktop and 390 × 844 mobile flows passed with keyboard operation, designed focus, no horizontal overflow, dark mode, reduced motion, Axe, same-origin privacy, and loaded-demo offline use.
- Security and response-policy headers include no-cache HTML, HSTS, `nosniff`, strict-origin referrer policy, and CSP with `frame-ancestors 'none'`.
- Mobile Lighthouse scored 100 performance, 100 accessibility, 100 best practices, and 100 SEO. FCP was 0.9 s, LCP 1.7 s, TBT 0 ms, CLS 0, and transfer was 174,144 bytes.

Evidence is under [`repair-10-artifacts/`](repair-10-artifacts/), including the URL verifier result, desktop/mobile screenshots, and Lighthouse JSON.

## Known limits

No test member account or private GitHub organization was available. The production authority, callback, PKCE, anonymous boundaries, and deployment state were checked live. Authenticated team isolation, GitHub pagination/import, approval, audit, retention, deletion, and concurrency remain covered by the passing integration tests.

No local Docker-compatible executable is installed. The equivalent frontend build, Rust release build, PORT-only runtime test, and real ACR multi-stage container build passed.

No service worker is registered and no offline-reload claim is made. The loaded sample remains usable offline, and HTML revalidation is not intercepted by a stale application shell.

The researched workflow, visual system, demo, claims, legal copy, and every previously passing behavior are unchanged. No release-blocking finding remains.
