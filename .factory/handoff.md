# Diff Gate verification 22 repair handoff — PASS

- **Work order:** `agent-diff-gate-repair-20`
- **Failed candidate/report:** `52b389fd8f0b4886021b8fa46dc196dfc3addaf0` / `6dc47e6eaf9ccd98932b4981f3b55fa704b543c9`
- **Repair code commit:** `72ee70f7f72c717301694f20fe1d22371d345eb9`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Repaired and verified:** 2026-08-30 UTC

## Result

**PASS — the release blocker is repaired.** Production uses one running
replica, durable Azure Files storage at `/data`, the required SQLite URL, and
the Sociobot Entra contract. `/health`, `/auth/entra`, and `/api/packets` no
longer fail closed.

## Reproduction and root cause

Before any change, revision `sf-agent-diff-gate--0000084` ran candidate image
`52b389fd8f0b` with only `PORT=8080`, no volume or mount, and a one-to-three
replica range. The failure reproduced exactly:

- `/health`, `/auth/entra`, and `/api/packets` returned HTTP 503.
- `/api/auth/status` returned `service_ready:false` and no Entra setup.
- A load probe scaled the revision to three running replicas. A fresh set of
  100 successful health responses was entirely `unsafe_configuration` and
  exposed three different storage IDs.

The application correctly refused real work. The root cause was deployment
drift: the generic container release had replaced the stateful template with a
stateless multi-replica template.

## Repair

- Added the exact verification-22 Azure fixture: candidate image, revision,
  PORT-only environment, no volume or mount, and one-to-three scale.
- Added a reusable 100-response health assertion. The regression rejects the
  observed three storage identities and accepts only one healthy build and
  storage identity.
- Strengthened the live gate to require exactly one **running** Azure replica,
  in addition to the one-replica template and request-level storage proof.
- Built the committed source in ACR, rendered image plus stateful settings in
  one patch, and ran a deliberate revision replacement.

## Exact production evidence

The first repaired deployment built image
`sociobotregistry.azurecr.io/sf-agent-diff-gate:72ee70f7f72c`, digest
`sha256:3ab03421aea336f67d5ac7f57b30f5d1eed81b801aa8fbfa8b6b02e87f192f6f`.
Its post-replacement revision was `sf-agent-diff-gate--0000086`.

- `/health` → HTTP 200, build `72ee70f7f72c717301694f20fe1d22371d345eb9`,
  storage ID `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- `/api/auth/status` → HTTP 200 with `service_ready:true`, Entra configured,
  and GitHub App setup available.
- `/auth/entra` → HTTP 307 to the configured
  `sociobotcustomers.ciamlogin.com` tenant, public HTTPS callback, and PKCE
  `S256`.
- Anonymous `/api/packets` → HTTP 401 with the sign-in recovery message. It no
  longer returns a storage-configuration 503.
- Control plane → Single revision mode, scale `1/1`, exactly one running
  replica, Azure Files `agent-diff-gate-data-v4` mounted at `/data`, and
  `sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none`.
- Storage replacement → the same storage ID survived a deliberate new
  revision. Both 100-response probes returned one identity; the final probe
  returned 100×200, one build, and one storage ID.
- Rate policy before and after replacement → exactly 40 accepted requests and
  60 HTTP 429 responses; every rejection included `Retry-After: 1`.

The evidence-only handoff commit is pushed before the final dedicated stateful
deployment. This keeps the repository deployment, rather than a generic image
update, as the last production mutation. Live `/health` is the authoritative
final source identity because adding that SHA here would change it again.

## Clean local verification

From a clean `npm ci`:

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

- `npm ci`: 58 packages, zero audit vulnerabilities.
- `npm test`: 12 Node unit tests and 25 Playwright tests passed.
- All 20 commands in `.factory/claims.json` passed independently.
- TypeScript, Rust formatting, and clippy passed with no warnings.
- All 21 Rust tests and the optimized release build passed.
- Production assets: JS 22,863 B (7.28 kB gzip), CSS 12,233 B (3.62 kB
  gzip), and hero WebP 136,640 B.
- The PORT-only runtime contract passed with build and durable-store IDs.
- Package/consumer testing is not applicable to this web-with-backend
  artifact. The `.git`-free ACR container build passed.

## Browser, accessibility, privacy, and performance

- Live checks passed four cold loads plus desktop 1440×1000 and mobile
  390×844 dark/reduced-motion views.
- Keyboard focus, 200% text, 44px targets, route history, reset, export,
  approval, offline demo use, and recovery routes passed.
- Playwright Axe found no serious or critical issue on `/`, `/demo`,
  `/privacy`, `/terms`, or the HTTP 404 page in either theme.
- The live sample flow made same-origin requests only. No analytics or
  third-party runtime script was observed.
- Factory `verify-url.sh` passed in 582 ms with no console errors, one `h1`,
  one `main`, complete image alt text, and no unlabeled button.
- Live Lighthouse mobile: 100 Performance, 100 Accessibility, 100 Best
  Practices, 100 SEO; FCP 0.9 s, LCP 1.7 s, TBT 10 ms, CLS 0.
- Live Lighthouse desktop: 100/100/100/100; FCP 0.3 s, LCP 0.4 s, TBT 0 ms,
  CLS 0.
- Response policy passed: documents `no-cache`; hashed JS one-year immutable;
  WebP one-hour `must-revalidate`; unknown routes HTTP 404 with noindex.

Screenshots, Lighthouse reports, and factory URL evidence are in
`.factory/repair-20-artifacts/`.

## Known limits

No release blocker remains. A non-human worker cannot finish an interactive
Entra login or install a private GitHub App. The live tenant-only PKCE boundary
is verified. Team isolation, GitHub import/setup, repository policy, evidence,
approval, audit, retention, and deletion are covered by backend integration
tests.

## Re-run

```sh
npm ci
npm test
npx tsc --noEmit
npm run build
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
./scripts/verify-runtime-contract.sh
./scripts/verify-live-deployment.sh https://agent-diff-gate.sociobot.in
DIFF_GATE_ARTIFACT_DIR=.factory/repair-20-artifacts \
  node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
```
