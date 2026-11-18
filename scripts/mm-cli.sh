#!/usr/bin/env bash
set -euo pipefail

HOST="${MM_BRIDGE_HOST:-127.0.0.1}"
PORT="${MM_BRIDGE_PORT:-8765}"

# Read JSON from stdin or argument
if [[ $# -gt 0 ]]; then
  INPUT="$1"
else
  read -r INPUT
fi

# Use -q 1 to wait 1 second for response after EOF on stdin
printf '%s\n' "$INPUT" | nc -q 1 "$HOST" "$PORT"
