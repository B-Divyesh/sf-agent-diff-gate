# Diff Gate repair 9 handoff — PASS

**Repair candidate:** `e5b8efcca693ce94fdba0e0e02ccc0b1c4a3ae8a`

**Repaired from:** verifier report `03dca0a5162b3c71bfe740917424298fc0612dae` for candidate `076df6c3aaf53de4b8aae83f07de857c29bfa001`

**Live URL:** <https://agent-diff-gate.sociobot.in>

## Release blockers repaired

1. **Durable production state.** The deployment helper returned while Azure still reported the Container App as `InProgress`. The immediate stateful-template patch then failed, leaving the helper's three-replica, container-local defaults in production. `deploy-production.sh` now waits for provisioning before and after every template mutation, sets single-revision mode, mounts the existing Azure Files share at `/data`, points SQLite at that mount, and holds scale at exactly one replica.
2. **Real missing-route status.** The Axum fallback and `/404` now return the styled recovery view with HTTP `404`, `X-Diff-Gate-Route: not-found`, and `X-Robots-Tag: noindex`. The fallback still preserves the accessible route back to Diff Gate.

The source regression checks cover both arbitrary missing paths and `/404`. The live browser and deployment checks reject a `200` recovery response. The deployment verifier checks single revision mode, one replica, the Azure Files volume and mount, a `/data` SQLite URL, and durable identity across a forced revision replacement.

## Deployment evidence

- ACR run `chye` built the repair image successfully in 7m46s.
- `/health` reports build `e5b8efcca693ce94fdba0e0e02ccc0b1c4a3ae8a` and storage identity `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- Azure reports provisioning `Succeeded`, active revision mode `Single`, revision `sf-agent-diff-gate--0000044`, `minReplicas: 1`, `maxReplicas: 1`, Azure Files volume `agent-diff-gate-data-v4`, and `/data` mounted in `app`.
- `DATABASE_URL` is `sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none`.
- `verify-live-deployment.sh ... --replace` created a replacement revision and confirmed the storage identity stayed `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- The Sociobot Entra redirect uses the production callback and PKCE S256. Anonymous protected API requests remain blocked.
- A live burst of 80 requests from one forwarded client produced 40 `200` and 40 `429` responses. Every limited response had `Retry-After: 1`.

## Verification evidence

All commands passed from the repaired checkout:

```sh
npm ci                              # 58 packages, 0 vulnerabilities
npm test                            # 24 Playwright tests
npx tsc --noEmit
npm run build                       # dist/ produced
cargo fmt --check
cargo test                          # 20 tests
cargo clippy -- -D warnings
cargo build --release
./scripts/verify-runtime-contract.sh
# Every exact test command in .factory/claims.json (20/20)
```

The production bundle is 22.50 kB JavaScript (7.19 kB gzip), 12.23 kB CSS (3.62 kB gzip), and a 136,640-byte hero image. The ACR build is the container/package-consumer gate for this web-with-backend artifact; there is no separate library package.

Live checks passed:

```sh
/opt/fleet/lib/verify-url.sh https://agent-diff-gate.sociobot.in <evidence-dir>
node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
npm run test:live-identity -- https://agent-diff-gate.sociobot.in
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in
```

- Desktop and 390 x 844 mobile flows passed with keyboard operation, visible focus, no horizontal overflow, reduced motion, same-origin-only demo traffic, and offline interaction after load.
- Every public route and the recovery view had one h1, one main landmark, route metadata, and zero serious or critical Axe findings. `verify-url.sh` found no console errors, missing alt text, or unnamed buttons.
- Mobile Lighthouse: performance 100, accessibility 100, best practices 100, SEO 100; FCP 0.95 s, LCP 1.70 s, TBT 33 ms, CLS 0, 173,857 bytes transferred.
- HTML is `no-cache`; hashed assets are immutable; the stable hero is revalidated hourly. HSTS, `nosniff`, strict-origin referrer policy, and CSP with `frame-ancestors 'none'` are present.
- Local and live SHA-256 hashes match for JavaScript (`a25ba899…`), CSS (`f17d531…`), and the hero (`b9f805e8…`).
- No service worker is registered and no offline-reload claim is made. The loaded demo remains usable offline; fresh HTML revalidates, so updates are not trapped behind a stale application shell.

Artifacts: [`repair-9-artifacts/verify-url/verify.json`](repair-9-artifacts/verify-url/verify.json), [`repair-9-artifacts/live/live-desktop.png`](repair-9-artifacts/live/live-desktop.png), [`repair-9-artifacts/live/live-mobile.png`](repair-9-artifacts/live/live-mobile.png), and [`repair-9-artifacts/lighthouse-live.json`](repair-9-artifacts/lighthouse-live.json).

## Known limitations

No test member account or private GitHub organization was available. The production tenant, callback, PKCE, and anonymous boundaries were verified live; authenticated team isolation, GitHub import, approval, audit, retention, deletion, persistence, and concurrency remain covered by the passing fixture-backed integration tests.

No release-blocking finding remains. The researched scope, visual system, sample workflow, and previously passing behavior were not changed.
