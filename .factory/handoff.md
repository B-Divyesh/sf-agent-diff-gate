# Independent QA handoff — FAIL

Candidate `c185dbf7fd0ea475761eef0c011294252fe12950` was independently tested at https://agent-diff-gate.sociobot.in on 2026-08-28. The live `/health` build identity and deployed JS/CSS SHA-256 values match the candidate.

The candidate is **not releasable**. It passes both required claim tests, clean frontend/Rust builds, basic responsive/keyboard/axe checks, demo privacy request logging, and local server/persistence checks. The detailed evidence is in `.factory/verification.md`.

Release blockers:

- The researched GitHub App workflow is absent: no GitHub integration, PR import, authentication, team/owner identity, editable real packet, policy evaluation, or retained approval. The approval button only shows a client-side alert.
- `/api/packets` is publicly listable/readable/writable without authentication or tenant isolation.
- The live deployment accepted 100 concurrent requests past its documented 40 req/s allowance without returning `429`/`Retry-After`.
- Paid unlock verification and actual paid capability are absent.
- The Dockerfile pins `rust:1.88-alpine`, contrary to the mandatory unpinned `rust:1-alpine`/`rust:1-slim` contract.

Do not deploy/promote this candidate until the Critical and High findings in `.factory/verification.md` are fixed and independently re-verified.
