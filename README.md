# Diff Gate

Review agent changes before merge. Diff Gate is for 3–30 person software teams that need a named owner and review evidence before an agent-authored change lands.

It imports an installed GitHub App's pull requests into a review packet. The packet records changed paths, default contract/migration checks, test evidence, and the named owner's approval. It does not write code or merge pull requests.

## Try it

Open `/demo` after starting the app, or use the live demo at `https://agent-diff-gate.sociobot.in/demo`. The demo is sample-only and does not save a packet.

## Run locally

Prerequisites: Node 22+ and Rust 1.88+.

```sh
npm ci
npm run build
cargo run
```

Visit `http://localhost:8080`. The server uses `PORT` (default `8080`) and creates a SQLite database under `/data`. For a local non-container run, set `DATABASE_URL=sqlite:diff-gate.db?mode=rwc` if `/data` is not writable.

The sample demo works with no configuration. Real review packets require GitHub identity and an installed GitHub App:

```sh
GITHUB_OAUTH_CLIENT_ID=... GITHUB_OAUTH_CLIENT_SECRET=... \
GITHUB_APP_ID=... GITHUB_APP_PRIVATE_KEY='-----BEGIN...\\n...' \
GITHUB_APP_INSTALLATION_ID=... GITHUB_APP_SLUG=... \
PUBLIC_BASE_URL=https://your-host cargo run
```

The OAuth app requests `read:user` and `read:org` to identify a reviewer and their active organization. Repository reads use a short-lived GitHub App installation token; configure the App with read-only pull-request and contents permissions for the repositories that team explicitly installs. Packets are scoped to the signed-in GitHub organization; a user without one receives a private workspace.

For frontend iteration, run `npm run dev` and visit the address Vite prints.

## Verify

```sh
npm test
cargo test
cargo clippy -- -D warnings
npm run build
docker build --build-arg BUILD_SHA=dev -t diff-gate .
docker run --rm -p 8080:8080 diff-gate
```

The browser tests include the observable demo privacy boundary and JSON packet export claims. See `.factory/claims.json` and `.factory/demo.md`.

## Deploy

The root `Dockerfile` builds the Vite frontend and Rust server. The image listens on `PORT=8080`, has no required environment variables, and serves `/health` with the build SHA. Mount `/data` for durable packet storage.

## Privacy

Sample packets stay in the browser. Saved packets contain the values a reviewer submits and changed file paths returned by GitHub. The site has no analytics or third-party runtime scripts. Read the in-product `/privacy` and `/terms` pages before connecting a team.
