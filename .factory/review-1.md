# Diff Gate adversarial first-read review 1

**Verdict: FAIL**

Reviewed 2026-08-29 at https://agent-diff-gate.sociobot.in in fresh Chromium contexts at 390 × 844 and 1440 × 960. .factory/brief.json was absent. There were no prior review-*.md or polish-*.md files; the earlier handoff was a PASS assertion rather than a finding list.

## Cold first read

Desktop, before scrolling: this is a review gate for agent-authored changes before merge, for small software teams; click the sample packet first.

Phone, before scrolling: I can identify the product and audience, but cannot see what to click first. The art is above the copy. Measured positions: art top 214 px, h1 top 583 px, and “Try it with sample data” top 854 px; the 390 px viewport ends at 844 px.

## Findings

### F-1-1 — BLOCKING — the phone first screen hides the required first action

**Location / quote:** Home at 390 px: “Try it with sample data” starts at y=854 px. The visible first screen ends with “For small software teams who need an owner and evidence before an agent-made change lands.”

**Why:** A cold visitor cannot answer “what should I click first?” without scrolling. CSS explicitly puts .hero-art before copy on small screens.

**Fix:** Put the h1, one-sentence description, and “Try it with sample data — Opens a complete review packet” before art at ≤700 px. Add a 390 × 844 assertion that the full primary action is inside the initial viewport.

### F-1-2 — BLOCKING — the paid-plan action is dead; its claim test checks only a string

**Location / quote:** “Choose a Sociobot plan (opens checkout)” and README “Plans cost $12 per developer monthly or $99 per team monthly and use Sociobot checkout.”

**Evidence:** A normal GET to https://api.sociobot.in/api/v1/products/agent-diff-gate/checkout returns 404 with {"error":"enabled factory product","status":404}. The claimed browser test passes because it asserts the literal href, not checkout behavior.

**Why:** The only purchase action promises checkout but opens an error. This is a dead link and makes the listed billing claim false in production.

**Fix:** Provision the product or use the working Sociobot checkout endpoint. Change the claim test to follow the anchor in a safe fixture or assert a successful checkout redirect/result, rather than matching its URL.

### F-1-3 — BLOCKING — a declared claim command fails from a clean checkout

**Location / quote:** claims.json runtime-port-health: “The container starts with PORT only and /health returns its build and durable store identities.” Test: ./scripts/verify-runtime-contract.sh.

**Evidence:** From this clean checkout the command exited 1 because its first operation requires target/release/diff-gate, which does not exist. It gives no useful diagnostic. After cargo build --release, the same script passes.

**Why:** The claims contract says every declared command runs from a clean clone. This one has an undeclared build prerequisite.

**Fix:** Make the script build the release binary when absent, or declare one self-contained command: cargo build --release && ./scripts/verify-runtime-contract.sh. Test the failure message too.

### F-1-4 — BLOCKING — claim-like promises are not all listed and tested

**Why:** Claims.json must cover every user-reliable promise. The following live/README sentences have no exact claim entry and observable test. A broader, related test does not prove these exact statements.

| Location | Exact unlisted claim | Concrete fix |
| --- | --- | --- |
| Home caption | “Every packet names an owner and records review evidence.” | Add a create/render test for both requirements, or make it sample-specific. |
| Home plans | “Sociobot bills the plan; Diff Gate never receives a payment card.” | Add a checkout/request-log proof or remove it. |
| Home boundary | “It does not merge code for you.” | Add an import/approval no-merge-request test or remove it. |
| Home boundary | “Diff Gate keeps security findings advisory.” | Define and test this behavior or delete it. |
| Home boundary | “Your team decides what to change and who approves it.” | Replace with the existing tested owner-approval rule. |
| README opening | “It does not write code or merge pull requests.” | Add a no-write/no-merge test or delete it. |
| README run | “The sample demo works with no configuration.” | Add a clean-start demo test. |
| README identity | “Sign-in uses authorization-code PKCE, so Diff Gate does not store an Entra client secret.” | Add redirect/configuration and secret-absence tests or remove it. |
| README identity | “The service validates issuer, tenant, audience, nonce, signing algorithm, and signing key before creating a session.” | Add one fixture rejection test per named condition and list the claim. |
| README GitHub | “GitHub's App Manifest flow returns a generated App identity and key directly to the backend.” | Add a mocked flow proving the key never reaches the browser. |
| README GitHub | “Its installation id is verified against that App before being bound to the Entra team.” | Add a mismatched-App-installation rejection test. |
| README policy | “A pull request matching multiple owners is rejected so the team can split the change or align its policy.” | Add a two-rule import fixture and assert the stated error. |
| README evidence | “The service stores a command, result, signed-in actor, and server timestamp.” | Assert all four fields in named-approval and list the exact claim. |
| README privacy | “Saved packets contain the values a reviewer submits and changed file paths returned by GitHub.” | Add a collection/data-shape privacy test or reduce to proven fields. |
| README plans | “The free sample, export, privacy controls, and accessibility controls remain available without a paid plan.” | Add a plan-state browser test for each stated control, or remove the bundle claim. |
| README plans | “Buyers can return with ?license=<token> or restore the token through the plan section.” | Add the URL recovery flow to the billing claim test. |
| README deploy | “The team self-provisioning flow stores GitHub's generated private App key only in the team-scoped backend database.” | Add a storage-boundary test proving the key is not returned or stored elsewhere. |
| Privacy | “The team-bound GitHub App reads only pull requests your team can access.” | Add an inaccessible-repository authorization fixture. |
| Terms | “Diff Gate records review evidence.” | Replace with a precise existing claim or test it. |

