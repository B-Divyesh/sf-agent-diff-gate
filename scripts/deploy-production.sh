#!/bin/sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app_name=sf-agent-diff-gate
storage_name=agent-diff-gate-data-v4
storage_account=sociobotblob
subscription_id=${AZURE_SUBSCRIPTION_ID:-283af945-693b-4a6e-b952-df928d0a18a9}

/opt/fleet/lib/deploy-container.sh agent-diff-gate "$repo_dir" Dockerfile 8080

public_base=$(jq -r .PUBLIC_BASE_URL "$repo_dir/deploy/production.env.json")
authority=$(jq -r .ENTRA_AUTHORITY "$repo_dir/deploy/production.env.json")
tenant_id=$(jq -r .ENTRA_TENANT_ID "$repo_dir/deploy/production.env.json")
client_id=$(jq -r .ENTRA_CLIENT_ID "$repo_dir/deploy/production.env.json")
team_claim=$(jq -r .ENTRA_TEAM_CLAIM "$repo_dir/deploy/production.env.json")

az containerapp update --resource-group sociobot --name "$app_name" \
  --min-replicas 1 \
  --max-replicas 1 \
  --set-env-vars \
  "PUBLIC_BASE_URL=$public_base" \
  "ENTRA_AUTHORITY=$authority" \
  "ENTRA_TENANT_ID=$tenant_id" \
  "ENTRA_CLIENT_ID=$client_id" \
  "ENTRA_TEAM_CLAIM=$team_claim" \
  "DEPLOYMENT_CONFIG_VERSION=1" \
  --output none

storage_exists=$(az storage share-rm exists --resource-group sociobot --storage-account "$storage_account" --name "$storage_name" --query exists --output tsv)
if [ "$storage_exists" != true ]; then
  az storage share-rm create --resource-group sociobot --storage-account "$storage_account" --name "$storage_name" --quota 5 --enabled-protocols SMB --output none
fi
storage_key=$(az storage account keys list --resource-group sociobot --name "$storage_account" --query '[0].value' --output tsv)
az containerapp env storage set --resource-group sociobot --name factory-env \
  --storage-name "$storage_name" --access-mode ReadWrite \
  --azure-file-account-name "$storage_account" --azure-file-account-key "$storage_key" \
  --azure-file-share-name "$storage_name" --output none

template=$(az containerapp show --resource-group sociobot --name "$app_name" --output json | jq -c --arg storage "$storage_name" '{properties:{template:{containers:(.properties.template.containers | map(if .name=="app" then . + {volumeMounts:[{volumeName:"data",mountPath:"/data"}],env:((.env | map(select(.name != "DATABASE_URL"))) + [{name:"DATABASE_URL",value:"sqlite:/data/diff-gate.db?mode=rwc&vfs=unix-none"}])} else . end)),scale:{minReplicas:1,maxReplicas:1},volumes:[{name:"data",storageType:"AzureFile",storageName:$storage}]}}}')
az rest --method patch \
  --uri "https://management.azure.com/subscriptions/${subscription_id}/resourceGroups/sociobot/providers/Microsoft.App/containerApps/${app_name}?api-version=2024-03-01" \
  --body "$template" --output none

"$repo_dir/scripts/verify-live-identity.sh" "$public_base"
