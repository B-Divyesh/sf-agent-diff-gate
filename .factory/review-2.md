# Diff Gate adversarial first-read review 2

**Verdict: FAIL**

Reviewed 2026-08-29 at <https://agent-diff-gate.sociobot.in> from fresh Chromium contexts at 390 × 844 and 1440 × 900. The reviewed live build and clean-clone commit were both `9e09b906d76c410d8bc4f4367706af4ae6996650`. `.factory/brief.json` is absent, so product-scope checks used the shipped copy, code, README, and `.factory/design.md`.

## Cold first read

- **Phone, before scrolling:** This reviews changes written by coding agents before merge; it is for small software teams that require an owner and test evidence; click **Try it with sample data** first. The complete button ended at y=589 in the 844 px viewport.
- **Desktop, before scrolling:** The same three answers are explicit. The primary action ended at y=624 in the 900 px viewport.

This gate passes. The exact first-screen copy was “Review agent-authored changes before merge”, “For small software teams that need a required owner and test evidence before an agent-authored change lands”, and “Try it with sample data”.

## Findings

### F-2-1 / F-1-7 — BLOCKING — the 404 console error has regressed

**Exact location:** Direct navigation to `/missing-review-2` at both tested widths.

**Observed output:** `Failed to load resource: the server responded with a status of 404 ()`.

**Why this fails:** F-1-7 reported this exact defect. Polish 1 marked it fixed, but its artifact recorded the missing route as HTTP 200. Commit `88498f7` restored a real HTTP 404 and `scripts/live-browser-smoke.mjs` now deliberately suppresses the resulting console error with `expectedNotFoundNavigation`. The live browser still reports the error, so this is not genuinely fixed and the no-console-errors gate is not met.

**Concrete fix:** Do not suppress this event in verification. Use a serving strategy that retains a real missing-route contract without a browser console error, then add a direct-navigation regression assertion over the unfiltered console stream. If the platform makes those requirements incompatible, record and approve a narrow exception instead of reporting zero console errors.

### F-2-2 — BLOCKING — the JSON export claim test does not inspect JSON

**Exact location:** `.factory/claims.json`, claim `packet-export`; `tests/diff-gate.spec.ts:61`.

**Quote:** “Exports the review packet as JSON.”

**Why this fails:** The declared test asserts only the filename and that `createReadStream()` returns an object. Empty, malformed, or unrelated bytes would pass. An independent live download did parse successfully and contained the sample title, 3 changed files, and 4 checks, but the required repeatable claim test does not establish that result.

**Concrete fix:** Read the download, `JSON.parse` it, and assert the sample packet title plus the expected changed files and checks. Keep that assertion under exactly `@claim:packet-export`.

### F-2-3 — BLOCKING — claim-like README and live statements are absent from `claims.json`

The following statements are user-reliable promises but have no claim entry that names and tests the complete statement.

| Exact quote / location | Gap | Concrete fix |
|---|---|---|
| Home: “Sociobot Entra identifies the reviewer.” | `entra-team-installation` checks allowed configuration and team installation mapping, not a completed identity callback that identifies the reviewer. | Add a claim entry and callback fixture that verifies the displayed reviewer identity, or rewrite to the tested team-isolation statement. |
| Home, How it works: “Start a team review workspace.” | No listed browser or backend claim proves that a first sign-in creates/opens the workspace. | Add a claim and authenticated first-use test, or say “Open your team’s review workspace” if creation is not part of this product. |
| README, Run locally: “The server uses `PORT` (default `8080`) and creates its SQLite database under `/data`.” | `runtime-port-health` supplies port 18080; it does not verify the 8080 default or assert the database path. | Add a configuration test for both defaults and list it as a claim, or remove the untested defaults from the sentence. |
| README, Verify: “`npm test` runs the browser demo, routing, accessibility, keyboard, mobile-layout, privacy, and export checks.” | This broad suite-scope claim is not listed. The current suite passed, but claims inventory is incomplete. | Add a claim entry tied to `npm test`, or replace the sentence with a neutral instruction to run it. |
| README, Verify: “Rust tests cover authenticated team isolation, policy evaluation, evidence and approval, retention, audit history, GitHub integration boundaries, response headers, and rate limiting.” | Several items have claim entries, but response headers and rate limiting do not. | List the omitted promises with their exact tests, or narrow the sentence to the listed claims. |
| README, Deploy: “The root `Dockerfile` builds the Vite frontend and Rust server.” and “It deploys the container, applies the product's durable `/data` mount, and runs the production verification script.” | Neither deployment behavior is in the claims inventory, and no clean-sandbox claim command exercises the deployment script. | Add a non-mutating Docker/deployment contract test and claim entry, or describe the files without promising execution behavior. |

