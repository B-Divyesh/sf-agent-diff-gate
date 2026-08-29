# Independent verification 10 — PASS

**Candidate:** `2200fa4875ee6691688da36ea3152ee0884497ae`
**Live URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC

## Release decision

**PASS.** The live service identifies itself as the requested candidate, and all required claim, local quality, demo, privacy, accessibility, deployment, rate-limit, and performance checks passed. The prior deployment-only 404 failure is not present: an unknown live route returns the styled recovery document with HTTP `404`.

## Cold first read

A new desktop browser session said **“Review agent-authored changes before merge.”** It plainly says this is for **small software teams** that need a required owner and test evidence before an agent-authored change lands. The first action is **“Try it with sample data”**, immediately followed by **“Opens a complete review packet.”** One click opened the realistic, isolated sample packet. The same primary action was wholly visible in a fresh 390 x 844 viewport (bottom at y=588.52). This passes the plain-words and one-click demo gates.

## Required claims gate

`.factory/claims.json` exists and its 19 exact declared commands were run from this checkout after `npm ci`. Every claim passed.

| Claims | Exact command family | Result |
| --- | --- | --- |
| `sample-sandbox`, `packet-export`, `demo-query-path`, `mobile-first-action`, `no-merge-action`, `audit-export`, `no-third-party-runtime` | `npm run test:browser -- --grep @claim:<id>` | PASS (each exact command) |
| `team-packet-boundary`, `named-approval`, `entra-team-installation`, `github-complete-import`, `github-app-provisioning`, `repository-policy`, `retention-deletion`, `audit-history`, `github-file-limit`, `retention-limits-and-cleanup`, `durable-store-replacement` | `cargo test <declared test name>` | PASS (each exact command) |
| `runtime-port-health` | `./scripts/verify-runtime-contract.sh` | PASS — clean environment with only `PATH`, `PORT`, and build identity started the release server and `/health` returned the configured build plus a durable-store id. |

The browser claim tests use the product's `/demo` / `?demo=1` entry point and sample packet, not an account or manually seeded production data.

## Clean-checkout quality suite

All passed:

```text
npm ci
npx tsc --noEmit
npm test                         # 21 Playwright tests passed
npm run build                    # Vite production build passed
cargo fmt --check
cargo test                       # 19 Rust tests passed
cargo clippy -- -D warnings
./scripts/verify-runtime-contract.sh
```

The build emitted 21,413 B JavaScript (7,018 B gzip) and 12,233 B CSS (3,623 B gzip), below the static initial-JS and CSS budgets. The bundled hero image is 136,640 B, below the 300 kB mobile-image budget. Docker is unavailable in this verifier container, so a local Docker build was not run; the release binary itself was built and executed by the runtime-contract check.

## Live deployment, functional QA, and security boundaries

- `GET /health` returned `200` with `build: "2200fa4875ee6691688da36ea3152ee0884497ae"` and durable storage id `6c09102e-4cf3-4c72-81f5-031e7b95cc67`.
- The live hashed JS SHA-256 was `37ea2c13ec18d541056cb6073ffd8e020275d07ad8401cd702edc35f1bdcd07d`, exactly matching the freshly built local candidate artifact.
- In a fresh live demo, keyboard Enter resolved both required checks, enabled approval, approved the packet, downloaded `diff-gate-packet.json`, retained the approval after reload, and **Start for real** removed `sessionStorage['demo:diff-gate']`.
- Anonymous `GET /api/packets` returned `401`; invalid packet input returned `422`. Server-side tests cover the authenticated normal, incorrect-owner, missing-evidence, cross-team, conflicting concurrent-approval, retention/deletion, GitHub pagination/limit, and policy-path boundary cases.
- All public links were crawled. Internal and fragment targets returned `200`; the sign-in endpoint returned `307` exclusively to `sociobotcustomers.ciamlogin.com` using the configured Sociobot tenant, production callback, and PKCE S256.
- A random unknown live URL returned `404` with the designed recovery page, fixing the prior release-blocking issue.

## Privacy, accessibility, responsive behavior, and performance

- Playwright's complete live flow request log (landing, demo, two evidence changes, export, approval, reload, and exit) contained only `https://agent-diff-gate.sociobot.in`. There were no analytics, third-party scripts, or sample-data egress.
- Fresh light and dark 390 px checks of `/`, `/demo`, `/privacy`, and `/terms` each found one h1, no console/page errors, and no serious or critical Axe findings.
- At 390 px there was no horizontal overflow; every verified interactive target met the 44 px test; the focused sample action had a visible `3px solid rgb(201, 76, 59)` outline. The complete review action was operable by keyboard. Reduced-motion emulation was active and the page remained functional.
- Live response headers include HSTS, `X-Content-Type-Options: nosniff`, strict-origin referrer policy, and CSP with `frame-ancestors 'none'`. HTML is `no-cache`; hashed JS/CSS are `public, max-age=31536000, immutable`; the stable hero asset is revalidated hourly.
- Fresh idle mobile Lighthouse: **99 performance**, **100 accessibility**, **1.9 s LCP**, **40 ms TBT**, **0 CLS**, and **169 KiB** total transfer.

## Request allowance

Using one browser client, 55 simultaneous harmless calls to `/api/auth/status` produced **40 x 200** and **15 x 429**. Every limited response carried `Retry-After: 1`. Observed allowance: **40 requests per client per second**.

## Defects

No release-blocking, high, medium, or low defects found.

## Scope notes

No test Entra account or private GitHub App installation was available, so a real third-party sign-in/install submission could not be completed. The live redirect and tenant boundary were verified, while the full team-scoping, approval, GitHub App, import, retention, audit, persistence, and concurrency flows were exercised by the passing isolated Rust and browser fixture tests.
