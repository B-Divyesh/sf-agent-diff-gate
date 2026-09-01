# Diff Gate first-read review 3

**Verdict: FAIL**

Reviewed 2026-09-01 at <https://agent-diff-gate.sociobot.in> in fresh Chromium contexts at 390 × 844 and 1440 × 900. The clean checkout was `46b3fcc33a141479ca949c27e684bcb1103fcde6`. Live `/health` reported build `155e6c200f3cffa3a98f904337b695571f5ba78d`; the only later commit changes factory documentation, so the shipped product files match the checkout. `.factory/brief.json` is absent. Scope checks therefore used the live product, code, README, claims, demo contract, and design document.

## Cold first read

- **Phone, before scrolling:** This checks agent-authored code changes before merge. It is for small software teams that require a named owner and test evidence. I should select **Try it with sample data** first. The complete action ends at y=589 in the 844 px viewport.
- **Desktop, before scrolling:** The same three answers are explicit. The complete action ends at y=646 in the 900 px viewport.

The exact first-screen text is “Review agent-authored changes before merge,” “For small software teams that need a required owner and test evidence before an agent-authored change lands,” and “Try it with sample data.” This check passes.

## Findings

### F-3-1 / F-2-1 / F-1-7 — BLOCKING — the live 404 still logs an error and the live check suppresses it

**Exact location:** Direct navigation to `/404` or `/missing-review-3`, at both 390 px and desktop widths.

**Observed output:** Chromium logs `Failed to load resource: the server responded with a status of 404 ()` for the document. The recovery page itself is designed, accessible, and returns HTTP 404 with `X-Diff-Gate-Route: not-found` and `X-Robots-Tag: noindex`.

**Code evidence:** `scripts/live-browser-smoke.mjs` sets `expectedNotFoundNavigation = true` immediately before the missing-route navigation and excludes the error from its console collection. The local Playwright regression uses Vite, where the unknown route returns 200, so it cannot reproduce the production response.

**Why this fails:** This is the same defect reported in rounds 1 and 2. The required no-console-errors check still fails on the deployed route, while the production check reports success by filtering the event.

**Concrete fix:** Remove the console filter and test the production serving behavior locally. If the required HTTP 404 and Chromium's document-error message cannot coexist with the zero-console gate, record an explicit approved exception and assert that one exact browser message. Do not report an unfiltered zero-error result while the filter remains.

### F-3-2 — BLOCKING — three deployment promises are not listed or proved by their claimed test

**Exact location:** README, **Deploy**.

| Exact quote | What the listed test proves | Concrete fix |
|---|---|---|
| “This command builds the image and applies the image, one-replica limit, Azure Files mount, and SQLite path in one stateful revision.” | `stateful-worker-deploy` calls a fake `deploy-production.sh` and checks its output. It never runs or inspects the production release behavior. | Add a claim entry tagged to a test that renders the production configuration and confirms the image, one replica, `/data` mount, and SQLite path together; or remove the promise. |
| “This hook routes the work order to the same stateful release and prevents a generic template from replacing it.” | The tagged test confirms delegation and rejects a wrong slug or port. It does not confirm that a later generic deployment cannot replace the stateful configuration. | Narrow the sentence to “The factory hook calls the stateful release script for this product and port,” or add a tagged replacement-regression test to the claim. |
| “The deployment ends by sending 100 concurrent health requests, replacing the revision, and repeating that probe. Both probes must report the committed build and one unchanged durable store identity.” | `runtime-port-health` checks one local process. `durable-store-replacement` checks a database reopen. Neither listed claim runs the two 100-request deployment probes or confirms the complete statement. | Add one claim and a non-mutating fixture test for both 100-request probes, revision replacement, build identity, and unchanged storage identity; or remove the statement. |

**Why this fails:** These are observable operational promises. `.factory/claims.json` has no entry whose claim and tagged test cover them. The broader unit suite contains related untagged tests, but the claims contract requires each promise to have its own listed test.

### F-3-3 — MINOR — one README heading is vague and deployment copy requires operator jargon to be decoded