### F-2-4 — BLOCKING — approvals are not bound to a pull-request revision

**Exact location:** GitHub import and approval flow; `backend/src/main.rs` `GithubPull`, `import_github_pr`, and `Packet`.

**Observed behavior:** Import stores the PR title, URL, author, changed paths, and policy result. It does not read or retain the head commit SHA. Approval does not re-check GitHub. A PR can change after import while the saved packet still presents the old path list as approvable.

**Why this fails:** A normal reviewer expects an approval gate to identify the exact code revision that was reviewed. Without a revision, the retained decision can be mistaken for approval of later agent-authored code.

**Concrete fix:** Import and display the PR head SHA, bind evidence and approval to it, provide **Refresh from GitHub**, and mark prior evidence/approval stale when the head changes. Add a claim test where GitHub returns a new head SHA before approval and verify approval is blocked until the packet is refreshed and re-reviewed.

### F-2-5 — MINOR — the dedicated 404 omits required route metadata and header content

**Exact location:** `/missing-review-2`, served from `frontend/public/404.html`.

**Observed:** The route has no canonical, Open Graph fields, Twitter card, or apple-touch icon. Its header also drops **How it works**, while all application routes include it.

**Why this fails:** The route does not use the same complete metadata and header skeleton as the rest of the product.

**Concrete fix:** Add the 404 canonical/social metadata and apple-touch icon, and use the same header navigation as the other public routes. Add the standalone 404 to the metadata and header-consistency tests.

### F-2-6 — MINOR — social image metadata is not a complete absolute card declaration

**Exact location:** `/`, `/demo`, `/privacy`, and `/terms` head metadata.

**Observed:** `og:image` is `/social.webp`, not an absolute URL. Only `twitter:card` is declared; `twitter:title`, `twitter:description`, and `twitter:image` are absent.

**Why this fails:** Social parsers are being asked to infer missing Twitter fields and resolve a relative Open Graph URL. The structure contract asks for explicit title, description, and the 1200 × 630 image.

**Concrete fix:** Emit `https://agent-diff-gate.sociobot.in/social.webp` and explicit Twitter title, description, and image fields on every public route; test their live values.

### F-2-7 — MINOR — two headings and several README phrases are vague or jargon-heavy

| Exact quote / location | Problem | Proposed rewrite |
|---|---|---|
| Home action note: “Opens a complete review packet.” | “Complete” is an unmeasured adjective and does not name the sample contents. | “Opens a sample packet with changed files, test evidence, and owner checks.” |
| 404 h1: “This review desk is empty” | Metaphor heading; it does not name the error out of context. | “Page not found” |
| README: “configure the approved Sociobot Entra values and GitHub App values for your deployment.” | “approved” and “values” are vague; “Entra” is unexplained setup jargon. | “Set the sign-in and GitHub App variables below for your deployment.” |
| README: “Each team sets repository-sensitive paths and their required owners.” | “repository-sensitive paths” drifts from the established term “sensitive paths”. | “Each team sets sensitive paths and a required owner for each path.” |
| README: “Only the required owner can approve after saved test command and result evidence.” | Dense noun stack; “result evidence” is not the established term. | “Only the required owner can approve after the test command and result are saved.” |
| README: “Rust tests cover authenticated team isolation, policy evaluation, evidence and approval, retention, audit history, GitHub integration boundaries, response headers, and rate limiting.” | “authenticated team isolation” and “integration boundaries” require rereading. | “Rust tests check team privacy, owner approval, GitHub imports, retention, audit history, headers, and rate limits.” |
| README: “It deploys the container, applies the product's durable `/data` mount, and runs the production verification script.” | “applies the durable mount” is operator jargon. | “It deploys the container, keeps the `/data` database between releases, and verifies the live service.” |

No sentence exceeds 22 words. No banned marketing word appears. All buttons otherwise use result-naming verbs, and the remaining section headings name their sections.

## Demo and sandbox

**Pass.** One click from the live home opened `/?demo=1`. The initial 390 × 844 screen already showed the sample title “Add organization-level retention controls”, owner “Mira Chen”, PR number, and hold status. The persistent banner said “Demo — sample data, nothing is saved” and exposed **Reset demo** and **Start for real**.

After one **Mark reviewed**, Reset restored both required-owner checks. Start for real removed `demo:diff-gate` while preserving seeded non-demo local- and session-storage values. The complete flow made only same-origin GET requests for the document, assets, art, and `/api/auth/status`; it made no packet API request and no write request. The loaded demo remained usable offline. Evidence: `review-2-artifacts/demo-live.json` and `live-demo-mobile.png`.

## Claims execution

Every listed command was run from clean clone `9e09b906d76c410d8bc4f4367706af4ae6996650` after `npm ci`.

