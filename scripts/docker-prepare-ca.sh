#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CERT_DIR="${ROOT_DIR}/docker/certs"
TARGET="${CERT_DIR}/corporate-ca.crt"

SOURCE="${MM_CORPORATE_CA_FILE:-/etc/ssl/certs/ca-certificates.crt}"

if [[ ! -f "${SOURCE}" ]]; then
  echo "CA source not found: ${SOURCE}" >&2
  echo "Set MM_CORPORATE_CA_FILE to your corporate CA bundle path." >&2
  exit 1
fi

mkdir -p "${CERT_DIR}"
cp "${SOURCE}" "${TARGET}"

echo "CA bundle copied:"
echo "  from: ${SOURCE}"
echo "  to:   ${TARGET}"
