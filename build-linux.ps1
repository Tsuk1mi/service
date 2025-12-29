<#
Скрипт для сборки Linux-бинарника на Windows через Docker.
Требования:
- Установлен Docker Desktop (должен работать docker cli).
- Запускать из корня репозитория PowerShell.

Результат:
- release\rimskiy_service (Linux ELF)
- миграции и README в release\
#>

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building Rimskiy Service (Linux via Docker)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Docker not found. Install Docker Desktop." -ForegroundColor Red
    exit 1
}

$projectRoot = (Resolve-Path ".").Path
$image = "rust:1.76"

Write-Host "[1/4] Cleaning previous release..." -ForegroundColor Yellow
Remove-Item -Recurse -Force "$projectRoot\release" -ErrorAction SilentlyContinue

Write-Host "[2/4] Pulling builder image ($image)..." -ForegroundColor Yellow
docker pull $image | Out-Null

Write-Host "[3/4] Building inside container..." -ForegroundColor Yellow
$buildCmd = @"
set -euo pipefail
apt-get update -y
apt-get install -y pkg-config libssl-dev
cargo clean
cargo build --release
strip target/release/rimskiy_service || true
rm -rf release
mkdir -p release/migrations
cp target/release/rimskiy_service release/rimskiy_service
cp .env.example release/.env.example
cp README.md release/README.md
cp migrations/*.sql release/migrations/
"@

docker run --rm `
  -v "$projectRoot":/workspace `
  -w /workspace `
  $image bash -lc "$buildCmd"

Write-Host "[4/4] Done. Linux artifacts in release/ (rimskiy_service)" -ForegroundColor Green

