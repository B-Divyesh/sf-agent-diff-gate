#!/bin/sh
set -eu

base_url=${1:-https://agent-diff-gate.sociobot.in}
status_file=$(mktemp)
headers_file=$(mktemp)
trap 'rm -f "$status_file" "$headers_file"' EXIT

ready=false
for _attempt in $(seq 1 30); do
  if curl --fail --silent --show-error "$base_url/api/auth/status" >"$status_file" \
    && jq -e '.authenticated == false and .entra_sign_in_configured == true and .github_app_setup_available == true' "$status_file" >/dev/null \
    && curl --fail --silent --show-error --output /dev/null --dump-header "$headers_file" "$base_url/auth/entra"; then
    location=$(awk 'BEGIN { IGNORECASE=1 } /^location:/ { sub(/^[^:]+:[[:space:]]*/, ""); sub(/\r$/, ""); print; exit }' "$headers_file")
    if printf '%s' "$location" | grep -Fq 'sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/oauth2/v2.0/authorize' \
      && printf '%s' "$location" | grep -Fq 'client_id=25c704f4-465a-47af-80ab-2c489466b697' \
      && printf '%s' "$location" | grep -Fq 'redirect_uri=https%3A%2F%2Fagent-diff-gate.sociobot.in%2Fauth%2Fcallback' \
      && printf '%s' "$location" | grep -Fq 'code_challenge_method=S256'; then
      ready=true
      break
    fi
  fi
  sleep 5
done

[ "$ready" = true ]

printf 'Live identity configuration is ready and redirects only to Sociobot Entra with PKCE.\n'
