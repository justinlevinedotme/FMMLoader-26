# Building FMMLoader26

This guide consolidates all build, dev, and release instructions for FMMLoader26.

## Prerequisites

### All platforms
- Node.js 18+ and npm
- Rust (latest stable) – install via https://rustup.rs
- Git

### Platform-specific
- **Windows:** Visual Studio C++ Build Tools (or VS 2022 with “Desktop development with C++”), WebView2 (usually preinstalled; download if missing).
- **macOS:** Xcode Command Line Tools (`xcode-select --install`).
- **Linux (Ubuntu/Debian):**
  ```bash
  sudo apt update
  sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl wget file \
    libssl-dev libgtk-3-dev libayatana-appindicator3-dev \
    librsvg2-dev libgdk-pixbuf-2.0-dev pango1.0-dev \
    atk1.0-dev libsoup-3.0-dev libjavascriptcoregtk-4.0-dev
  ```
- **Linux (Fedora/RHEL/CentOS):**
  ```bash
  sudo dnf install -y \
    webkit2gtk4.0-devel openssl-devel \
    curl wget file libappindicator-gtk3-devel \
    librsvg2-devel gtk3-devel pango-devel atk-devel
  ```
- **Linux (Arch):**
  ```bash
  sudo pacman -Syu
  sudo pacman -S --needed \
    webkit2gtk base-devel curl wget file \
    openssl appmenu-gtk-module gtk3 \
    libappindicator-gtk3 librsvg libvips pango atk
  ```

## Setup
```bash
git clone https://github.com/justinlevinedotme/FMMLoader-26.git
cd FMMLoader-26
npm install
```

## Development (hot reload)
```bash
npm run dev          # Vite dev server
# or
npm run tauri dev    # Tauri dev with hot reload
```

## Build for testing
```bash
npm run build:debug          # Faster, includes debug symbols
# or platform helpers
./build-local.sh             # macOS/Linux shortcut
build-local.bat              # Windows shortcut
```

## Build for release (distribution)
```bash
npm run build:release        # Full optimized build + Tauri bundle
```

### Build outputs
- **Debug:** `src-tauri/target/debug/...`
- **Release installers:** `src-tauri/target/release/bundle/`
  - Windows: `.msi`, `.exe`
  - macOS: `.dmg`, `.app`
  - Linux: `.AppImage`, `.deb`, `.rpm`

## CI and releases (distribution scope)
- **Tag-based releases:** `.github/workflows/build.yml` builds and publishes Tauri installers on `v*` tags (GitHub releases).
- **CI checks:** `.github/workflows/ci.yml` runs npm build/lint/format and Rust check/test/clippy/fmt on pushes/PRs.
- No npm/package publish workflow is used; distribution is via GitHub release artifacts.

## Code quality
```bash
npm run lint          # ESLint
npm run lint:fix
npm run format        # Prettier check
npm run format:fix
(cd src-tauri && cargo fmt && cargo clippy && cargo test)
```

## Troubleshooting
- “command not found: tauri”: ensure `npm install` ran (pulls Tauri CLI scripts) and prerequisites are installed.
- Slow builds: use `npm run build:debug`.
- Module resolution errors: `npm install && npm run build`.
- Platform-specific issues: see https://tauri.app/v1/guides/building/ for OS notes.

## Local release testing
```bash
npm run build:release
# then install from src-tauri/target/release/bundle/<platform>/
```

## App data locations (reset)
- Windows: `%APPDATA%/FMMLoader26/`
- macOS: `~/Library/Application Support/FMMLoader26/`
- Linux: `~/.local/share/FMMLoader26/`