### F-1-5 — MINOR — headings/slogan do not name their section

**Location / quote:** “Live review desk”, “Find the merge blockers first”, “Make the review decision visible”, “Pay for team review”, and footer “Diff Gate makes change ownership visible.”

**Why:** In a heading list, these are mood/outcome phrases, not section names. “Live” also overstates the signed-out panel.

**Fix:** Use “Review a pull request”, “How review packets work”, and “Plans and billing”. Delete the footer slogan or use the product one-line description.

### F-1-6 — MINOR — terms drift for the same concept

**Location / quote:** Home says “agent changes” and “agent-made change”; README says “agent-authored change”. The product says “owner”, “Accountable owner”, “named owner”, and “required owner” without defining different roles.

**Why:** A first-time reviewer cannot tell whether these are separate workflow roles or inconsistent wording.

**Fix:** Use this terminology everywhere: agent-authored change; required owner (the person allowed to approve); review packet; test evidence. Rewrite the home lede: “For small software teams who need a required owner and test evidence before an agent-authored change lands.”

### F-1-7 — MINOR — the designed 404 produces a console error

**Location / evidence:** Direct navigation to /does-not-exist renders “This review desk is empty”, but Chromium logs “Failed to load resource: the server responded with a status of 404” for the document; /404.html behaves the same.

**Why:** The visual 404 exists, but the load fails the no-console-errors quality gate.

**Fix:** Serve a standalone styled 404 response without booting a failed SPA document, or adjust hosting routing. Add a direct-404 console assertion.

### F-1-8 — BLOCKING — the local quality suite has repeatable serious contrast failures

**Location / evidence:** `npm test` reproducibly ends **18 passed, 1 failed**. The local light-mode Axe test reports three serious WCAG 2 AA contrast failures on `/demo`: both **“Mark reviewed”** buttons render `#716f6a` on `#f7db91` at **3.70:1**; **“Export packet”** renders `#7e8181` on `#f6f1e6` at **3.48:1**. The required ratio is 4.5:1.

**Why:** This fails the stated accessibility baseline and prevents `npm test` from passing. A lone rerun of the test may pass, but two complete suite runs failed at the same test and selectors; that makes the quality gate unreliable as well as inaccessible in the failing rendering state.

**Fix:** Set explicit foreground/background values for `.small-button` and `.secondary` that meet 4.5:1 in every color scheme, then make the Axe test deterministic by waiting for the final styles before analysis. Run the complete suite repeatedly, not only the isolated test.

## Demo and sandbox

**Pass.** One click on “Try it with sample data” opened /demo directly into the realistic “Add organization-level retention controls” packet: three paths, a migration of 14,382 rows, test evidence, and two owner checks. The persistent “Demo — sample data, nothing is saved” banner was present. Reset demo restored two checks after one was reviewed. Start for real returned home and removed sessionStorage demo:diff-gate.

In a fresh context, every demo request was same-origin (document, self-hosted JS/CSS/art, and /api/auth/status); no analytics or third-party runtime request appeared. Demo state used only sessionStorage key demo:diff-gate, no local-storage key, no packet API request, and the loaded demo remained operable offline.

## Claims execution

