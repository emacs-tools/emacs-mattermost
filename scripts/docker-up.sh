#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "${ROOT_DIR}"

mkdir -p data

./scripts/docker-prepare-ca.sh

docker compose up -d --build mm-bridge

echo "Bridge TCP: 127.0.0.1:8765"
echo "Web status: http://127.0.0.1:8766/"
