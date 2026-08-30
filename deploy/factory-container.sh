#!/bin/sh
set -eu

# The factory's ordinary container release intentionally knows nothing about
# this product's durable SQLite topology. This repository hook is the single
# deployment entry point for worker-driven releases, including the automatic
# post-turn deploy.
slug=${1:-}
repo_dir=${2:-}
port=${4:-8080}

if [ "$slug" != "agent-diff-gate" ]; then
  printf 'Refusing factory deployment for unexpected product %s.\n' "${slug:-<empty>}" >&2
  exit 2
fi
if [ "$port" != "8080" ]; then
  printf 'Diff Gate must be deployed on container port 8080, not %s.\n' "$port" >&2
  exit 2
fi
if [ -z "$repo_dir" ] || [ ! -x "$repo_dir/scripts/deploy-production.sh" ]; then
  printf 'Diff Gate stateful deployment script is unavailable.\n' >&2
  exit 2
fi

exec "$repo_dir/scripts/deploy-production.sh"
