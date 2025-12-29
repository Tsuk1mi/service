# Скрипт для сборки релизной версии Rimskiy Service для Windows (PowerShell)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building Rimskiy Service Release" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Проверяем наличие cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Cargo not found! Please install Rust from https://rustup.rs/" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host "[1/4] Cleaning previous build..." -ForegroundColor Yellow
cargo clean

Write-Host "[2/4] Building release version..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Build failed!" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host "[3/4] Creating release directory..." -ForegroundColor Yellow
if (Test-Path "release") {
    Remove-Item -Recurse -Force "release"
}
New-Item -ItemType Directory -Path "release" | Out-Null
New-Item -ItemType Directory -Path "release\migrations" | Out-Null

Write-Host "[4/4] Copying files..." -ForegroundColor Yellow
Copy-Item "target\release\rimskiy_service.exe" "release\rimskiy_service.exe"
Copy-Item ".env.example" "release\.env.example"
Copy-Item "README.md" "release\README.md"
if (Test-Path "release\README_DEPLOY.md") { Copy-Item "release\README_DEPLOY.md" "release\README_DEPLOY.md" }
if (Test-Path "release\START.bat") { Copy-Item "release\START.bat" "release\START.bat" }
Copy-Item "migrations\*.sql" "release\migrations\"

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Executable: release\rimskiy_service.exe" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Copy the 'release' folder to your server"
Write-Host "2. Rename .env.example to .env and configure it"
Write-Host "3. Run rimskiy_service.exe"
Write-Host ""
Read-Host "Press Enter to exit"

