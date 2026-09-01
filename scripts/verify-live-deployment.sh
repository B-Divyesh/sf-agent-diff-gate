#!/bin/sh
set -eu

base_url=${1:-https://agent-diff-gate.sociobot.in}
replace=${2:-}
expected_build=${3:-$(git -C "$(dirname "$0")/.." rev-parse HEAD)}
expected_image=${4:-}
app_name=sf-agent-diff-gate
resource_group=sociobot
expected_base=https://agent-diff-gate.sociobot.in
storage_name=agent-diff-gate-data-v4
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

config=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
assert_control_plane() {
  current=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
  if [ -n "$expected_image" ]; then
    printf '%s' "$current" | node "$repo_dir/deploy/production-contract.mjs" assert \
      --config "$repo_dir/deploy/production.env.json" --storage "$storage_name" --image "$expected_image"
  else
    printf '%s' "$current" | node "$repo_dir/deploy/production-contract.mjs" assert \
      --config "$repo_dir/deploy/production.env.json" --storage "$storage_name"
  fi
}
assert_control_plane

wait_for_one_running_replica() {
  for _attempt in $(seq 1 48); do
    revision=$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
      --query properties.latestRevisionName --output tsv)
    running=$(az containerapp replica list --resource-group "$resource_group" --name "$app_name" \
      --revision "$revision" --query "[?properties.runningState == 'Running'] | length(@)" --output tsv)
    if [ "$running" = 1 ]; then return 0; fi
    sleep 5
  done
  printf 'Latest revision did not converge to exactly one running replica.\n' >&2
  return 1
}

wait_for_one_running_replica

wait_for_health() {
  for _attempt in $(seq 1 48); do
    if health=$(curl --fail --silent "$base_url/health") && printf '%s' "$health" | jq -e --arg build "$expected_build" \
      '.status == "ok" and .build == $build and (.storage_id | type == "string" and length > 0)' >/dev/null; then
      printf '%s' "$health"
      return 0
    fi
    sleep 5
  done
  return 1
}

before=$(wait_for_health)
before_id=$(printf '%s' "$before" | jq -r .storage_id)
test "$(printf '%s' "$before" | jq -r .build)" = "$expected_build"
before_revision=$(printf '%s' "$config" | jq -r .properties.latestRevisionName)
"$(dirname "$0")/verify-live-identity.sh" "$base_url"

concurrent_health_identity() {
  sample_dir=${1:?usage: concurrent_health_identity SAMPLE_DIR}
  mkdir -p "$sample_dir"
  for request in $(seq 1 100); do
    curl --fail --silent "$base_url/health" >"$sample_dir/$request.json" &
  done
  wait
  node "$repo_dir/deploy/live-health-identity.mjs" "$expected_build" "$sample_dir"/*.json
}

before_samples=$(mktemp -d)
test "$(concurrent_health_identity "$before_samples")" = "$before_id"

not_found_headers=$(mktemp)
trap 'rm -f "$not_found_headers"' EXIT
not_found_status=$(curl --header 'Accept: application/json' --silent --dump-header "$not_found_headers" --output /dev/null --write-out '%{http_code}' "$base_url/this-route-does-not-exist")
test "$not_found_status" = 404
"$repo_dir/scripts/assert-not-found-headers.sh" "$not_found_headers"
test "$(curl --header 'Accept: application/json' --silent --output /dev/null --write-out '%{http_code}' "$base_url/404")" = 404
node "$repo_dir/deploy/live-rate-limit.mjs" "$base_url"

if [ "$replace" = "--replace" ]; then
  probe="durable-$(date +%s)"
  az containerapp update --resource-group "$resource_group" --name "$app_name" \
    --set-env-vars "DURABLE_REPLACEMENT_PROBE=$probe" --output none
  for _attempt in $(seq 1 48); do
    current_revision=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --query properties.latestRevisionName --output tsv)
    if [ "$current_revision" != "$before_revision" ]; then break; fi
    sleep 5
  done
  test "$current_revision" != "$before_revision"
  after=$(wait_for_health)
  after_id=$(printf '%s' "$after" | jq -r .storage_id)
  test "$after_id" = "$before_id"
  test "$(printf '%s' "$after" | jq -r .build)" = "$expected_build"
  assert_control_plane
  wait_for_one_running_replica
  after_samples=$(mktemp -d)
  test "$(concurrent_health_identity "$after_samples")" = "$before_id"
  test "$(node "$repo_dir/deploy/release-health-evidence.mjs" \
    --expected-build "$expected_build" \
    --before-revision "$before_revision" \
    --after-revision "$current_revision" \
    --before "$before_samples"/*.json \
    --after "$after_samples"/*.json)" = "$before_id"
  rm -rf "$after_samples"
  node "$repo_dir/deploy/live-rate-limit.mjs" "$base_url"
fi

rm -rf "$before_samples"

printf 'Live deployment contract passed: expected build, one concurrent storage identity, global 40-request allowance with Retry-After, public Entra callback, one replica, Azure Files /data, and durable replacement identity %s.\n' "$before_id"
