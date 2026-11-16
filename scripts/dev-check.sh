#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "${ROOT_DIR}/bridge"

if [[ ! -f Cargo.lock ]]; then
  echo "Cargo.lock is missing. Generate it first:" >&2
  echo "  cd bridge && cargo generate-lockfile" >&2
  exit 1
fi

cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
