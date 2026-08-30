# Landing-page copy audit

Audited again against the verification-24 repair on 2026-08-30. Commands and field labels are included where they convey visitor-facing meaning. No sentence exceeds 22 words or uses a banned marketing word.

| Sentence | Words | Flag |
|---|---:|---|
| Review packets for agent-authored changes | 5 | — |
| Review agent-authored changes before merge | 5 | — |
| For small software teams that need a required owner and test evidence before an agent-authored change lands. | 17 | — |
| Opens a sample packet with changed files, test evidence, and owner checks. | 11 | — |
| Sample data stays in this browser. | 6 | tested: sample-sandbox |
| Signed-in teams see only their review packets. | 7 | tested: team-packet-boundary |
| Export the sample packet as JSON. | 6 | tested: packet-export |
| Sign in before reviewing repository changes. | 6 | — |
| Packets are visible only to that reviewer’s team. | 8 | tested: team-packet-boundary |
| Team workspace is temporarily unavailable | 5 | — |
| Diff Gate is preparing its durable review workspace. | 8 | — |
| Try again shortly. | 3 | — |
| The sample demo still works without an account. | 9 | tested: sample-sandbox |
| How review packets work | 4 | — |
| Open your team review workspace. | 5 | — |
| Name sensitive paths and the required owner. | 7 | tested: repository-policy |
| Save test evidence before the required owner approves. | 8 | tested: named-approval |
| Diff Gate records a review decision. | 6 | tested: no-merge-action |
| Your team merges code outside Diff Gate. | 7 | tested: no-merge-action |
| Review agent-authored changes before merge. | 5 | — |
| Sign-in did not complete | 4 | — |
| Sign-in was cancelled or your account did not grant access. | 10 | — |
| No review data was changed. | 5 | — |
| Try again, return to Diff Gate, or open the sample. | 10 | — |
| Try sign-in again | 4 | — |
| Return to Diff Gate | 4 | — |
| Try it with sample data | 6 | tested: sample-sandbox |

## Terminology

| Concept | Word used |
|---|---|
| Change made with an agent | agent-authored change |
| Person allowed to approve | required owner |
| Collected review record | review packet |
| Command and result supporting a change | test evidence |
| Repository path requiring approval | sensitive path |
| Isolated try-out | sample demo |
| Stored sequence of packet actions | audit history |
