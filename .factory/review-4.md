# Diff Gate first-read product QA review 4

**Verdict: PASS**

Reviewed 2026-09-01 UTC at <https://agent-diff-gate.sociobot.in> from fresh
Chromium contexts at 390 × 844 and 1440 × 900. `.factory/brief.json` is not
present, so scope was checked against the public product, README, claims,
design document, earlier reviews, polish records, and handoff.

There are **zero findings**. No blocking or minor item remains, and every
declared claim was run and passed from a clean clone.

## Cold first read

Before scrolling, on both phone and desktop, I could state:

- **What it does:** it records required-owner review and test evidence for
  agent-authored changes before merge.
- **Who it is for:** small software teams.
- **What to do first:** select **Try it with sample data**.

The exact first-screen text is “Review agent-authored changes before merge,”
“For small software teams that need a required owner and test evidence before
an agent-authored change lands,” and “Try it with sample data.” The complete
primary action ended at y=589 in the 844 px phone viewport and y=646 in the
900 px desktop viewport.

## Copy audit

Word counts treat hyphenated terms, paths, and version strings as one word.
Commands and code blocks are excluded. Product labels and controls are
included where they convey reader-facing meaning. No item exceeds 22 words;
no jargon, unexplained slogan, inconsistent product term, or non-result-naming
button requires a change.

### Landing page

| Text | Words | Result |
|---|---:|---|
| Skip to content | 3 | Clear control |
| Diff Gate | 2 | Wordmark |
| Demo | 1 | Clear navigation label |
| How it works | 3 | Clear navigation label |
| Privacy | 1 | Clear navigation label |
| Diff Gate home | 3 | Clear accessible home label |
| Review packets for agent-authored changes | 5 | Product label |
| Review agent-authored changes before merge | 5 | Plain job headline |
| For small software teams that need a required owner and test evidence before an agent-authored change lands. | 17 | Audience and outcome |
| Try it with sample data | 5 | Result-naming action |
| Opens a sample packet with changed files, test evidence, and owner checks. | 12 | Claim-backed action note |
| Sample data stays in this browser. | 6 | Claim-backed privacy fact |
| Signed-in teams see only their review packets. | 7 | Claim-backed privacy fact |
| Export the sample packet as JSON. | 6 | Claim-backed product fact |
| CHECK | 1 | Decorative artwork label; not required reading |
| Review packet | 2 | Product label |
| Review a pull request | 4 | Clear section heading |
| Team review | 2 | Clear sign-in panel label |
| Sign in before reviewing repository changes | 6 | Clear instruction |
| Packets are visible only to their signed-in team. | 8 | Claim-backed privacy fact |
| Sign in with Sociobot | 4 | Result-naming action |
| How review packets work | 4 | Clear section heading |
| Sign in. | 2 | Workflow step |
| Open your team review workspace. | 5 | Workflow step |
| Set repository policy. | 3 | Workflow step |
| Name sensitive paths and the required owner. | 7 | Claim-backed workflow detail |
| Record evidence. | 2 | Workflow step |
| Save test evidence before the required owner approves. | 8 | Claim-backed workflow detail |
| What Diff Gate does not do | 6 | Clear boundary heading |
| Diff Gate records a review decision. | 6 | Claim-backed boundary |
| Your team merges code outside Diff Gate. | 7 | Claim-backed boundary |
| Review agent-authored changes before merge. | 5 | Footer product description |
| Terms | 1 | Clear footer link |
| Built by Param Factory | 4 | Attribution |
| v0.5.0 | 1 | Build label |

### README