No rendered landing-page or README sentence exceeds 22 words, and no banned marketing adjective appears. These remaining copy flags still require changes:

| Exact quote / location | Flag | Proposed rewrite |
|---|---|---|
| README heading: “Try it” | The pronoun does not name the section out of context. | “Try the sample review” |
| README: “This command builds the image and applies the image, one-replica limit, Azure Files mount, and SQLite path in one stateful revision.” | “Applies the image” and “stateful revision” are deployment jargon. | “This command deploys one app instance and keeps its SQLite database under `/data` on Azure Files.” |
| README: “This hook routes the work order to the same stateful release and prevents a generic template from replacing it.” | “Routes the work order” and “generic template” do not tell an operator which result to check. | “The factory hook calls this product's stateful release script and rejects the wrong product or port.” |
| README: “The deployment ends by sending 100 concurrent health requests, replacing the revision, and repeating that probe.” | “Revision” and “probe” are unexplained here. | “After deployment, the script sends 100 health requests. It replaces the app process and repeats the check.” |
| README: “Both probes must report the committed build and one unchanged durable store identity.” | “Durable store identity” requires interpretation. | “Both checks must report the committed build and the same database identity.” |

## Copy audit

Counts treat hyphenated terms, paths, environment variables, and version strings as one word. Commands and code blocks are excluded because they are not sentences. Headings, navigation, controls, labels, and image descriptions are included.

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
| Try it with sample data | 5 | result-naming action |
| Opens a sample packet with changed files, test evidence, and owner checks. | 12 | tested: `packet-export` |
| Sample data stays in this browser. | 6 | tested: `sample-sandbox` |
| Signed-in teams see only their review packets. | 7 | tested: `team-packet-boundary` |
| Export the sample packet as JSON. | 6 | tested: `packet-export` |
| Printed file sheets, a test receipt, and an approval stamp arranged as a review desk. | 15 | useful image description |
| Printed file sheets and review marks arranged across a change-control desk. | 11 | useful image description |
| CHECK | 1 | decorative art text |
| Review packet | 2 | — |
| Review a pull request | 4 | — |
| Team review | 2 | — |
| Sign in before reviewing repository changes | 6 | — |
| Packets are visible only to their signed-in team. | 8 | tested: `team-packet-boundary` |
| Sign in with Sociobot | 4 | result-naming action |
| How it works | 3 | — |
| How review packets work | 4 | — |
| Sign in. | 2 | — |
| Open your team review workspace. | 5 | instruction |
| Set repository policy. | 3 | — |
| Name sensitive paths and the required owner. | 7 | tested: `repository-policy` |
| Record evidence. | 2 | — |
| Save test evidence before the required owner approves. | 8 | tested: `named-approval` |
| What Diff Gate does not do | 6 | — |
| Diff Gate records a review decision. | 6 | tested: `no-merge-action` |
| Your team merges code outside Diff Gate. | 7 | tested: `no-merge-action` |
| Review agent-authored changes before merge. | 5 | — |
| Terms | 1 | — |
| Built by Param Factory | 4 | — |
| v0.5.0 | 1 | — |

### README