Passed commands: sample-sandbox, packet-export, team-packet-boundary, named-approval, entra-team-installation, github-complete-import, github-app-provisioning, repository-policy, retention-deletion, audit-history, audit-export, no-third-party-runtime, sociobot-billing, github-file-limit, retention-limits-and-cleanup, and durable-store-replacement.

runtime-port-health failed from the clean checkout as F-1-3 records, then passed only after manually building release. The sociobot-billing test passes but does not refute F-1-2 because it never opens checkout.

## Copy audit

Counts include headings, labels, buttons, and sentence copy; commands, code blocks, and environment examples are excluded. No prose sentence exceeds 22 words except the README deployment sentence explicitly marked below.

### Landing copy inventory

- Diff Gate (2); Demo (1); How it works (3); Plans (1); Privacy (1).
- Accountable review for agent changes (5, F-1-6); Review agent changes before merge (5).
- For small software teams who need an owner and evidence before an agent-made change lands. (15, F-1-6)
- Try it with sample data (5, clear result verb but F-1-1); Opens a complete review packet. (5).
- Sample data stays in this browser. (6, sample-sandbox); Sociobot sign-in limits packets to one team. (7, team boundary); $12 per developer monthly or $99 per team monthly. (9, billing).
- Every packet names an owner and records review evidence. (9, F-1-4).
- Live review desk (3, F-1-5); Find the merge blockers first (5, F-1-5); Team review (2).
- Sign in before reviewing repository changes (6); Sociobot Entra identifies the reviewer. (5); Packets are visible only to that reviewer’s team. (8).
- Sign in with Sociobot (4); Make the review decision visible (5, F-1-5).
- Sign in. (2); Sociobot Entra identifies the reviewer and team. (7); Set repository policy. (3); Name sensitive paths and the owner each path needs. (9, F-1-6); Record the decision. (3); Save test evidence and retain the named approval. (8, F-1-6).
- Pay for team review (4, F-1-5); $12 per developer each month or $99 per team each month. (11); Sociobot bills the plan; Diff Gate never receives a payment card. (11, F-1-4).
- Choose a Sociobot plan (opens checkout) (6, F-1-2); Restore a paid plan (4).
- It does not merge code for you (7, F-1-4); Diff Gate keeps security findings advisory. (6, F-1-4); Your team decides what to change and who approves it. (10, F-1-4).
- Diff Gate makes change ownership visible. (6, F-1-5); Privacy / Terms / Built by Param Factory / v0.4.0 (7).

### README copy inventory

