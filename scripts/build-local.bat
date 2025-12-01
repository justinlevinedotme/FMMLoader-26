@echo off
REM FMMLoader26 Local Build Script for Windows
REM This script builds the app for testing without creating a release

echo.
echo 🔨 Building FMMLoader26 (Debug Mode)...
echo.

REM Build the frontend
echo 📦 Building frontend...
call pnpm run build

if %errorlevel% neq 0 (
    echo ❌ Frontend build failed!
    exit /b %errorlevel%
)

REM Build the Tauri app in debug mode (faster compilation)
echo.
echo 🦀 Building Tauri app (debug mode)...
call pnpm run tauri build -- --debug

if %errorlevel% neq 0 (
    echo ❌ Tauri build failed!
    exit /b %errorlevel%
)

echo.
echo ✅ Build complete!
echo.
echo 📍 Build artifacts location:
echo.
echo Windows executables:
echo   - src-tauri\target\debug\fmmloader26.exe
echo   - src-tauri\target\debug\bundle\nsis\FMMLoader26_*_x64-setup.exe (if bundled)
echo.
echo 💡 To run the app directly:
echo    cd src-tauri\target\debug
echo    fmmloader26.exe
echo.
echo 🚀 For a release build (slower but optimized), run:
echo    pnpm run build:release
echo.
pause
