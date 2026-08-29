#!/bin/sh
set -eu

base_url=${1:-https://agent-diff-gate.sociobot.in}
replace=${2:-}
app_name=sf-agent-diff-gate
resource_group=sociobot
expected_base=https://agent-diff-gate.sociobot.in

config=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
printf '%s' "$config" | jq -e --arg base "$expected_base" '
  .properties.template.scale.minReplicas == 1 and
  .properties.template.scale.maxReplicas == 1 and
  ([.properties.template.volumes[]? | select(.name == "data" and .storageType == "AzureFile")] | length) == 1 and
  ([.properties.template.containers[] | select(.name == "app") | .volumeMounts[]? | select(.volumeName == "data" and .mountPath == "/data")] | length) == 1 and
  ([.properties.template.containers[] | select(.name == "app") | .env[]? | select(.name == "PUBLIC_BASE_URL" and .value == $base)] | length) == 1
' >/dev/null

wait_for_health() {
  for _attempt in $(seq 1 48); do
    if health=$(curl --fail --silent "$base_url/health") && printf '%s' "$health" | jq -e '.status == "ok" and (.storage_id | type == "string" and length > 0)' >/dev/null; then
      printf '%s' "$health"
      return 0
    fi
    sleep 5
  done
  return 1
}

before=$(wait_for_health)
before_id=$(printf '%s' "$before" | jq -r .storage_id)
before_revision=$(printf '%s' "$config" | jq -r .properties.latestRevisionName)
"$(dirname "$0")/verify-live-identity.sh" "$base_url"

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
fi

printf 'Live deployment contract passed: public Entra callback, one replica, Azure Files /data, and durable replacement identity.\n'
