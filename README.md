# Diff Gate

Review agent changes before merge. Diff Gate is for 3–30 person software teams that need a named owner and review evidence before an agent-authored change lands.

It imports a team-bound GitHub App pull request into a review packet. The packet records changed paths, default contract/migration checks, saved test evidence, and the named owner's approval. It does not write code or merge pull requests.

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

The sample demo works with no configuration. Real review packets use Sociobot Entra External ID and a team-bound GitHub App installation:

```sh
ENTRA_AUTHORITY=https://sociobotcustomers.ciamlogin.com/<tenant> \
ENTRA_CLIENT_ID=... ENTRA_CLIENT_SECRET=... \
ENTRA_TEAM_CLAIM=extension_DiffGateTeam \
GITHUB_APP_ID=... GITHUB_APP_PRIVATE_KEY='-----BEGIN...\\n...' \
GITHUB_TEAM_INSTALLATIONS='{"entra:<team-id>":"<installation-id>"}' GITHUB_APP_SLUG=... \
PUBLIC_BASE_URL=https://your-host cargo run
```

Configure the Entra application to issue the `extension_DiffGateTeam` claim (or set `ENTRA_TEAM_CLAIM` to your assigned team claim). The service validates the Entra issuer, audience, and signing key before creating a secure session. `GITHUB_TEAM_INSTALLATIONS` maps that exact claim value, prefixed with `entra:`, to one installation id; an unmapped team cannot import. Repository reads use a short-lived GitHub App installation token with only pull-request and contents read permission. Imports paginate changed files and stop safely above 10,000 files.

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

The browser tests include the observable demo privacy boundary and JSON packet export claims. The Rust suite covers team isolation, named-owner approval with durable evidence, and Entra/team installation configuration. See `.factory/claims.json` and `.factory/demo.md`.

## Deploy

The root `Dockerfile` builds the Vite frontend and Rust server. The image listens on `PORT=8080`, has no required environment variables, and serves `/health` with the build SHA. Mount `/data` for durable packet storage.

## Privacy

Sample packets stay in the browser. Saved packets contain the values a reviewer submits and changed file paths returned by GitHub. The site has no analytics or third-party runtime scripts. Read the in-product `/privacy` and `/terms` pages before connecting a team.