| Text | Words | Result |
|---|---:|---|
| Diff Gate | 2 | Product name |
| Review agent-authored changes before merge. | 5 | Plain job statement |
| Diff Gate is for small software teams that need a required owner and test evidence before a change lands. | 19 | Audience and outcome |
| Try the sample review | 4 | Clear heading |
| Open `/?demo=1`, `/demo`, or click **Try it with sample data**. | 10 | Clear instruction |
| The sample packet includes changed files, test evidence, and owner checks. | 11 | Claim-backed sample detail |
| Use the banner to reset the sample or return to the real workspace. | 13 | Claim-backed instruction |
| Run locally | 2 | Clear heading |
| Prerequisites: Node 22+ and current stable Rust. | 7 | Clear prerequisite |
| Visit `http://localhost:8080`. | 2 | Clear instruction |
| Set `PORT` to use another port. | 6 | Claim-backed configuration detail |
| For a local non-container run, set `DATABASE_URL=sqlite:diff-gate.db?mode=rwc` when `/data` is not writable. | 12 | Clear local instruction |
| To connect a real team workspace, set the sign-in and GitHub App variables below. | 14 | Clear setup instruction |
| Deployment configuration is in `deploy/production.env.json`. | 5 | Clear file reference |
| Review workflow | 2 | Clear heading |
| Each team sets sensitive paths and a required owner for each path. | 12 | Claim-backed workflow detail |
| GitHub imports read every changed-file page and evaluate those paths. | 10 | Claim-backed workflow detail |
| Only the required owner can approve after the test command and result are saved. | 14 | Claim-backed workflow detail |
| GitHub-imported packets show the reviewed revision. | 6 | Claim-backed workflow detail |
| Refresh when the pull request changes. | 6 | Clear instruction |
| Teams can set retention and delete a packet with its audit history. | 12 | Claim-backed workflow detail |
| Signed-in reviewers can view and export a packet's audit history. | 10 | Claim-backed workflow detail |
| Find claim commands in `.factory/claims.json`. | 5 | Clear file reference |
| Find the demo contract in `.factory/demo.md`. | 6 | Clear file reference |
| Verify | 1 | Clear heading |
| Run the commands above before submitting a change. | 8 | Clear instruction |
| Deploy | 1 | Clear heading |
| Run `scripts/deploy-production.sh` from a clean, committed tree on an authenticated factory worker. | 12 | Clear instruction |
| The production release template sets one app replica, the `/data` Azure Files mount, and the SQLite database path together. | 19 | Claim-backed deployment detail |
| The factory hook calls this product's release script only for `agent-diff-gate` on port `8080`. | 14 | Claim-backed deployment detail |
| Release verification checks 100 health responses before and after it replaces the app process. | 14 | Claim-backed deployment detail |
| Every response must report the committed build and the same database identity. | 12 | Claim-backed deployment detail |
| Privacy and terms | 3 | Clear heading |
| Read the in-product privacy page and terms before connecting a team. | 11 | Clear instruction |
| The sample demo uses browser session storage only; real packets use the authenticated team workspace. | 15 | Claim-backed data-handling detail |
| License | 1 | Clear heading |
| MIT | 1 | License label |

Terminology is consistent: **agent-authored change**, **required owner**,
**review packet**, **test evidence**, **sensitive path**, **sample demo**, and
**audit history** each have one meaning.

## Demo and sandbox

**PASS.** The first-screen action opens `/?demo=1` in one selection. Its first
phone screen already contains the packet title “Add organization-level
retention controls,” required owner “Mira Chen,” the pull-request context, and
the review state. The persistent banner reads “Demo — sample data, nothing is
saved” and provides **Reset demo** and **Start for real**.

Reset restored the shipped required checks after a demo change. Start for real
returned to `/`, removed the banner, and left the fresh context with no
demo-state keys. The direct demo flow issued same-origin GET requests only;
there were no console messages or page errors. The clean-clone browser suite
also confirms the isolated namespace, reset behavior, export content,
offline reviewability after loading, and removal of sample state on exit.

## Claims and local verification

Fresh clone: `/tmp/agent-diff-gate-review-4-CF0KlK`, created from the reviewed
checkout. After `npm ci`, every command listed in `.factory/claims.json` was
run separately and passed:

| Claim id | Result |
|---|---|
| sample-sandbox | PASS |
| packet-export | PASS |
| demo-query-path | PASS |
| mobile-first-action | PASS |
| no-merge-action | PASS |
| team-packet-boundary | PASS |
| named-approval | PASS |
| entra-team-installation | PASS |
| github-complete-import | PASS |
| github-revision-refresh | PASS |
| github-app-provisioning | PASS |
| repository-policy | PASS |
| retention-deletion | PASS |
| audit-history | PASS |
| audit-export | PASS |
| no-third-party-runtime | PASS |
| github-file-limit | PASS |
| retention-limits-and-cleanup | PASS |
| runtime-port-health | PASS |
| durable-store-replacement | PASS |
| stateful-worker-deploy | PASS |
| production-stateful-template | PASS |
| deployment-health-replacement | PASS |

