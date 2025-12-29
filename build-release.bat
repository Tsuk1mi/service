@echo off
REM Скрипт для сборки релизной версии Rimskiy Service для Windows

echo ========================================
echo Building Rimskiy Service Release
echo ========================================
echo.

REM Проверяем наличие cargo
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Cargo not found! Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo [1/4] Cleaning previous build...
cargo clean

echo [2/4] Building release version...
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo [3/4] Creating release directory...
if exist release rmdir /s /q release
mkdir release
mkdir release\migrations

echo [4/4] Copying files...
copy target\release\rimskiy_service.exe release\rimskiy_service.exe
copy .env.example release\.env.example
copy README.md release\README.md
if exist release\README_DEPLOY.md copy release\README_DEPLOY.md release\README_DEPLOY.md
if exist release\START.bat copy release\START.bat release\START.bat
xcopy migrations\*.sql release\migrations\ /Y /Q

echo.
echo ========================================
echo Build completed successfully!
echo ========================================
echo.
echo Executable: release\rimskiy_service.exe
echo.
echo Next steps:
echo 1. Copy the 'release' folder to your server
echo 2. Rename .env.example to .env and configure it
echo 3. Run rimskiy_service.exe
echo.
pause

