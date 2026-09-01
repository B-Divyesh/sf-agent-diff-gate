# Diff Gate review 3 handoff

- **Result:** FAIL
- **Review:** `.factory/review-3.md`
- **Reviewed:** 2026-09-01 at <https://agent-diff-gate.sociobot.in>
- **Clean checkout:** `46b3fcc33a141479ca949c27e684bcb1103fcde6`
- **Live product build:** `155e6c200f3cffa3a98f904337b695571f5ba78d` (later checkout change is documentation only)

## What was done

Checked the live product cold at 390 × 844 and 1440 × 900, completed the one-click sample flow, confirmed Reset and Start for real, parsed the export, checked offline behavior and request boundaries, crawled links, checked route metadata and navigation, ran live Axe checks, and compared every earlier review finding with the current live site and code.

No product code was changed. Review screenshots and the URL verifier output are in `.factory/review-3-artifacts/`.

## Verification

- All 21 commands in `.factory/claims.json`: PASS from a clean checkout.
- `npm test`: PASS, including 17 Node tests and 26 Playwright tests.
- `npm run build`: PASS; `dist/` produced 7.28 kB gzip JavaScript.
- `npx tsc --noEmit`: PASS.
- `cargo test --all-targets`: PASS, 23 tests.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS.
- `/opt/fleet/lib/verify-url.sh`: PASS for the live home page.
- Live public routes: no serious or critical Axe findings.

## Work remaining

1. F-3-1 reopens F-2-1/F-1-7: direct live 404 navigation logs a resource error, and the live smoke script suppresses that exact event.
2. F-3-2: README deployment promises are broader than the listed `stateful-worker-deploy` claim and tagged test.
3. F-3-3: the README retains one vague heading and several jargon-heavy deployment sentences.

See `.factory/review-3.md` for exact quotes, evidence, and concrete rewrites.
