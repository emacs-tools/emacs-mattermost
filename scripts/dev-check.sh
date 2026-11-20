#!/bin/bash
# scripts/dev-check.sh
# Быстрая проверка Rust-части проекта emacs-mattermost

set -e

# Переходим в директорию с Rust workspace
cd "$(dirname "$0")/../bridge"

echo "==> Running cargo fmt --all..."
cargo fmt --all

echo "==> Running cargo check --workspace..."
cargo check --workspace

echo "==> Running cargo test --workspace..."
cargo test --workspace

echo "==> All checks passed successfully."
