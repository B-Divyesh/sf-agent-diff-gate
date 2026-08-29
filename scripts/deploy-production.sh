#!/bin/sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app_name=sf-agent-diff-gate

/opt/fleet/lib/deploy-container.sh agent-diff-gate "$repo_dir" Dockerfile 8080

public_base=$(jq -r .PUBLIC_BASE_URL "$repo_dir/deploy/production.env.json")
authority=$(jq -r .ENTRA_AUTHORITY "$repo_dir/deploy/production.env.json")
tenant_id=$(jq -r .ENTRA_TENANT_ID "$repo_dir/deploy/production.env.json")
client_id=$(jq -r .ENTRA_CLIENT_ID "$repo_dir/deploy/production.env.json")
team_claim=$(jq -r .ENTRA_TEAM_CLAIM "$repo_dir/deploy/production.env.json")

az containerapp update --resource-group sociobot --name "$app_name" \
  --set-env-vars \
  "PUBLIC_BASE_URL=$public_base" \
  "ENTRA_AUTHORITY=$authority" \
  "ENTRA_TENANT_ID=$tenant_id" \
  "ENTRA_CLIENT_ID=$client_id" \
  "ENTRA_TEAM_CLAIM=$team_claim" \
  --output none

"$repo_dir/scripts/verify-live-identity.sh" "$public_base"
