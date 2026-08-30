# Diff Gate verification 22 handoff — FAIL

- **Candidate:** `52b389fd8f0b4886021b8fa46dc196dfc3addaf0`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Verified:** 2026-08-30 UTC

## Result

**FAIL — do not release.** The live URL serves the candidate build, but the
backend has a generic, stateless deployment shape and deliberately fails
closed. `/health`, `/auth/entra`, and `/api/packets` return 503; real team
review work is unavailable. A 100-request health probe found two storage IDs.
Azure shows only `PORT`, no data volume/mount, and a one-to-three replica
range. The sample demo, claims suite, local test/build/type/lint checks,
accessibility, privacy request log, offline demo, cache/security headers, and
40-then-429 rate limiter all passed.

Read the complete evidence and each individual claim result in
[`verification-22.md`](verification-22.md). Remediate the production topology
(one replica, Azure Files `/data`, SQLite and Entra contract) and re-verify
before release.

---

# Superseded: Diff Gate verification 21 repair handoff — PASS

- **Work order:** `agent-diff-gate-repair-19`
- **Failed candidate/report:** `ce5bf429b0b5bf119773fd50eee846ff69c97612` / `63c865be3b0e5e02ed2d0d28ec9a4c5404886888`
- **Repair code commit:** `bd8ea63e02339e294436063950659553d5eecc00`
- **Live URL:** <https://agent-diff-gate.sociobot.in>
- **Repaired and verified:** 2026-08-30 UTC

## Result

**PASS — the release blocker is repaired.** The live service again uses one
stateful replica, durable `/data` storage, the production public URL, and the
Sociobot Entra configuration. Real-team routes no longer fail closed.

`GET /health` returned HTTP 200 with `status: "ok"`. The public auth status
returned `service_ready:true`, `entra_sign_in_configured:true`, and
`github_app_setup_available:true`. `/auth/entra` redirected only to the
configured Sociobot tenant and used PKCE.

## Finding, reproduction, and root cause

Verification 21 found the correct `ce5bf429b0b5` image inside Azure Container
App revision `sf-agent-diff-gate--0000079`, but the deployed template had only
`PORT=8080`, no volume, no `/data` mount, and a one-to-three-replica range.
Fresh public responses reproduced the report exactly:

- `/health` returned HTTP 503 and `status:"unsafe_configuration"`.
- `/api/auth/status` returned `service_ready:false` and no Entra setup.
- `/auth/entra` returned HTTP 503.

The application correctly refused real writes under unsafe SQLite topology.
The root cause was deployment drift: a generic container release replaced the
previously verified stateful template after the earlier repair deployment.

## Repair

- Added an exact verification-21 control-plane regression fixture for revision
  `0000079` and candidate image `ce5bf429b0b5`. It asserts all ten missing
  invariants and proves the production renderer repairs them atomically.
- Pushed the repair before deployment, then ran
  `scripts/deploy-production.sh`. The script built the committed source in ACR
  and applied image, scale, storage, mount, database URL, public URL, Entra
  values, and contract version in one template patch.
- The evidence-only handoff commit is pushed before the final dedicated deploy.
  This ordering prevents a later generic image update from becoming the last
  production mutation. The live `/health` value is the authoritative final
  source identity because adding that SHA to this tracked file would change it.

## Exact production evidence

The first repaired deployment produced image
`sociobotregistry.azurecr.io/sf-agent-diff-gate:bd8ea63e0233`, digest
`sha256:f04c2bb7d018e26a3f93c92726b8f2157db50106174ba1eea32b66f9e62996d8`,
and revision `sf-agent-diff-gate--0000081`.

- `/health` → HTTP 200, build `bd8ea63e02339e294436063950659553d5eecc00`,
  storage ID `1da0c91d-ce8d-4ea1-983d-665beebfbe13`.
- `/api/auth/status` → HTTP 200 with `service_ready:true`, Entra configured,
  and GitHub App setup available.
- Anonymous `/api/packets` → HTTP 401 with the sign-in recovery message.
- Control plane → Single revision mode, minimum/maximum replicas `1/1`, Azure
  Files storage `agent-diff-gate-data-v4`, mount `/data`, database
  `sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none`, contract version `5`.
- Replacement proof → 100 concurrent health responses named one storage ID;
  a deliberate revision replacement kept that ID; the same 100-response proof
  passed after replacement.
- Rate policy before and after replacement → exactly 40 HTTP 200, then 60 HTTP
  429; every rejection included `Retry-After: 1`.
- Identity → tenant-bound `sociobotcustomers.ciamlogin.com` authorization URL,
  expected client and public callback, and `code_challenge_method=S256`.
- Response policy → documents `no-cache`; hashed JS one-year immutable; WebP
  one-hour `must-revalidate`; designed unknown routes HTTP 404 with noindex.

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

Results:

- `npm ci`: 58 packages, zero audit vulnerabilities.
- `npm test`: 11 Node unit tests and 25 Playwright tests passed.
- Every one of the 20 commands in `.factory/claims.json` passed separately.
- TypeScript and Rust formatting/clippy passed without warnings.
- All 21 Rust unit/integration tests passed; optimized release build passed.
- Production frontend: JS 22,863 B / 7.28 kB gzip; CSS 12,233 B / 3.62 kB
  gzip; image 136,640 B. All remain inside product budgets.
- PORT-only runtime check passed with build and durable-store identities.
- Package/consumer testing is not applicable to this web-with-backend
  artifact. The ACR build is the container-consumer proof and passed from the
  `.git`-free source archive using the repository Dockerfile.

## Browser, accessibility, privacy, and performance

- Dedicated live smoke passed four cold loads, desktop 1440×1000, mobile
  390×844 dark/reduced-motion, keyboard focus, `/`, `/demo`, `/privacy`,
  `/terms`, and the HTTP 404 recovery page.
- Playwright Axe reported no serious or critical issue on any checked route.
  All pages had `lang=en`, one `main`, one `h1`, route titles, no overflow, and
  no browser console or page errors.
- The loaded demo remained reviewable offline. Landing, demo review, and
  export made same-origin requests only; no analytics or third-party runtime
  request occurred.
- Worker `verify-url.sh` passed: load 599 ms, no console errors, one `h1`, one
  `main`, no missing image alt text, and no unlabeled button.
- Live mobile Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 0.9 s, LCP 1.7 s, TBT 0 ms, CLS 0.
- Live desktop Lighthouse: 100/100/100/100; FCP 0.3 s, LCP 0.4 s, TBT 0 ms,
  CLS 0.

Evidence is in `.factory/repair-19-artifacts/`.

## Known limits

No release blocker remains. This non-human worker cannot complete an
interactive Entra login or install a private GitHub App. The live tenant-only
PKCE boundary is verified; signed-in team isolation, GitHub import/setup,
policy, evidence, approval, audit, retention, and deletion are covered by the
backend integration suite.

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
DIFF_GATE_ARTIFACT_DIR=.factory/repair-19-artifacts/live \
  node scripts/live-browser-smoke.mjs https://agent-diff-gate.sociobot.in
```
