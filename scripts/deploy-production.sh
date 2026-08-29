#!/bin/sh
set -eu

# The factory helper intentionally starts an unconfigured, scalable app. Apply this
# product's stateful runtime contract immediately afterwards and verify the final
# revision, rather than treating a successful image deployment as a release.
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app_name=sf-agent-diff-gate
resource_group=sociobot
environment_name=factory-env
storage_name=agent-diff-gate-data-v4
storage_account=sociobotblob
subscription_id=${AZURE_SUBSCRIPTION_ID:-283af945-693b-4a6e-b952-df928d0a18a9}

public_base=$(jq -r .PUBLIC_BASE_URL "$repo_dir/deploy/production.env.json")
authority=$(jq -r .ENTRA_AUTHORITY "$repo_dir/deploy/production.env.json")
tenant_id=$(jq -r .ENTRA_TENANT_ID "$repo_dir/deploy/production.env.json")
client_id=$(jq -r .ENTRA_CLIENT_ID "$repo_dir/deploy/production.env.json")
team_claim=$(jq -r .ENTRA_TEAM_CLAIM "$repo_dir/deploy/production.env.json")
test "$public_base" = https://agent-diff-gate.sociobot.in

/opt/fleet/lib/deploy-container.sh agent-diff-gate "$repo_dir" Dockerfile 8080

storage_exists=$(az storage share-rm exists --resource-group "$resource_group" --storage-account "$storage_account" --name "$storage_name" --query exists --output tsv)
if [ "$storage_exists" != true ]; then
  az storage share-rm create --resource-group "$resource_group" --storage-account "$storage_account" --name "$storage_name" --quota 5 --enabled-protocols SMB --output none
fi
storage_key=$(az storage account keys list --resource-group "$resource_group" --account-name "$storage_account" --query '[0].value' --output tsv)
az containerapp env storage set --resource-group "$resource_group" --name "$environment_name" \
  --storage-name "$storage_name" --access-mode ReadWrite \
  --azure-file-account-name "$storage_account" --azure-file-account-key "$storage_key" \
  --azure-file-share-name "$storage_name" --output none

# Preserve the just-built image and resources while replacing the product's
# stateful template. Explicit arrays prevent a later helper revision retaining
# its PORT-only, three-replica configuration.
app=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
template=$(printf '%s' "$app" | jq -c \
  --arg storage "$storage_name" \
  --arg public_base "$public_base" \
  --arg authority "$authority" \
  --arg tenant_id "$tenant_id" \
  --arg client_id "$client_id" \
  --arg team_claim "$team_claim" '
  .properties.template
  | .containers |= map(
      if .name == "app" then
        .volumeMounts = ((.volumeMounts // []) | map(select(.mountPath != "/data" and .volumeName != "data")) + [{volumeName:"data",mountPath:"/data"}])
        | .env = ((.env // []) | map(select(.name != "DATABASE_URL" and .name != "PUBLIC_BASE_URL" and .name != "ENTRA_AUTHORITY" and .name != "ENTRA_TENANT_ID" and .name != "ENTRA_CLIENT_ID" and .name != "ENTRA_TEAM_CLAIM" and .name != "DEPLOYMENT_CONFIG_VERSION")) + [
            {name:"DATABASE_URL",value:"sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none"},
            {name:"PUBLIC_BASE_URL",value:$public_base},
            {name:"ENTRA_AUTHORITY",value:$authority},
            {name:"ENTRA_TENANT_ID",value:$tenant_id},
            {name:"ENTRA_CLIENT_ID",value:$client_id},
            {name:"ENTRA_TEAM_CLAIM",value:$team_claim},
            {name:"DEPLOYMENT_CONFIG_VERSION",value:"2"}
          ])
      else . end)
  | .scale = {minReplicas:1,maxReplicas:1}
  | .volumes = ((.volumes // []) | map(select(.name != "data")) + [{name:"data",storageType:"AzureFile",storageName:$storage}])
')
body=$(jq -cn --argjson template "$template" '{properties:{template:$template}}')
az rest --method patch \
  --uri "https://management.azure.com/subscriptions/${subscription_id}/resourceGroups/${resource_group}/providers/Microsoft.App/containerApps/${app_name}?api-version=2024-03-01" \
  --body "$body" --output none

"$repo_dir/scripts/verify-live-deployment.sh" "$public_base" --replace
