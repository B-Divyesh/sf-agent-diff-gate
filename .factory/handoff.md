# Diff Gate repair handoff

## Repaired release blockers

- Reproduced the ACR failure before changing it with `cargo +1.86.0 build --release`. The locked ICU 2.3 packages rejected Rust 1.86 and reported that Rust 1.88 is required.
- Raised the Docker Rust build stage from `rust:1.86-alpine` to `rust:1.88-alpine`, retained the committed `Cargo.lock`, and changed the frontend stage to deterministic `npm ci`.
- Added a Rust regression test that asserts the locked ICU 2.3 dependency and the Rust 1.88 Docker build stage. The same test checks that the lockfile, non-root runtime port, and `EXPOSE 8080` contract remain present.
- Refactored the service router so all non-health routes, including frontend and packet routes, share per-forwarded-IP limiting. It uses the first `X-Forwarded-For` hop and returns `429` with `Retry-After: 1` after 40 requests in a one-second window. `/health` stays available for probes and returns the build SHA.
- Corrected startup logging so it accurately reports whether the SQLite configuration was supplied or generated; no runtime environment variable is required.
- Made the documented `demo:diff-gate` session-storage sandbox real, with persistence across reload, reset, and disposal when starting for real. Canonical URLs now follow client-side route changes.
- Added 44px navigation/review targets and a higher-contrast dark treatment. The design record now accurately states that the product uses a self-hosted system font stack.

## Verification evidence

All commands below passed from a clean dependency install:

```sh
npm ci
npx tsc --noEmit
npm test
npm run build
cargo fmt --check
cargo test
cargo clippy -- -D warnings
cargo +1.88.0 build --release
```

- `npm test`: 7 Playwright tests passed. This includes both claims, desktop keyboard review, a 390px no-overflow check, offline-after-load review, demo namespace/reset regression, and a Playwright Axe scan with zero serious or critical findings.
- `cargo test`: 3 tests passed: health/build identity, forwarded-IP rate limiting with `Retry-After`, and the Rust/ICU Docker-stage regression.
- Production Vite output: 12.35 KB JavaScript (4.69 KB gzip) and 10.34 KB CSS (3.26 KB gzip). There are no third-party runtime scripts, fonts, or analytics.
- `cargo +1.86.0 build --release` failed as expected before the repair; `cargo +1.88.0 build --release` passed after it.
- The release binary was run with `env -i` (no application configuration), listened on port 8080, returned `{"status":"ok","build":"dev"}` from `/health`, and served `/demo` with 200. A 41-request forwarded-IP smoke yielded 40 × 200 then 429 with `Retry-After: 1`.
- `/opt/fleet/lib/verify-url.sh http://127.0.0.1:8080/demo` passed: 585ms local load, zero console errors, title/lang/main/one-h1 present, and zero images missing alt text. Desktop and 390px screenshots are in the ephemeral verifier output directory used for that run.
- Local Lighthouse (mobile defaults, performance and accessibility categories) scored 100 performance and 100 accessibility against the production release server.
- ACR run `chjw` completed the real multi-stage Docker build and push in 7m4s: `sociobotregistry.azurecr.io/sf-agent-diff-gate:65f7b63166c6` at digest `sha256:4e74273d94c1acf8c8d20d668d7b9ee8e1d62d02620760145823d833ba7d802f`.

## Run and deploy

```sh
npm ci
npm run build
cargo run
# or build the production container:
docker build --build-arg BUILD_SHA=$(git rev-parse --short HEAD) -t diff-gate .
docker run --rm -p 8080:8080 diff-gate
```

The runtime needs only `PORT` (defaults to 8080); it generates its SQLite `/data/diff-gate.db` location when `DATABASE_URL` is absent. The root Dockerfile is multi-stage, runs as a non-root user, and does not depend on `.git` in its build context.

## Known deployment note

This worker has no local Docker daemon. The factory ACR build above validates the actual Docker image. The repository has no checked-in Container App deployment target, and no `agent-diff-gate` Container App exists in the configured factory subscription, so no Azure infrastructure was created or modified here. The pushed image is ready for the factory deployment target.
