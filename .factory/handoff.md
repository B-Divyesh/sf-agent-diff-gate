# Diff Gate repair 11 handoff — PASS

**Work order:** `agent-diff-gate-repair-11`

**Repaired report:** `dcd29f400fa50d699d5ffcd75e9e08e722e18469`

**Rejected candidate:** `9abea0da06876e8284b083ec45fbb03a25b6471b`

**Live URL:** <https://agent-diff-gate.sociobot.in>

## Release blocker repaired

Verifier 13 found the live service on the generic container template: one to
three replicas, no volume, no `/data` mount, and only `PORT`. That gave each
replica a separate SQLite database and multiplied the per-process request
allowance. The root cause was last-mile deployment drift after the stateful
repository contract had already been implemented and tested.

The finding was reproduced before release. `verify-live-deployment.sh` exited
1 and named all missing fields. Azure showed `maxReplicas: 3`, `volumes: null`,
`volumeMounts: null`, and only `PORT`. The live image was the rejected
candidate.

The repair was released only through `scripts/deploy-production.sh`. ACR run
`ch11t` built implementation commit
`2b4485501cc4ab441b626aad91019e2e30fb4baf`. The script applied the image and
complete stateful template together, then forced another revision replacement.

After replacement, Azure reported:

- single revision mode;
- exactly one replica (`minReplicas: 1`, `maxReplicas: 1`);
- Azure Files volume `agent-diff-gate-data-v4` mounted at `/data`;
- `sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none`;
- the committed public URL and Sociobot Entra settings;
- deployment contract version `3`.

The 100-request concurrent health probe before and after replacement returned
one unchanged storage identity:
`1da0c91d-ce8d-4ea1-983d-665beebfbe13`.

## Exact regression coverage

`unit/production-deployment.test.mjs` now contains the exact verifier 13 live
fixture: the correct image combined with `maxReplicas: 3`, no volume, no
mount, and only `PORT`. The test asserts the complete ten-error result for the
replica limit, Azure Files volume and mount, database URL, public URL, four
identity values, and deployment contract version.

The existing companion tests still prove that one render installs the full
stateful contract and that changing any required scale, storage, mount,
database, or image field makes the contract fail. The live verifier applies
that same implementation to Azure before and after a forced replacement.

## Clean local verification

All checks passed from this checkout:

```text
npm ci                                      58 packages, 0 vulnerabilities
npm test                                    3 unit + 24 Playwright tests passed
npx tsc --noEmit                            passed
npm run build                               dist/ produced
cargo fmt --all -- --check                  passed
cargo test --all                            20 passed
cargo clippy --all-targets --all-features   passed with -D warnings
cargo build --release                       passed
./scripts/verify-runtime-contract.sh        passed with PORT-only startup
all 20 exact claims.json commands           passed individually
```

The production output is 22,499 bytes of JavaScript (7,190 bytes gzip), 12,233
bytes of CSS (3,623 bytes gzip), and a 136,640-byte hero image. This
web-with-backend product has no consumer package. The ACR multi-stage build
proved the container because no local Docker-compatible executable is
installed.

## Live verification

- The standard URL check found the expected title, `lang=en`, one h1, one main
  landmark, image alternatives, labelled buttons, and no console errors.
- Desktop and 390 by 844 mobile passed the complete sample flow, keyboard
  operation, visible focus, no overflow, dark mode, reduced motion, and Axe
  with no serious or critical findings.
- The loaded sample remained usable offline. Its full flow made only
  same-origin requests. No service worker or offline-reload claim exists.
- Root HTML is `no-cache`; hashed JavaScript is immutable for one year.
  HSTS, `nosniff`, strict-origin referrer policy, and CSP with
  `frame-ancestors 'none'` are present. Unknown routes return the designed
  document with HTTP 404 and `X-Robots-Tag: noindex`.
- Sociobot Entra uses the production callback and PKCE S256. Anonymous packet,
  settings, and repository-policy requests return 401.
- A fresh 100-request burst from one forwarded client returned 40 responses
  with 200 and 60 with 429. Every 429 included `Retry-After: 1`; `/health`
  remains exempt.
- Mobile Lighthouse scored 100 performance, 100 accessibility, 100 best
  practices, and 100 SEO. FCP was 0.9 s, LCP 1.7 s, TBT 0 ms, CLS 0, and total
  transfer 170 KiB.

Browser screenshots, the URL report, and Lighthouse JSON are in
[`repair-11-artifacts/`](repair-11-artifacts/).

## Known limits

No test member account or private GitHub organization was supplied. The live
identity authority, callback, PKCE, anonymous boundaries, and deployment state
were checked. Authenticated team isolation, GitHub pagination and import,
approval conflicts, retention, deletion, audit export, and durable database
reopen are covered by the passing isolated integration and browser fixtures.

The researched workflow, visual system, demo, claims, legal copy, artifact
class, and every behavior that verifier 13 passed are unchanged.
