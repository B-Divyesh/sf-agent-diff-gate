#!/bin/sh
set -eu

# Build first, then apply the image and stateful template in one revision. The
# generic factory deploy helper writes a three-replica, volume-free template and
# must not be used for this SQLite product.
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app_name=sf-agent-diff-gate
resource_group=sociobot
environment_name=factory-env
storage_name=agent-diff-gate-data-v4
storage_account=sociobotblob
registry=sociobotregistry
subscription_id=${AZURE_SUBSCRIPTION_ID:-283af945-693b-4a6e-b952-df928d0a18a9}
source_sha=$(git -C "$repo_dir" rev-parse HEAD)
short_sha=$(printf '%.12s' "$source_sha")
image="$registry.azurecr.io/$app_name:$short_sha"

public_base=$(jq -r .PUBLIC_BASE_URL "$repo_dir/deploy/production.env.json")
test "$public_base" = https://agent-diff-gate.sociobot.in

wait_for_provisioned() {
  for attempt in $(seq 1 120); do
    state=$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
      --query properties.provisioningState --output tsv 2>/dev/null || true)
    case "$state" in
      Succeeded) return 0 ;;
      Failed|Canceled)
        printf 'Container App provisioning ended in %s.\n' "$state" >&2
        return 1
        ;;
    esac
    if [ $((attempt % 12)) -eq 0 ]; then
      printf 'Waiting for Container App provisioning (state: %s).\n' "${state:-unavailable}"
    fi
    sleep 5
  done
  printf 'Container App did not finish provisioning within 10 minutes.\n' >&2
  return 1
}

if ! git -C "$repo_dir" diff --quiet || ! git -C "$repo_dir" diff --cached --quiet; then
  printf 'Refusing to deploy an uncommitted tree; commit the release first.\n' >&2
  exit 1
fi

printf 'Building %s from committed source %s.\n' "$image" "$source_sha"
az acr build --registry "$registry" --image "$app_name:$short_sha" \
  --file Dockerfile --build-arg "BUILD_SHA=$source_sha" --build-arg "GIT_SHA=$source_sha" \
  --build-arg "SOURCE_COMMIT=$source_sha" "$repo_dir"

storage_exists=$(az storage share-rm exists --resource-group "$resource_group" --storage-account "$storage_account" --name "$storage_name" --query exists --output tsv)
if [ "$storage_exists" != true ]; then
  az storage share-rm create --resource-group "$resource_group" --storage-account "$storage_account" --name "$storage_name" --quota 5 --enabled-protocols SMB --output none
fi
storage_key=$(az storage account keys list --resource-group "$resource_group" --account-name "$storage_account" --query '[0].value' --output tsv)
az containerapp env storage set --resource-group "$resource_group" --name "$environment_name" \
  --storage-name "$storage_name" --access-mode ReadWrite \
  --azure-file-account-name "$storage_account" --azure-file-account-key "$storage_key" \
  --azure-file-share-name "$storage_name" --output none
wait_for_provisioned

# Single revision mode is a control-plane invariant as well as an application
# template invariant. Set it before creating the new stateful revision.
az containerapp revision set-mode --resource-group "$resource_group" --name "$app_name" \
  --mode single --output none
wait_for_provisioned

# Preserve unrelated secret references and resource settings, but render every
# part of the durable SQLite contract and the new image into the same template.
app=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
body=$(printf '%s' "$app" | node "$repo_dir/deploy/production-contract.mjs" render \
  --config "$repo_dir/deploy/production.env.json" --storage "$storage_name" --image "$image")
az rest --method patch \
  --uri "https://management.azure.com/subscriptions/${subscription_id}/resourceGroups/${resource_group}/providers/Microsoft.App/containerApps/${app_name}?api-version=2024-03-01" \
  --body "$body" --output none
wait_for_provisioned

"$repo_dir/scripts/verify-live-deployment.sh" "$public_base" --replace "$source_sha" "$image"
