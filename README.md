# Diff Gate

Review agent changes before merge. Diff Gate is for 3–30 person software teams that need a named owner and review evidence before an agent-authored change lands.

It imports a team-bound GitHub App pull request into a review packet. Each repository has its own sensitive-path policy and required owner. The packet records every changed-file page, server-recorded test evidence, and the named owner's approval. It does not write code or merge pull requests.

## Try it

Open `/demo` after starting the app, or use the live demo at `https://agent-diff-gate.sociobot.in/demo`. The demo is sample-only. Its session storage is cleared when demo mode ends.

## Run locally

Prerequisites: Node 22+ and Rust 1.88+.

```sh
npm ci
npm run build
cargo run
```

Visit `http://localhost:8080`. The server uses `PORT` (default `8080`) and creates a SQLite database under `/data`. For a local non-container run, set `DATABASE_URL=sqlite:diff-gate.db?mode=rwc` if `/data` is not writable.

The sample demo works with no configuration. Real review packets use only Sociobot Entra External ID and a team-bound GitHub App installation. The approved Sociobot tenant and public SPA client are non-secret deployment settings. Sign-in uses authorization-code PKCE, so Diff Gate does not store an Entra client secret.

```sh
ENTRA_AUTHORITY=https://sociobotcustomers.ciamlogin.com/<tenant> \
ENTRA_TENANT_ID=... ENTRA_CLIENT_ID=... \
ENTRA_TEAM_CLAIM=oid \
GITHUB_APP_ID=... GITHUB_APP_PRIVATE_KEY='-----BEGIN...\\n...' \
GITHUB_TEAM_INSTALLATIONS='{"entra:<team-id>":"<installation-id>"}' GITHUB_APP_SLUG=... \
PUBLIC_BASE_URL=https://your-host cargo run
```

Production defaults to the approved Sociobot tenant and client recorded in `deploy/production.env.json`. Set `ENTRA_TEAM_CLAIM` to an assigned shared-team claim when the tenant issues one; the approved deployment uses the stable Entra `oid` as an isolated team workspace. Only the exact Sociobot tenant on `sociobotcustomers.ciamlogin.com` is accepted. The service validates issuer, tenant, audience, nonce, signing algorithm, and signing key before creating a session.

After sign-in, a team can create its own private GitHub App from the real-work panel. GitHub's App Manifest flow returns a generated App identity and key directly to the backend. The App requests read-only repository contents and pull-request access. Its installation id is verified against that App before being bound to the Entra team. `GITHUB_APP_*` and `GITHUB_TEAM_INSTALLATIONS` remain available for an administrator-provisioned shared App. An unmapped team cannot import. Imports read every changed-file page and reject pull requests above 10,000 files.

Before an import, a signed-in team saves a policy for the exact `owner/repository`: one sensitive path rule per line and the person required to approve it. For example, `schema/** | database-owner@example.com`. Imports refuse repositories without a policy, apply only that repository’s rules, and set the matching rule’s required owner as the packet owner. A pull request matching multiple owners is rejected so the team can split the change or align its policy.

Test evidence is not a client-controlled checkbox. The service stores a command, result, signed-in actor, and server timestamp. A client can send a `done` state, but the backend replaces it with the incomplete evidence check until it receives a non-empty command and result.

Each signed-in team can set packet retention from 1 to 3,650 days; the default is 90 days. Expired packets and their audit history are removed according to that setting before packet reads. A reviewer can also delete a packet and its history immediately. The packet view lists team-scoped audit entries and includes them in the JSON export.

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

The browser tests cover demo isolation, demo data egress, JSON export, keyboard use, mobile layout, and both color schemes. The Rust suite covers team isolation, named-owner approval, multi-page GitHub import, retention, audit history, response policy, and Sociobot authority restrictions. See `.factory/claims.json` and `.factory/demo.md`.

## Plans

Diff Gate costs **$12 per developer per month** or **$99 per team per month**. The landing page sends checkout and license verification only to the Sociobot billing API; it does not include a payment-provider key. Sociobot is the merchant of record. The free sample, export, privacy controls, and accessibility controls remain available without a paid plan. Buyers can return with `?license=<token>` or restore the token through the plan section.

## Deploy

The root `Dockerfile` builds the Vite frontend and Rust server. The image starts with `PORT` only and `/health` returns the build SHA plus a non-secret durable store identity. Mount `/data` for durable packet storage.

Run `scripts/deploy-production.sh` from an authenticated factory worker. It reapplies the approved public Entra settings after the container helper, mounts the product's Azure Files `/data` share, forces one replica, and verifies the Entra callback plus a real revision replacement with the same durable store identity. Production uses SQLite's single-process `unix-none` VFS because Azure Files does not provide SQLite byte-range locking. Administrator-provisioned `GITHUB_APP_PRIVATE_KEY` values must use a Key Vault secret reference. The team self-provisioning flow stores GitHub's generated private App key only in the team-scoped backend database.

## Privacy

Sample packets stay in the browser until demo mode ends. Saved packets contain the values a reviewer submits and changed file paths returned by GitHub. Team retention and explicit deletion remove packet audit history too. The sample has no analytics or third-party runtime scripts. Read the in-product `/privacy` and `/terms` pages before connecting a team.