- Review agent changes before merge. (5). Diff Gate is for 3–30 person software teams that need a named owner and review evidence before an agent-authored change lands. (21, F-1-6).
- It imports a team-bound GitHub App pull request into a review packet. (11). Each repository has its own sensitive-path policy and required owner. (10). The packet records every changed-file page, server-recorded test evidence, and the named owner's approval. (13). It does not write code or merge pull requests. (9, F-1-4).
- Open /demo after starting the app, or use the live demo at agent-diff-gate.sociobot.in/demo. (13). The demo is sample-only. (5). Its session storage is cleared when demo mode ends. (9).
- Prerequisites: Node 22+ and Rust 1.88+. (5). Visit localhost:8080. (1). The server uses PORT (default 8080) and creates a SQLite database under /data. (12). For a local non-container run, set DATABASE_URL=sqlite:diff-gate.db?mode=rwc if /data is not writable. (13).
- The sample demo works with no configuration. (7, F-1-4). Real review packets use only Sociobot Entra External ID and a team-bound GitHub App installation. (15).
- The approved Sociobot tenant and public SPA client are non-secret deployment settings. (13, jargon). Sign-in uses authorization-code PKCE, so Diff Gate does not store an Entra client secret. (12, jargon/F-1-4). Production defaults to the approved Sociobot tenant and client recorded in deploy/production.env.json. (12).
- Set ENTRA_TEAM_CLAIM to an assigned shared-team claim when the tenant issues one. (13, jargon). The approved deployment uses the stable Entra oid as an isolated team workspace. (13, jargon). Only the exact Sociobot tenant on sociobotcustomers.ciamlogin.com is accepted. (9).
- The service validates issuer, tenant, audience, nonce, signing algorithm, and signing key before creating a session. (15, F-1-4).
- After sign-in, a team can create its own private GitHub App from the real-work panel. (16). GitHub's App Manifest flow returns a generated App identity and key directly to the backend. (15, F-1-4). The App requests read-only repository contents and pull-request access. (10). Its installation id is verified against that App before being bound to the Entra team. (15, F-1-4).
- GITHUB_APP_* and GITHUB_TEAM_INSTALLATIONS remain available for an administrator-provisioned shared App. (8, jargon). An unmapped team cannot import. (5). Imports read every changed-file page and reject pull requests above 10,000 files. (11).
- Before an import, a signed-in team saves a policy for the exact owner/repository: one sensitive path rule per line and the person required to approve it. (22, split for scanning). For example, schema/** | database-owner@example.com. (2).
- Imports refuse repositories without a policy, apply only that repository’s rules, and set the matching rule’s required owner as the packet owner. (20, terminology drift). A pull request matching multiple owners is rejected so the team can split the change or align its policy. (19, F-1-4).
- Test evidence is not a client-controlled checkbox. (7, vague). The service stores a command, result, signed-in actor, and server timestamp. (10, F-1-4). A client can send a done state, but the backend replaces it with the incomplete evidence check until it receives a non-empty command and result. (22, split).
- Each signed-in team can set packet retention from 1 to 3,650 days; the default is 90 days. (17). Expired packets and their audit history are removed according to that setting before packet reads. (14). A reviewer can also delete a packet and its history immediately. (11). The packet view lists team-scoped audit entries and includes them in the JSON export. (14).
- For frontend iteration, run npm run dev and visit the address Vite prints. (11). The browser tests cover demo isolation, demo data egress, JSON export, keyboard use, mobile layout, and both color schemes. (18). The Rust suite covers team isolation, named-owner approval, multi-page GitHub import, retention, audit history, response policy, and Sociobot authority restrictions. (18). See .factory/claims.json and .factory/demo.md. (3).
- Diff Gate costs $12 per developer per month or $99 per team per month. (12). The landing page sends checkout and license verification only to the Sociobot billing API; it does not include a payment-provider key. (20, F-1-2/F-1-4). Sociobot is the merchant of record. (6). The free sample, export, privacy controls, and accessibility controls remain available without a paid plan. (14, unlisted). Buyers can return with ?license=<token> or restore the token through the plan section. (11).
- The root Dockerfile builds the Vite frontend and Rust server. (9). The image starts with PORT only and /health returns the build SHA plus a non-secret durable store identity. (17, F-1-3). Mount /data for durable packet storage. (5). Run scripts/deploy-production.sh from an authenticated factory worker. (6).
- It reapplies the approved public Entra settings after the container helper, mounts the product's Azure Files /data share, forces one replica, and verifies the Entra callback plus a real revision replacement with the same durable store identity. (35, **over 22**; split into short operator steps).
- Production uses SQLite's single-process unix-none VFS because Azure Files does not provide SQLite byte-range locking. (18, operator jargon). Administrator-provisioned GITHUB_APP_PRIVATE_KEY values must use a Key Vault secret reference. (10). The team self-provisioning flow stores GitHub's generated private App key only in the team-scoped backend database. (16, unlisted security claim).
- Sample packets stay in the browser until demo mode ends. (9). Saved packets contain the values a reviewer submits and changed file paths returned by GitHub. (14, F-1-4). Team retention and explicit deletion remove packet audit history too. (10). The sample has no analytics or third-party runtime scripts. (9). Read the in-product /privacy and /terms pages before connecting a team. (9).

## Structure and visual checks

Confirmed: the dithered/halftone print identity matches design.md and is distinct; self-hosted assets; 44 px demo targets; no 390 px horizontal overflow; lang, single h1, landmarks, canonical update, favicon, OG/Twitter metadata, security headers, robots, sitemap, Privacy/Terms, and Back-to-demo focus all work. Initial JS gzip was 7.68 kB. /, /demo, /privacy, and /terms returned 200. The Entra link redirected to the Sociobot tenant. The checkout link was the dead link (F-1-2); the 404 console error is F-1-7; and full local accessibility verification fails (F-1-8).

## Missed leverage

No AI feature is warranted. This is a human-accountability workflow; an AI review conclusion would be decorative and weaken the named-owner boundary. GitHub import and JSON export already exist.

## What would make this perfect

Show the sample CTA in the first 390 px screen; make checkout real and prove it; make all claim commands clean-clone runnable; retain only exact claim-backed promises; standardize terminology; use descriptive section headings; and remove the 404 console error. Re-run the full review after repair.
