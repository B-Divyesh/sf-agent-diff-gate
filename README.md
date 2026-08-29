# Diff Gate

Review agent-authored changes before merge. Diff Gate is for small software teams that need a required owner and test evidence before a change lands.

## Try it

Open `/?demo=1`, `/demo`, or click **Try it with sample data**. The sample packet includes changed files, test evidence, and owner checks. Use the banner to reset the sample or return to the real workspace.

## Run locally

Prerequisites: Node 22+ and current stable Rust.

```sh
npm ci
npm run build
cargo run
```

Visit `http://localhost:8080`. Set `PORT` to use another port. For a local non-container run, set `DATABASE_URL=sqlite:diff-gate.db?mode=rwc` when `/data` is not writable.

To connect a real team workspace, set the sign-in and GitHub App variables below. Deployment configuration is in `deploy/production.env.json`.

```sh
ENTRA_AUTHORITY=https://sociobotcustomers.ciamlogin.com/<tenant> \
ENTRA_TENANT_ID=... ENTRA_CLIENT_ID=... ENTRA_TEAM_CLAIM=oid \
GITHUB_APP_ID=... GITHUB_APP_PRIVATE_KEY='-----BEGIN...\\n...' \
GITHUB_TEAM_INSTALLATIONS='{"entra:<team-id>":"<installation-id>"}' \
GITHUB_APP_SLUG=... PUBLIC_BASE_URL=https://your-host cargo run
```

## Review workflow

- Each team sets sensitive paths and a required owner for each path.
- GitHub imports read every changed-file page and evaluate those paths.
- Only the required owner can approve after the test command and result are saved.
- GitHub-imported packets show the reviewed revision. Refresh when the pull request changes.
- Teams can set retention and delete a packet with its audit history.
- Signed-in reviewers can view and export a packet's audit history.

Find claim commands in [`.factory/claims.json`](.factory/claims.json). Find the demo contract in [`.factory/demo.md`](.factory/demo.md).

## Verify

```sh
npm ci
npm test
npm run build
cargo fmt --check
cargo test
cargo clippy -- -D warnings
./scripts/verify-runtime-contract.sh
docker build --build-arg BUILD_SHA=dev -t diff-gate .
docker run --rm -p 8080:8080 diff-gate
```

Run the commands above before submitting a change.

## Deploy

Build the image from the root `Dockerfile`. Run `scripts/deploy-production.sh` from an authenticated factory worker.

## Privacy and terms

Read the in-product [privacy page](/privacy) and [terms](/terms) before connecting a team. The sample demo uses browser session storage only; real packets use the authenticated team workspace.

## License

[MIT](LICENSE)