The additional clean-clone gates passed: `npm test` (17 Node checks, build,
27 browser checks), `cargo test --all-targets` (24 tests), `cargo fmt --all
-- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and
`npx tsc --noEmit`. The production frontend build is 7.28 kB gzip JavaScript.

Every live claim-like product statement was checked against `claims.json`.
Operational statements have a matching listed claim; audience descriptions
and setup instructions do not make an additional measurable promise.

## Earlier-review confirmation

Read in full: `review-1.md`, `review-2.md`, `review-3.md`, `polish-1.md`,
`polish-2.md`, `polish-3.md`, and the prior handoff. Each earlier finding was
checked against both current live behavior and the current code/tests.

| Earlier finding | Current confirmation | Status |
|---|---|---|
| F-1-1 | The full sample action is in the first 390 × 844 screen; its claim passed. | Fixed |
| F-1-2 | No plan, payment, or checkout action remains. | Fixed |
| F-1-3 | The declared runtime command builds a missing release binary and passes from the clean clone. | Fixed |
| F-1-4 | Earlier public promises were removed, narrowed, or given matching listed claims. | Fixed |
| F-1-5 | Landing headings describe their sections. | Fixed |
| F-1-6 | The product terminology is consistent. | Fixed |
| F-1-7 / F-2-1 / F-3-1 | A browser navigation gets the noindex recovery view with `X-Diff-Gate-Route: not-found` and no console message; a non-navigation request correctly gets HTTP 404. | Fixed |
| F-1-8 | The complete suite and current Axe checks pass. | Fixed |
| F-2-2 | The export claim parses downloaded JSON and checks title, files, and checks. | Fixed |
| F-2-3 | The cited public and README statements are now claim-backed or neutral instruction text. | Fixed |
| F-2-4 | Revision refresh clears prior evidence and blocks stale approval; the claim passed. | Fixed |
| F-2-5 | The recovery document has the public header, metadata, icons, and footer. | Fixed |
| F-2-6 | All public routes declare absolute Open Graph and Twitter image metadata. | Fixed |
| F-2-7 / F-3-3 | The cited vague headings and README language are replaced with clear labels and operational text. | Fixed |
| F-3-2 | The three deployment statements now have their own direct, passing claim tests. | Fixed |

## Structure, links, accessibility, and identity

**PASS.** `/`, `/demo`, `/privacy`, `/terms`, and a direct unknown route each
have a route-specific title, one h1, `lang=en`, main landmark, description,
canonical URL, and absolute social image fields. The recovery route uses
“Not found — Diff Gate,” h1 “Page not found,” the noindex header, and the
same navigation/footer skeleton without a browser console message.

The sitemap, robots file, favicon, Apple touch icon, and social image returned
200. Public routes returned 200; the same-origin sign-in handoff returned its
expected 307. The unknown route returned HTTP 404 for a non-navigation request.
No dead link was found. Browser checks cover deep links, back navigation,
h1 focus on navigation, keyboard use, 44 px touch targets, 200% phone text,
reduced motion, and serious/critical Axe results.

The cream paper, navy ink, cyan, coral, halftone print art, clipped-note
shapes, and editorial hierarchy match `.factory/design.md`. This is a distinct
change-control visual system rather than a generic product template.

## Missed leverage

No missing feature requires a finding. The product already supports the
implied GitHub import, revision refresh, manual evidence, JSON export, audit
export, retention, and team-scoped review. A generated review conclusion
would not strengthen a workflow based on deterministic policy and named human
approval, so an AI step is not expected here.

## What would make this perfect

Keep the current claim inventory, independent demo checks, recovery-route
contract, and plain-language copy under regression coverage as the product
changes. No product change is requested by this review.
