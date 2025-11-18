# Building FMMLoader26

This document explains how to build FMMLoader26 locally for testing and development.

## Quick Start

### For Testing (Debug Build - Faster)

**Linux/macOS:**
```bash
./build-local.sh
```

**Windows:**
```bash
build-local.bat
```

**Or using npm:**
```bash
npm run build:debug
```

The debug build:
- ✅ Compiles much faster
- ✅ Includes debug symbols for troubleshooting
- ❌ Larger file size
- ❌ Not optimized for performance

### For Production (Release Build - Optimized)

```bash
npm run build:release
```

The release build:
- ✅ Fully optimized for performance
- ✅ Smaller file size
- ❌ Takes longer to compile
- ❌ No debug symbols

## Build Output Locations

### Debug Builds

**Linux:**
- Executable: `src-tauri/target/debug/fmmloader26`
- AppImage: `src-tauri/target/debug/bundle/appimage/fmmloader26_*.AppImage`

**macOS:**
- Executable: `src-tauri/target/debug/fmmloader26`
- App Bundle: `src-tauri/target/debug/bundle/macos/FMMLoader26.app`
- DMG: `src-tauri/target/debug/bundle/dmg/FMMLoader26_*.dmg`

**Windows:**
- Executable: `src-tauri\target\debug\fmmloader26.exe`
- Installer: `src-tauri\target\debug\bundle\nsis\FMMLoader26_*_x64-setup.exe`

### Release Builds

**Linux:**
- Executable: `src-tauri/target/release/fmmloader26`
- AppImage: `src-tauri/target/release/bundle/appimage/fmmloader26_*.AppImage`

**macOS:**
- Executable: `src-tauri/target/release/fmmloader26`
- App Bundle: `src-tauri/target/release/bundle/macos/FMMLoader26.app`
- DMG: `src-tauri/target/release/bundle/dmg/FMMLoader26_*.dmg`

**Windows:**
- Executable: `src-tauri\target\release\fmmloader26.exe`
- Installer: `src-tauri\target\release\bundle\nsis\FMMLoader26_*_x64-setup.exe`

## Development Workflow

### Running in Dev Mode
For the fastest development experience with hot-reload:

```bash
npm run dev
# or
npm run tauri dev
```

This starts the Vite dev server and runs Tauri in development mode with hot-reload.

### Building for Testing
When you want to test a build without the dev server:

```bash
npm run build:debug
```

### Building for Distribution
When you're ready to distribute or release:

```bash
npm run build:release
```

## Prerequisites

Before building, ensure you have:

1. **Node.js** (v18 or later)
2. **Rust** (latest stable)
3. **Tauri Prerequisites** for your platform:
   - **Linux:** See [Tauri Linux Setup](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux)
   - **macOS:** Xcode Command Line Tools
   - **Windows:** Microsoft Visual Studio C++ Build Tools

## Troubleshooting

### Build fails with "command not found: tauri"
Make sure Tauri CLI is installed:
```bash
npm install
```

### Build is very slow
Use debug mode for faster builds:
```bash
npm run build:debug
```

### "Cannot find module" errors
Rebuild dependencies:
```bash
npm install
npm run build
```

### Platform-specific issues
Check the [Tauri documentation](https://tauri.app/v1/guides/building/) for platform-specific build requirements.

## GitHub Actions

This project includes GitHub Actions workflows for CI/CD:

### CI Workflow (`.github/workflows/ci.yml`)
Runs on all pull requests and pushes to feature branches to ensure code quality:
- Frontend validation (build, ESLint, Prettier)
- Backend validation (cargo check, test, clippy, fmt)
- Must pass before merging to main

### Build/Release Workflow (`.github/workflows/build.yml`)
When you push a tag matching `v*`, it automatically:
1. Builds for Windows, macOS, and Linux
2. Creates installers/packages for each platform
3. Creates a GitHub release with all artifacts

See `.github/workflows/` for the workflow configurations.

## Local Release Testing

To test the release process locally without publishing:

1. Build in release mode:
   ```bash
   npm run build:release
   ```

2. Find your platform's installer in the build output locations above

3. Test the installer/package on your platform

## Next Steps

After building:
- Test the executable in `src-tauri/target/debug/` (or `release/`)
- Verify all features work as expected
- For distribution, use the bundled installers from `src-tauri/target/release/bundle/`
