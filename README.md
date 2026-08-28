# Diff Gate

Review agent changes before merge. Diff Gate is for 3–30 person software teams that need a named owner and review evidence before an agent-authored change lands.

It creates a review packet that makes changed contracts, migrations, tests, risky paths, and owner checks visible in one place. It does not write code or merge pull requests.

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

For frontend iteration, run `npm run dev` and visit the address Vite prints.

## Verify

```sh
npm test
cargo test
npm run build
docker build --build-arg BUILD_SHA=dev -t diff-gate .
docker run --rm -p 8080:8080 diff-gate
```

The browser tests include the observable demo privacy boundary and JSON packet export claims. See `.factory/claims.json` and `.factory/demo.md`.

## Deploy

The root `Dockerfile` builds the Vite frontend and Rust server. The image listens on `PORT=8080`, has no required environment variables, and serves `/health` with the build SHA. Mount `/data` for durable packet storage.

## Privacy and billing

Sample packets stay in the browser. Saved packets contain the values a reviewer submits. The team plan is billed through Sociobot checkout; it never embeds a payment provider. Read the in-product `/privacy` and `/terms` pages before connecting a team.
