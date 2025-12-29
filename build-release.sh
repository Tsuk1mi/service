#!/usr/bin/env bash
# Скрипт быстрой сборки релизной версии Rimskiy Service под Linux
set -euo pipefail

echo "========================================"
echo "Building Rimskiy Service (Linux)"
echo "========================================"
echo

if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: Cargo not found! Install Rust via https://rustup.rs/." >&2
  exit 1
fi

echo "[1/4] Cleaning previous build..."
cargo clean

echo "[2/4] Building release..."
cargo build --release

echo "[3/4] Preparing release directory..."
rm -rf release
mkdir -p release/migrations

echo "[4/4] Copying artifacts..."
cp target/release/rimskiy_service release/rimskiy_service
cp .env.example release/.env.example
cp README.md release/README.md
cp migrations/*.sql release/migrations/

echo
echo "========================================"
echo "Build completed successfully!"
echo "========================================"
echo "Executable: release/rimskiy_service"
echo
echo "Next steps:"
echo "1. Copy the 'release' folder to your server"
echo "2. Rename .env.example to .env and configure it"
echo "3. Run ./rimskiy_service"
echo