| Claim | Declared command result |
|---|---|
| `sample-sandbox` | PASS |
| `packet-export` | PASS command; test adequacy fails F-2-2 |
| `demo-query-path` | PASS |
| `mobile-first-action` | PASS |
| `no-merge-action` | PASS |
| `team-packet-boundary` | PASS |
| `named-approval` | PASS |
| `entra-team-installation` | PASS |
| `github-complete-import` | PASS |
| `github-app-provisioning` | PASS |
| `repository-policy` | PASS |
| `retention-deletion` | PASS |
| `audit-history` | PASS |
| `audit-export` | PASS |
| `no-third-party-runtime` | PASS |
| `github-file-limit` | PASS |
| `retention-limits-and-cleanup` | PASS |
| `runtime-port-health` | PASS; built its missing release binary itself |
| `durable-store-replacement` | PASS |

The broader clean-clone gates also passed: `npm test` (21/21 Playwright), `npm run build`, `cargo fmt --check`, and `cargo clippy -- -D warnings`. Production output was 6.99 kB gzip JavaScript.

## Copy audit

Counts treat hyphenated terms, paths, variables, and version strings as one word. Commands and code blocks are excluded. Headings, navigation, labels, controls, and accessible image descriptions are included because the review also requires those phrases to be checked.

### Landing page

| Copy | Words | Flag |
|---|---:|---|
| Skip to content | 3 | — |
| Diff Gate | 2 | — |
| Demo | 1 | — |
| How it works | 3 | — |
| Privacy | 1 | — |
| Diff Gate home | 3 | — |
| Review packets for agent-authored changes | 5 | — |
| Review agent-authored changes before merge | 5 | — |
| For small software teams that need a required owner and test evidence before an agent-authored change lands. | 17 | — |
| Try it with sample data | 5 | — |
| Opens a complete review packet. | 5 | F-2-7 |
| Sample data stays in this browser. | 6 | tested |
| Signed-in teams see only their review packets. | 7 | tested |
| Export the sample packet as JSON. | 6 | test gap F-2-2 |
| Printed file sheets, a test receipt, and an approval stamp arranged as a review desk. | 15 | — |
| Printed file sheets and review marks arranged across a change-control desk. | 11 | — |
| CHECK | 1 | decorative image text, not required copy |
| Review packet | 2 | — |
| Review a pull request | 4 | — |
| Team review | 2 | — |
| Sign in before reviewing repository changes | 6 | — |
| Sociobot Entra identifies the reviewer. | 5 | unlisted claim F-2-3 |
| Packets are visible only to that reviewer’s team. | 8 | tested |
| Sign in with Sociobot | 4 | — |
| How it works | 3 | — |
| How review packets work | 4 | — |
| Sign in. | 2 | — |
| Start a team review workspace. | 5 | unlisted claim F-2-3 |
| Set repository policy. | 3 | — |
| Name sensitive paths and the required owner. | 7 | tested |
| Record evidence. | 2 | — |
| Save test evidence before the required owner approves. | 8 | tested |
| What Diff Gate does not do | 6 | — |
| Diff Gate records a review decision. | 6 | tested |
| Your team merges code outside Diff Gate. | 7 | tested |
| Review agent-authored changes before merge. | 5 | — |
| Privacy | 1 | — |
| Terms | 1 | — |
| Built by Param Factory | 4 | — |
| v0.5.0 | 1 | — |

### README