| Copy | Words | Flag |
|---|---:|---|
| Diff Gate | 2 | — |
| Review agent-authored changes before merge. | 5 | — |
| Diff Gate is for small software teams that need a required owner and test evidence before a change lands. | 19 | — |
| Try it | 2 | F-3-3 |
| Open `/?demo=1`, `/demo`, or click **Try it with sample data**. | 10 | — |
| The sample packet includes changed files, test evidence, and owner checks. | 11 | tested: `packet-export` |
| Use the banner to reset the sample or return to the real workspace. | 13 | tested: `demo-query-path` |
| Run locally | 2 | — |
| Prerequisites: Node 22+ and current stable Rust. | 7 | — |
| Visit `http://localhost:8080`. | 2 | — |
| Set `PORT` to use another port. | 6 | tested: `runtime-port-health` |
| For a local non-container run, set `DATABASE_URL=sqlite:diff-gate.db?mode=rwc` when `/data` is not writable. | 12 | instruction |
| To connect a real team workspace, set the sign-in and GitHub App variables below. | 14 | instruction |
| Deployment configuration is in `deploy/production.env.json`. | 5 | — |
| Review workflow | 2 | — |
| Each team sets sensitive paths and a required owner for each path. | 12 | tested: `repository-policy` |
| GitHub imports read every changed-file page and evaluate those paths. | 10 | tested: `github-complete-import` |
| Only the required owner can approve after the test command and result are saved. | 14 | tested: `named-approval` |
| GitHub-imported packets show the reviewed revision. | 6 | tested: `github-revision-refresh` |
| Refresh when the pull request changes. | 6 | tested: `github-revision-refresh` |
| Teams can set retention and delete a packet with its audit history. | 12 | tested: `retention-deletion` |
| Signed-in reviewers can view and export a packet's audit history. | 10 | tested: `audit-export` |
| Find claim commands in `.factory/claims.json`. | 5 | — |
| Find the demo contract in `.factory/demo.md`. | 6 | — |
| Verify | 1 | clear operator heading |
| Run the commands above before submitting a change. | 8 | instruction |
| Deploy | 1 | clear operator heading |
| Run `scripts/deploy-production.sh` from a clean, committed tree on an authenticated factory worker. | 12 | instruction |
| This command builds the image and applies the image, one-replica limit, Azure Files mount, and SQLite path in one stateful revision. | 21 | F-3-2, F-3-3 |
| The factory post-turn deploy must invoke `deploy/factory-container.sh`. | 7 | instruction |
| This hook routes the work order to the same stateful release and prevents a generic template from replacing it. | 19 | F-3-2, F-3-3 |
| The deployment ends by sending 100 concurrent health requests, replacing the revision, and repeating that probe. | 16 | F-3-2, F-3-3 |
| Both probes must report the committed build and one unchanged durable store identity. | 13 | F-3-2, F-3-3 |
| Privacy and terms | 3 | — |
| Read the in-product privacy page and terms before connecting a team. | 11 | instruction |
| The sample demo uses browser session storage only; real packets use the authenticated team workspace. | 15 | tested: `sample-sandbox`, `team-packet-boundary` |
| License | 1 | — |
| MIT | 1 | — |

Terminology is otherwise consistent: **agent-authored change**, **required owner**, **review packet**, **test evidence**, **sensitive path**, **sample demo**, and **audit history** each keep one meaning.

## Demo and sandbox

**Pass.** One click on the live first-screen action opens `/?demo=1`. At 390 × 844, the first resulting screen already shows “Add organization-level retention controls,” “Required owner: Mira Chen,” PR #482, and the hold state. The banner says “Demo — sample data, nothing is saved” and includes **Reset demo** and **Start for real**.

After one check was marked reviewed, Reset restored both owner checks. The exported JSON parsed and contained the sample title, owner, three changed files, and four checks. The loaded sample remained usable offline. Start for real removed only `demo:diff-gate`; seeded non-demo local and session values remained. The complete flow made only same-origin GET requests and made no packet API or write request. No real-storage interaction was observed.

## Claims execution

Every command listed in `.factory/claims.json` was run separately from the clean checkout after `npm ci`.

| Claim id | Result |
|---|---|
| `sample-sandbox` | PASS |
| `packet-export` | PASS |
| `demo-query-path` | PASS |
| `mobile-first-action` | PASS |
| `no-merge-action` | PASS |
| `team-packet-boundary` | PASS |
| `named-approval` | PASS |
| `entra-team-installation` | PASS |
| `github-complete-import` | PASS |
| `github-revision-refresh` | PASS |
| `github-app-provisioning` | PASS |
| `repository-policy` | PASS |
| `retention-deletion` | PASS |
| `audit-history` | PASS |
| `audit-export` | PASS |
| `no-third-party-runtime` | PASS |
| `github-file-limit` | PASS |
| `retention-limits-and-cleanup` | PASS |
| `runtime-port-health` | PASS |
| `durable-store-replacement` | PASS |
| `stateful-worker-deploy` | PASS for its narrow delegation claim; see F-3-2 for adjacent unlisted promises |

