# FMMLoader26 Build Instructions

## Prerequisites

### All Platforms

1. **Node.js** (v18 or later)
   - Download from https://nodejs.org/

2. **Rust** (latest stable)
   - Install from https://rustup.rs/
   - Run: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

3. **Git**
   - Download from https://git-scm.com/

### Platform-Specific Dependencies

#### Windows

- **Microsoft Visual Studio C++ Build Tools**
  - Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
  - Or install Visual Studio 2022 with "Desktop development with C++" workload
- **WebView2** (usually pre-installed on Windows 10/11)
  - If needed, download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

#### macOS

- **Xcode Command Line Tools**
  ```bash
  xcode-select --install
  ```

#### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libgdk-pixbuf-2.0-dev \
  pango1.0-dev \
  atk1.0-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.0-dev
```

#### Linux (Fedora/RHEL/CentOS)

```bash
sudo dnf install -y \
  webkit2gtk4.0-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  gtk3-devel \
  pango-devel \
  atk-devel
```

#### Linux (Arch)

```bash
sudo pacman -Syu
sudo pacman -S --needed \
  webkit2gtk \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  gtk3 \
  libappindicator-gtk3 \
  librsvg \
  libvips \
  pango \
  atk
```

## Building the Application

### 1. Clone the Repository

```bash
git clone https://github.com/justinlevinedotme/FMMLoader-26.git
cd FMMLoader-26
```

### 2. Install Node Dependencies

```bash
npm install
```

### 3. Development Mode

Run the application in development mode with hot-reloading:

```bash
npm run tauri dev
```

This will:
- Start the Vite dev server on http://localhost:1420
- Build the Rust backend
- Launch the Tauri application window

### 4. Production Build

Build the application for distribution:

```bash
npm run tauri build
```

The built application will be in:
- **Windows**: `src-tauri/target/release/FMMLoader26.exe`
- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Linux**: `src-tauri/target/release/bundle/appimage/` or `src-tauri/target/release/bundle/deb/`

## Project Structure

```
FMMLoader-26/
├── src/                    # React frontend source
│   ├── components/         # UI components
│   │   ├── ui/            # shadcn/ui components
│   │   ├── ConflictsDialog.tsx
│   │   ├── DropZone.tsx
│   │   ├── ModMetadataDialog.tsx
│   │   ├── RestorePointsDialog.tsx
│   │   └── UpdateBanner.tsx
│   ├── hooks/             # React hooks
│   │   └── useTauri.ts    # Tauri command wrappers
│   ├── lib/               # Utilities
│   ├── App.tsx            # Main app component
│   └── main.tsx           # App entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── config.rs      # Configuration management
│   │   ├── conflicts.rs   # Conflict detection
│   │   ├── game_detection.rs  # Auto-detect FM26
│   │   ├── import.rs      # Mod import logic
│   │   ├── mod_manager.rs # Core mod operations
│   │   ├── restore.rs     # Backup/restore
│   │   ├── types.rs       # Type definitions
│   │   ├── updater.rs     # GitHub version checking
│   │   └── main.rs        # Tauri commands
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── package.json           # Node dependencies
├── vite.config.ts         # Vite configuration
└── tailwind.config.js     # Tailwind CSS config
```

## Technology Stack

- **Frontend**: React 18 + TypeScript + Vite
- **UI Framework**: Tailwind CSS + shadcn/ui
- **Backend**: Rust (via Tauri v2)
- **Desktop Framework**: Tauri v2

## Features

- ✅ Automatic Football Manager 2026 game detection
- ✅ Drag & drop mod installation
- ✅ Import mods from ZIP files
- ✅ Auto-detect mod types (UI, tactics, graphics, etc.)
- ✅ Mod conflict detection
- ✅ Backup and restore points
- ✅ GitHub update checking
- ✅ Cross-platform (Windows, macOS, Linux)

## Troubleshooting

### Build Errors on Linux

If you get errors about missing libraries, make sure you've installed all the platform-specific dependencies listed above.

### Rust Compilation Issues

If Rust compilation fails:
```bash
rustup update
cargo clean
npm run tauri build
```

### WebView Issues on Windows

If the app doesn't launch on Windows, install WebView2:
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

### Port Already in Use

If port 1420 is already in use during development:
- Kill the process using that port
- Or change the port in `tauri.conf.json` under `build.devUrl`

## Development Tips

### Hot Reload

- Frontend changes (React/CSS) reload automatically
- Backend changes (Rust) require app restart

### Console Logging

- **Frontend**: Check browser devtools (F12 in dev mode)
- **Backend**: Check terminal output where `npm run tauri dev` is running

### Clear App Data

Application data is stored at:
- **Windows**: `%APPDATA%/FMMLoader26/`
- **macOS**: `~/Library/Application Support/FMMLoader26/`
- **Linux**: `~/.local/share/FMMLoader26/`

Delete this folder to reset the app to defaults.

## Building for Distribution

### Code Signing (Optional but Recommended)

#### Windows
- Get a code signing certificate
- Configure in `tauri.conf.json` under `bundle.windows.certificateThumbprint`

#### macOS
- Join Apple Developer Program
- Set signing identity in `tauri.conf.json` under `bundle.macOS.signingIdentity`

### Creating Installers

Tauri automatically creates platform-specific installers:
- **Windows**: `.msi` and `.exe` installers
- **macOS**: `.dmg` and `.app` bundle
- **Linux**: `.AppImage`, `.deb`, and `.rpm` packages

## License

See the LICENSE file in the repository root.

## Support

For issues and questions:
https://github.com/justinlevinedotme/FMMLoader-26/issues
