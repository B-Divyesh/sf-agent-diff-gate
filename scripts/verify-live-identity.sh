#!/bin/sh
set -eu

base_url=${1:-https://agent-diff-gate.sociobot.in}
status_file=$(mktemp)
headers_file=$(mktemp)
callback_file=$(mktemp)
callback_headers=$(mktemp)
trap 'rm -f "$status_file" "$headers_file" "$callback_file" "$callback_headers"' EXIT

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

curl --fail --silent --show-error --dump-header "$callback_headers" \
  "$base_url/auth/callback?error=access_denied&error_description=User%20cancelled" >"$callback_file"
grep -Fq '<h1>Sign-in did not complete</h1>' "$callback_file"
grep -Fq 'Try sign-in again' "$callback_file"
grep -Fq 'Return to Diff Gate' "$callback_file"
grep -Fq 'Try it with sample data' "$callback_file"
if grep -Fq 'missing field `code`' "$callback_file"; then
  printf 'Entra error callback exposed raw deserialization text.\n' >&2
  exit 1
fi
awk 'BEGIN { IGNORECASE=1; html=0; robots=0 }
  /^content-type:[[:space:]]*text\/html/ { html=1 }
  /^x-robots-tag:[[:space:]]*noindex/ { robots=1 }
  END { exit !(html && robots) }' "$callback_headers"

printf 'Live identity configuration uses Sociobot Entra PKCE and its canceled callback has a product recovery screen.\n'
