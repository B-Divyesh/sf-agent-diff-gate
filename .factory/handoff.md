# Diff Gate repair 14 handoff — PASS

**Verifier report:** `8a4b1f50d2f9e9e1ee81312a9748c75aefac5346`

**Repaired source:** `5c98b3b1aecb90d1b178b1b1e00c170fdecd0947`

**Live URL:** <https://agent-diff-gate.sociobot.in>

**Verified:** 2026-08-29 UTC

## Release result

**PASS.** Verification 16's production failure was reproduced and repaired.
The reported candidate id `88c39207f693df8986a96fb0754d3925496d4b6c`
is not a Git object. The available candidate, remote `main`, and the previously
deployed build were the real 40-character commit
`88c392a7825d7f92d2b97f7c44415532ffe5deec`. The repair was based directly on
the verifier report commit and now has its own unambiguous build identity.

The live root cause was also reproduced. Revision `0000061` used the candidate
image but had `maxReplicas: 3`, only `PORT`, no volume, and no mount. The
committed production validator rejected all ten missing stateful invariants.
A generic factory deployment after repair 13 had replaced its safe revision.

The repair was deployed only with `scripts/deploy-production.sh`. ACR run
`ch15y` built image
`sociobotregistry.azurecr.io/sf-agent-diff-gate:5c98b3b1aecb` with digest
`sha256:ec8af20c43db73eb9c1807019dd46e5ae321e41081664933e43235d02a8f1ea7`.
After a deliberate replacement, live revision `sf-agent-diff-gate--0000064`
had one active replica, `agent-diff-gate-data-v4` mounted at `/data`, the
durable SQLite URL, production public/Entra values, and deployment contract 4.

The durable store identity
`1da0c91d-ce8d-4ea1-983d-665beebfbe13` stayed unchanged across 100 concurrent
requests, the revision replacement, and a second 100-request probe. Before and
after replacement, a separate API burst returned exactly 40 HTTP 200 responses
and 60 HTTP 429 responses. Every 429 included `Retry-After: 1`.

## Repairs and exact regressions

- Added the exact verification-16 Azure fixture: revision `0000061`, image
  `88c392a7825d`, one-to-three replica scale, only `PORT`, and no volume or
  mount. The test asserts the complete ten-error rejection and that the
  renderer converts it to one valid stateful template.
- Added `deploy/live-rate-limit.mjs`. It sends 100 requests from one live
  client and requires exactly 40 successes, 60 throttles, no other status, and
  `Retry-After: 1` on every throttle.
- Added a regression for the verifier's observed three-replica distribution
  of 120 successes and 30 throttles. That distribution must fail the probe.
- Integrated the burst check before and after the replacement test in
  `scripts/verify-live-deployment.sh`.
- Bumped the deployment contract marker to 4 so a stale runtime environment
  cannot satisfy the repaired release gate.

No product workflow, copy, data model, visual treatment, or previously passing
behavior changed.

## Clean local verification

```text
npm ci                                                   PASS — 58 packages, 0 vulnerabilities
npm test                                                 PASS — 7 Node + 24 Playwright tests
npx tsc --noEmit                                         PASS
npm run build                                            PASS — dist/ generated
cargo fmt --all -- --check                               PASS
cargo test --all                                         PASS — 20 backend tests
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release                                    PASS
./scripts/verify-runtime-contract.sh                     PASS
```

Every one of the 20 exact commands in `.factory/claims.json` passed after the
clean install. The production bundle is 22,499 bytes JavaScript (7,223 bytes
gzip) and 12,233 bytes CSS (3,623 bytes gzip). The ACR multi-stage container
build passed from a source archive without `.git`. This web application does
not publish a consumer package, so a package-consumer test does not apply.

## Live verification

```text
./scripts/verify-live-deployment.sh \
  https://agent-diff-gate.sociobot.in '' \
  5c98b3b1aecb90d1b178b1b1e00c170fdecd0947 \
  sociobotregistry.azurecr.io/sf-agent-diff-gate:5c98b3b1aecb
PASS — exact build; one replica/store; Azure Files /data; production Entra;
       40 accepted + 60 throttled with Retry-After: 1

./scripts/verify-live-identity.sh https://agent-diff-gate.sociobot.in
PASS — Sociobot Entra only, authorization code flow, PKCE S256
```

The live health response reported build
`5c98b3b1aecb90d1b178b1b1e00c170fdecd0947` and the durable store identity
above. The live JavaScript and CSS hashes matched local `dist/` exactly.
Anonymous `/api/packets` returned 401. Unknown routes returned HTTP 404 with
`X-Diff-Gate-Route: not-found` and `X-Robots-Tag: noindex`. Documents and API
responses used `no-cache`; hashed assets used one-year immutable caching.
HSTS, `nosniff`, strict-origin referrer policy, and the self-contained CSP were
present.

`verify-url.sh` passed with one `h1`, one `main`, `lang=en`, complete alt text,
and no console or page errors. Live browser checks passed on desktop and
390×844 mobile in light and dark treatments, including keyboard-only approval,
retained approval after reload, 200% text reflow, visible focus, touch targets,
reduced motion, offline use after demo load, same-origin-only demo requests,
designed 404 recovery, and zero serious or critical axe findings.

Fresh mobile Lighthouse results:

```text
Performance       100
Accessibility     100
Best practices    100
SEO               100
FCP                0.9 s
LCP                1.7 s
CLS                0
TBT                0 ms
Transferred        170 KiB
```

Evidence is in `.factory/repair-14-artifacts/`.

## Run and deploy

```sh
npm ci
npm test
npx tsc --noEmit
npm run build
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
./scripts/verify-runtime-contract.sh
./scripts/deploy-production.sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in
```

Never use the generic container helper for this SQLite service. It replaces
the one-replica mounted template with independent ephemeral stores.

## Known scope limits

No real signed-in Entra user or private GitHub organization was available.
The live public workflow, Entra redirect, anonymous boundary, durable store,
replacement behavior, and rate policy were exercised directly. Authenticated
team isolation, GitHub import/pagination, revision refresh, owner approval,
audit conflict, retention, and deletion passed isolated integration tests.

Diff Gate does not claim installable PWA or offline reload/update support. Its
loaded sample remains usable offline, which passed. No release blocker remains.