The complete clean-checkout gates also passed: `npm test` (17 Node tests and 26 browser tests), `npm run build`, `npx tsc --noEmit`, 23 Rust tests, formatting, and warning-denied Clippy. Vite produced 7.28 kB gzip JavaScript.

## Earlier finding audit

| Earlier finding | Live and code confirmation | Status |
|---|---|---|
| F-1-1 phone first action | Live action ends at y=589 on 390 × 844; its claim test passed. | Fixed |
| F-1-2 dead checkout | No checkout, plan, payment, or license-token action remains in product copy or code. | Fixed |
| F-1-3 clean-checkout runtime claim | The exact script built the absent release binary and passed. | Fixed |
| F-1-4 original unlisted promises | The exact reported promises were removed or listed. New deployment inventory gaps are F-3-2. | Fixed for the cited text |
| F-1-5 slogan headings | The cited headings are now descriptive. | Fixed |
| F-1-6 product-term drift | The four cited product terms remain consistent. | Fixed |
| F-1-7 404 console error | The live browser still emits the exact 404 resource error; the live script filters it. | **Unfixed; reopened as F-3-1** |
| F-1-8 contrast and suite failure | Full suite passed; live Axe checks found no serious or critical issue in light or dark contexts. | Fixed |
| F-2-1 404 console regression | Same live failure and filter as above. | **Unfixed; reopened as F-3-1** |
| F-2-2 export test did not inspect JSON | The tagged test now parses JSON and checks title, three files, and four checks. | Fixed |
| F-2-3 cited unlisted statements | The cited statements were removed, narrowed, or listed. New deployment statements are F-3-2. | Fixed for the cited text |
| F-2-4 approvals lacked revision binding | Backend stores the head SHA, refreshes it, clears prior evidence after a change, and blocks stale approval; the claim test passed. | Fixed |
| F-2-5 incomplete 404 structure | The live recovery view has full header/footer, canonical, icons, and social metadata. | Fixed |
| F-2-6 incomplete social cards | All checked routes have absolute Open Graph images and complete Twitter fields. | Fixed |
| F-2-7 vague copy | The exact phrases were replaced. F-3-3 lists different remaining README flags. | Fixed for the cited text |

`polish-1.md`, `polish-2.md`, and the current handoff were checked against the live site and code. The polish claim that F-2-1 had an unfiltered console test is not supported by the current live script.

## Structure, accessibility, and links

Passing checks: route-specific titles; `lang=en`; one h1 and one main; descriptions; canonicals; complete Open Graph and Twitter metadata; 1200 × 630 product artwork; SVG and Apple icons; `robots.txt`; a sitemap with all public routes; deep links; browser Back; focused h1 after route changes; polite announcements; skip links; Privacy and Terms; 44 px targets; 390 px layout; 200% text reflow; reduced-motion handling; keyboard operation; self-hosted runtime assets; no serious or critical Axe findings; and 7.28 kB gzip JavaScript.

Every discovered same-origin link was checked. Product links returned 200. `/auth/entra` returned the expected same-origin 307 sign-in handoff without following it. No dead link was found. F-3-1 is the remaining structure failure.

The cream, navy, cyan, coral, halftone print desk, clipped-paper shapes, hard shadows, and original change-control illustration match `.factory/design.md`. The result is distinct from a generic centered SaaS page.

## Missed leverage

No additional feature is clearly required. The product already supports GitHub import, manual packets, revision refresh, JSON export, audit export, retention, and team-scoped review. An AI-generated decision would weaken a workflow whose purpose is deterministic policy and named human approval, so an AI step is not warranted.

## What would make this perfect

Resolve or explicitly approve the production 404 console contradiction, add exact claim entries and tagged tests for the deployment promises, and replace the remaining vague or jargon-heavy README copy. Then repeat the complete review without filtering browser output. There is still work left, so this round cannot pass.
