#!/bin/bash

# FMMLoader26 Local Build Script
# This script builds the app for testing without creating a release

set -e

echo "🔨 Building FMMLoader26 (Debug Mode)..."
echo ""

# Build the frontend
echo "📦 Building frontend..."
npm run build

# Build the Tauri app in debug mode (faster compilation)
echo "🦀 Building Tauri app (debug mode)..."
npm run tauri build -- --debug

echo ""
echo "✅ Build complete!"
echo ""
echo "📍 Build artifacts location:"
echo ""

# Detect OS and show the correct path
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Linux executables:"
    echo "  - src-tauri/target/debug/fmmloader26"
    echo "  - src-tauri/target/debug/bundle/appimage/fmmloader26_*.AppImage (if bundled)"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "macOS executables:"
    echo "  - src-tauri/target/debug/fmmloader26"
    echo "  - src-tauri/target/debug/bundle/dmg/FMMLoader26_*.dmg (if bundled)"
    echo "  - src-tauri/target/debug/bundle/macos/FMMLoader26.app"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    echo "Windows executables:"
    echo "  - src-tauri/target/debug/fmmloader26.exe"
    echo "  - src-tauri/target/debug/bundle/nsis/FMMLoader26_*_x64-setup.exe (if bundled)"
else
    echo "Executable should be in: src-tauri/target/debug/"
fi

echo ""
echo "💡 To run the app directly:"
echo "   cd src-tauri/target/debug && ./fmmloader26"
echo ""
echo "🚀 For a release build (slower but optimized), run:"
echo "   npm run build:release"
echo ""
