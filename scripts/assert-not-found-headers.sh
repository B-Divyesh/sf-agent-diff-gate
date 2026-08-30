#!/bin/sh
set -eu

# Deployment hooks provide POSIX awk but not ripgrep. Keep the live 404
# assertions case-insensitive because HTTP header field names are case-insensitive.
headers_file=${1:?usage: assert-not-found-headers.sh HEADERS_FILE}

awk '
  {
    line = tolower($0)
    sub(/\r$/, "", line)
    if (line ~ /^x-diff-gate-route:[[:space:]]*not-found/) route = 1
    if (line ~ /^x-robots-tag:[[:space:]]*noindex/) robots = 1
  }
  END {
    if (!route) {
      print "Missing X-Diff-Gate-Route: not-found response header." > "/dev/stderr"
      exit 1
    }
    if (!robots) {
      print "Missing X-Robots-Tag: noindex response header." > "/dev/stderr"
      exit 1
    }
  }
' "$headers_file"
