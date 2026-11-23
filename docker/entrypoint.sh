#!/usr/bin/env bash
set -euo pipefail

export MM_BRIDGE_LISTEN="${MM_BRIDGE_LISTEN:-0.0.0.0:8765}"
export MM_WEB_LISTEN="${MM_WEB_LISTEN:-0.0.0.0:8766}"
export MM_DATA_DIR="${MM_DATA_DIR:-/data}"

# Ensure data directory exists with correct ownership.
# Entrypoint runs as root; we fix permissions then drop to mmbridge below.
mkdir -p "${MM_DATA_DIR}"
chown -R mmbridge:mmbridge "${MM_DATA_DIR}" 2>/dev/null || true

# Drop privileges to the unprivileged mmbridge user.
if [[ "$#" -eq 0 ]]; then
  exec runuser -u mmbridge -- mm-bridge
fi

if [[ "$1" == "mm-bridge" ]]; then
  shift
  exec runuser -u mmbridge -- mm-bridge "$@"
fi

exec runuser -u mmbridge -- "$@"
