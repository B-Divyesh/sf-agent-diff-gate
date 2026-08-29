# Independent verification 17 — FAIL

**Candidate:** `cfdd80845d42ebe477b3b51664eb41a5ab48fc68`
**URL:** <https://agent-diff-gate.sociobot.in>
**Verified:** 2026-08-29 UTC
**Result:** **FAIL — release blocking production deployment defect.**

## Blocking finding

### Critical — production does not have the required durable single-instance SQLite topology

The live process serves the exact candidate code, but its Azure Container App configuration is unsafe for this stateful service.

Fresh non-mutating control-plane inspection of `sf-agent-diff-gate` found:

```json
{
  "latestRevisionName": "sf-agent-diff-gate--0000067",
  "image": "sociobotregistry.azurecr.io/sf-agent-diff-gate:cfdd80845d42",
  "scale": { "minReplicas": 1, "maxReplicas": 3 },
  "volumes": null,
  "volumeMounts": null,
  "env": ["PORT"]
}
```

`./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in '' cfdd80845d42ebe477b3b51664eb41a5ab48fc68` failed before making any change. It reported all ten required production invariants missing: exactly one replica; the `agent-diff-gate-data-v4` Azure Files volume and `/data` mount; durable `DATABASE_URL`; the public base and Sociobot Entra variables; and deployment contract version.

The runtime failure is observable, not just configuration drift: 100 concurrent `/health` requests all returned build `cfdd80845d42ebe477b3b51664eb41a5ab48fc68` but returned **three** `storage_id` values:

```text
47127349-a1ce-4324-a1f5-02706b2e6682
70e41620-ca1c-4af3-b67f-0197f0990004
b16957d7-58e4-4cac-b510-d1d678548c9d
```

That means different replicas have independent ephemeral SQLite stores. Team packets and audit history can diverge or disappear, contrary to the product’s retention, audit, and team-workspace contract. This is release blocking.

**Required remediation:** deploy only through `scripts/deploy-production.sh` (or an equivalent template that meets `deploy/production-contract.mjs`): one replica, Azure Files `agent-diff-gate-data-v4` mounted at `/data`, the durable SQLite URL, all production Entra/public configuration, and `DEPLOYMENT_CONFIG_VERSION`. Then rerun the no-replacement live verifier and a 100-request health identity probe.

## Candidate and deployment identity

- Local checkout was clean and exactly the requested commit.
- Live `GET /health` returned HTTP 200 with build `cfdd80845d42ebe477b3b51664eb41a5ab48fc68`.
- The live JavaScript and CSS SHA-256 values exactly matched local production `dist/` assets. The candidate is therefore deployed; this is not a stale-site false failure.

## First-read and demo result

**PASS.** In a cold desktop browser the first screen says “Review agent-authored changes before merge,” identifies “small software teams” as the audience, and offers the one-click **Try it with sample data** action with an adjacent explanation of what opens. It plainly answers what it does, for whom, and what to click first.

The live demo passed a full representative workflow at 390 px: launch sample, resolve both required checks, export `diff-gate-packet.json` (3 changed files, 4 checks), approve, reload with the approval retained, reset, and leave demo with its `demo:diff-gate` session storage cleared. There were no console or page errors and every recorded request was same-origin.

The authenticated real-team workflow, real GitHub import, and Entra callback completion could not be exercised without a test tenant/team, but its integration claim tests all pass. Anonymous `/api/packets` correctly returned 401, and `/auth/entra` redirected only to `sociobotcustomers.ciamlogin.com` with PKCE S256.

## Mandatory claims

**PASS — all 20 commands in `.factory/claims.json` passed from the clean checkout.** This includes each demo browser claim, all 12 backend selectors, the runtime-port health contract, and the durable-storage reopen selector.

Browser claim evidence included sample isolation/reset, JSON packet export, mobile first action, no merge action, audit export, and same-origin-only demo traffic. Backend claim evidence included team isolation, required-owner approval/evidence, Entra and GitHub installation scoping, changed-file pagination/limit, revision refresh, repository policy, retention/deletion, concurrent approval conflict, and storage reopen.

## Local quality gates

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 58 packages, 0 vulnerabilities |
| `npm test` | PASS — 7 Node deployment tests and 24 Playwright tests |
| `npx tsc --noEmit` | PASS |
| `npm run build` | PASS — JS 22.50 kB (7.19 kB gzip), CSS 12.23 kB (3.62 kB gzip) |
| `cargo fmt --all -- --check` | PASS |
| `cargo test --all` | PASS — 20 tests |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `cargo build --release` | PASS |
| `./scripts/verify-runtime-contract.sh` | PASS |
| Docker production build | **Not run:** `docker` is not installed in this verification container. |

The Dockerfile was statically reviewed: it is multi-stage, uses `rust:1-alpine` without a minor pin, accepts `BUILD_SHA=dev`, avoids `.git`, runs non-root, and exposes 8080. This does not substitute for an image build.

## Live browser, privacy, accessibility, and headers

**PASS, except for the deployment topology above.**

- `/opt/fleet/lib/verify-url.sh` passed: HTTP 200, title, `lang=en`, one `h1`, a `main` landmark, complete image alt text, and no browser errors.
- `scripts/live-browser-smoke.mjs` passed desktop and 390×844 mobile, light and dark treatment, keyboard focus, reduced motion, all public routes, designed HTTP 404 recovery, offline-after-load demo use, and zero axe serious/critical violations.
- A fresh outgoing-request log during the demo workflow contained only the product origin. No analytics or third-party runtime script was observed.
- `node deploy/live-rate-limit.mjs` passed: one client received 40 accepted requests then 60 HTTP 429 responses, each with `Retry-After: 1`.
- Root and API responses include `nosniff`, strict-origin referrer policy, HSTS, and a self-contained CSP with `frame-ancestors 'none'`. The root document and anonymous packet API response use `no-cache`; hashed JS/CSS assets use `public, max-age=31536000, immutable`.

## Severity summary

| Severity | Finding |
| --- | --- |
| Critical | Live candidate runs up to three replicas with no durable volume/mount; concurrent health checks prove three independent stores. |
| Verification gap | Docker is unavailable in this container, so the exact Docker image build was not independently executed. |

No product source files were modified during this verification.
