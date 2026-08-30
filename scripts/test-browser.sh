#!/bin/sh
set -eu

# Playwright's backend is the already-built binary. This makes the exact
# browser/claim command reliable from a cold checkout: compiling Rust is a
# deliberate preparation step, not time spent inside the health-check window.
cargo build --quiet
exec ./node_modules/.bin/playwright test "$@"
