#!/bin/sh
set -eu

binary=${1:-target/release/diff-gate}
port=${DIFF_GATE_RUNTIME_TEST_PORT:-18080}
log_file=$(mktemp)

cleanup() {
  if [ -n "${server_pid:-}" ]; then kill "$server_pid" 2>/dev/null || true; fi
  rm -f "$log_file"
}
trap cleanup EXIT

test -x "$binary"
env -i PATH="$PATH" PORT="$port" BUILD_SHA=runtime-contract "$binary" >"$log_file" 2>&1 &
server_pid=$!
for _attempt in $(seq 1 30); do
  if health=$(curl --fail --silent "http://127.0.0.1:$port/health"); then
    printf '%s' "$health" | jq -e '.status == "ok" and .build == "runtime-contract" and (.storage_id | type == "string" and length > 0)' >/dev/null
    printf 'Runtime contract passed: PORT-only startup returned the build identity and durable store identity.\n'
    exit 0
  fi
  sleep 1
done

cat "$log_file" >&2
exit 1
