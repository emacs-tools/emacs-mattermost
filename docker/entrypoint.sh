#!/usr/bin/env bash
set -euo pipefail

export MM_BRIDGE_LISTEN="${MM_BRIDGE_LISTEN:-0.0.0.0:8765}"
export MM_WEB_LISTEN="${MM_WEB_LISTEN:-0.0.0.0:8766}"
export MM_DATA_DIR="${MM_DATA_DIR:-/data}"

mkdir -p "${MM_DATA_DIR}"

if [[ "$#" -eq 0 ]]; then
  exec mm-bridge
fi

if [[ "$1" == "mm-bridge" ]]; then
  shift
  exec mm-bridge "$@"
fi

exec "$@"