| Copy | Words | Flag |
|---|---:|---|
| Diff Gate | 2 | — |
| Review agent-authored changes before merge. | 5 | — |
| Diff Gate is for small software teams that need a required owner and test evidence before a change lands. | 19 | — |
| Try it | 2 | — |
| Open `/?demo=1`, `/demo`, or click **Try it with sample data**. | 10 | — |
| The sample opens a complete review packet in an isolated browser session. | 12 | “complete”; same issue as F-2-7 |
| The banner can reset the sample or return to the real workspace. | 12 | tested |
| Run locally | 2 | — |
| Prerequisites: Node 22+ and current stable Rust. | 7 | — |
| Visit `http://localhost:8080`. | 2 | — |
| The server uses `PORT` (default `8080`) and creates its SQLite database under `/data`. | 13 | unlisted claim F-2-3 |
| For a local non-container run, set `DATABASE_URL=sqlite:diff-gate.db?mode=rwc` if `/data` is not writable. | 12 | — |
| To connect a real team workspace, configure the approved Sociobot Entra values and GitHub App values for your deployment. | 19 | jargon F-2-7 |
| The deployment configuration lives in `deploy/production.env.json`. | 6 | — |
| Review workflow | 2 | — |
| Each team sets repository-sensitive paths and their required owners. | 9 | terminology F-2-7 |
| GitHub imports read every changed-file page and evaluate those paths. | 10 | tested |
| Only the required owner can approve after saved test command and result evidence. | 13 | jargon F-2-7 |
| Teams can set retention and delete a packet with its audit history. | 12 | tested |
| Signed-in reviewers can view and export a packet's audit history. | 10 | tested |
| The full list of tested product claims is in `.factory/claims.json`. | 10 | contradicted by F-2-3 |
| The demo contract is in `.factory/demo.md`. | 6 | — |
| Verify | 1 | — |
| `npm test` runs the browser demo, routing, accessibility, keyboard, mobile-layout, privacy, and export checks. | 14 | unlisted claim F-2-3 |
| Rust tests cover authenticated team isolation, policy evaluation, evidence and approval, retention, audit history, GitHub integration boundaries, response headers, and rate limiting. | 22 | jargon and unlisted claim F-2-3/F-2-7 |
| Deploy | 1 | — |
| The root `Dockerfile` builds the Vite frontend and Rust server. | 10 | unlisted claim F-2-3 |
| Run `scripts/deploy-production.sh` from an authenticated factory worker. | 7 | — |
| It deploys the container, applies the product's durable `/data` mount, and runs the production verification script. | 16 | jargon and unlisted claim F-2-3/F-2-7 |
| Privacy and terms | 3 | — |
| Read the in-product privacy page and terms before connecting a team. | 11 | — |
| The sample demo uses browser session storage only; real packets use the authenticated team workspace. | 15 | tested |
| License | 1 | — |
| MIT | 1 | — |

## Earlier finding audit

| Earlier finding | Live and code verification | Status |
|---|---|---|
| F-1-1 mobile first action | Button ends at y=589 on live 390 × 844; `@claim:mobile-first-action` passed. | Fixed |
| F-1-2 dead checkout | No checkout, plan, license, or billing URL remains in the live UI, README, or frontend source. | Fixed |
| F-1-3 clean-clone runtime claim | `./scripts/verify-runtime-contract.sh` built the absent release binary and passed from the clean clone. | Fixed |
| F-1-4 unlisted promises | The exact promises reported in review 1 were removed or listed. New inventory gaps are F-2-3. | Fixed for the cited text |
| F-1-5 slogan headings | The cited landing headings are now descriptive. The 404 metaphor is separately reported in F-2-7. | Fixed for the cited headings |
| F-1-6 terminology drift | “agent-authored change”, “required owner”, “review packet”, and “test evidence” are consistent in product copy. README has the smaller F-2-7 wording issue. | Fixed for the cited terms |
| F-1-7 404 console error | A real HTTP 404 now renders the designed page, but the exact console error is back. | **Reopened as F-2-1 / F-1-7** |
| F-1-8 contrast/test failure | Full local suite passed 21/21; live Axe found no serious or critical issues at either width. | Fixed |

`polish-1.md` and the current handoff were also checked. The polish assertion for F-1-7 relied on a 200 response; the later 404 correction invalidated that closure.

## Structure, accessibility, and links

Passing checks: route-specific titles; `lang=en`; one h1 and one main per route; descriptions and canonical URLs on application routes; 1200 × 630 original social image; SVG favicon and 180 × 180 apple icon; designed 404 with a route home; deep links; browser Back; h1 focus after navigation; polite announcer; skip link; Privacy and Terms; 44 px controls; no 390 px overflow at normal or 200% text; reduced-motion path; self-hosted assets; no serious/critical live Axe findings; and no generic SaaS layout.

The dithered print desk, cream/navy/cyan/coral palette, hard offset shadows, editorial hierarchy, and original change-control illustration match `.factory/design.md` and are visually distinct.

All discovered home/header/footer links were crawled. Product routes returned 200. `/auth/entra` returned a 307 to the configured Sociobot tenant and the destination returned 200. No dead link was found. F-2-1, F-2-5, and F-2-6 record the remaining structure failures. Evidence: `review-2-artifacts/structure-live.json`.

## Missed leverage

F-2-4 is the missed high-value control: bind each decision to the exact GitHub head revision and invalidate it when the PR changes. No AI feature is warranted. The job is human accountability and deterministic policy enforcement; generated review conclusions would weaken that boundary. GitHub import and JSON export already exist.

## What would make this perfect

Remove the real 404 console regression without suppressing it; make the export claim test parse and inspect the file; list or remove every remaining promise; bind approvals to a PR head SHA with refresh and stale-state handling; complete the 404/social metadata; and replace the remaining metaphor, vague adjective, and setup jargon. Re-run the full adversarial checklist after those changes. There is still work left, so the verdict cannot be PASS.
