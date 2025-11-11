# FMMLoader26 - AI Copilot Instructions

## Project Overview

**FMMLoader26** is a cross-platform **Tauri v2 + React** application for managing Football Manager 2026 mods. It provides a modern UI for installing, enabling/disabling, and managing mod conflicts with automatic game detection, backups, and restore points.

### Tech Stack

- **Frontend:** React 18 + TypeScript + Vite + Tailwind CSS + shadcn/ui
- **Backend:** Rust (Tauri v2) for native performance
- **Architecture:** Tauri IPC commands bridge React frontend → Rust backend

---

## Essential Architecture

### Data Flow (Frontend → Backend)

```
React Component → tauriCommands (hooks/useTauri.ts)
  → invoke() Tauri IPC → @tauri::command in Rust
  → File system/Config operations → JSON response
```

**Key Pattern:** All backend operations go through `tauriCommands` wrapper in `src/hooks/useTauri.ts`. This provides type-safe Tauri command invocations.

### Core Rust Modules (src-tauri/src/)

| Module              | Purpose                                                                  |
| ------------------- | ------------------------------------------------------------------------ |
| `main.rs`           | Tauri command definitions (`#[tauri::command]`), app initialization      |
| `mod_manager.rs`    | Core mod installation logic (copy_recursive, install_mod, read manifest) |
| `config.rs`         | Config persistence (JSON file in platform-specific app data dirs)        |
| `game_detection.rs` | Platform-specific FM26 path detection (Steam/Epic/GamePass/Linux)        |
| `conflicts.rs`      | Detects when multiple enabled mods modify same files                     |
| `types.rs`          | Core data structures (ModManifest, Config, ConflictInfo)                 |
| `import.rs`         | Handles ZIP extraction, mod validation, manifest auto-generation         |
| `restore.rs`        | Backup/restore point creation and rollback logic                         |
| `logging.rs`        | Structured logging with file output                                      |

### Critical Type Definitions (types.rs)

```rust
pub struct ModManifest {
  pub name: String,
  pub mod_type: String,  // "ui", "bundle", "tactics", "graphics", "skins", "editor-data"
  pub files: Vec<FileEntry>,  // Maps source to target_subpath
}

pub struct Config {
  pub target_path: Option<String>,  // FM26 game folder
  pub user_dir_path: Option<String>,  // User data folder
  pub enabled_mods: Vec<String>,  // Ordered list of enabled mod names
  pub dark_mode: bool,
}
```

### Load Order & Conflicts

- **Load Order = Last Wins:** Mods are listed in order; later mods in `config.enabled_mods` override earlier ones
- **Conflict Detection:** `conflicts.rs::find_conflicts()` builds a map of file → mods touching that file
- **Ordering:** Mods modify their position by adding/removing from the middle of `enabled_mods` vector

---

## Frontend Component Structure

### Main App Component (src/App.tsx)

- Manages global state: `config`, `mods`, `selectedMod`, `logs`, `updateInfo`
- Tauri event listeners for real-time logging
- Tabs: Mods List | Logs | Settings (target path, user dir, dark mode)

### Key Components

| Component                 | Purpose                                                               |
| ------------------------- | --------------------------------------------------------------------- |
| `TitleBar.tsx`            | Custom window chrome + buttons (minimize, maximize, close)            |
| `ModMetadataDialog.tsx`   | Form for user-provided mod metadata when manifest is missing          |
| `ConflictsDialog.tsx`     | Shows conflicting files + ability to disable mods or ignore conflicts |
| `RestorePointsDialog.tsx` | Lists and rolls back to restore points                                |
| `UpdateBanner.tsx`        | Notifies user of new releases with download link                      |
| `DropZone.tsx`            | Drag-and-drop zone for importing mod ZIPs                             |
| `ui/*`                    | shadcn/ui components (Button, Card, Dialog, Table, Tabs, etc.)        |

### UI Library Integration

- Uses `shadcn/ui` components (already configured in `src/components/ui/`)
- Styling via Tailwind CSS (`tailwind.config.js` includes full shadcn/ui preset)
- Icons from `lucide-react` + `react-icons`

---

## Critical Developer Workflows

### Local Development

```bash
npm install                    # Install Node deps
npm run dev                    # Tauri dev mode with hot reload
npm run tauri dev              # Alternative command
```

**Dev mode:**

- Vite dev server on `http://localhost:1420`
- Live reload + debugging console
- Frontend changes hot-reload without full recompile

### Building

```bash
npm run build:debug            # Fast build for testing (~2-5 min)
npm run build:release          # Optimized build (~10 min)
npm run tauri build            # Full build with all installers
```

**Build output:**

- Debug: `src-tauri/target/debug/` (executable) + `bundle/` (installers)
- Release: `src-tauri/target/release/` + `bundle/`
- Installers: Windows (.msi), macOS (.dmg), Linux (.AppImage, .deb)

### Key Build Files

- `src-tauri/tauri.conf.json` - Tauri config (window, bundle settings, plugins)
- `vite.config.ts` - Frontend build config
- `src-tauri/Cargo.toml` - Rust dependencies + version

---

## Project-Specific Conventions & Patterns

### Storage Structure

**Config Location (platform-specific):**

```
Windows:   %APPDATA%\FMMLoader26\config.json
macOS:     ~/Library/Application Support/FMMLoader26/config.json
Linux:     ~/.local/share/FMMLoader26/config.json
```

**App Data Directories:**

```
{app-data}/
  ├── config.json           # User configuration
  ├── mods/                 # Extracted mod folders (each has manifest.json)
  ├── backups/              # Pre-apply file backups
  ├── restore_points/       # Full game folder snapshots (keeps last 10)
  └── logs/                 # Daily log files
```

### Mod Installation Logic (mod_manager.rs)

1. **Read manifest** from mod folder
2. **Determine target** based on `mod_type`:
   - `ui`/`bundle` → Game data folder
   - `tactics` → User data/tactics/
   - `graphics` → User data/graphics/
   - `skins` → User data/skins/
   - `editor-data` → User data/editor data/
3. **Copy files** via `copy_recursive()` from mod to target, **last mod wins on conflicts**
4. **Create restore point** before applying changes (full backup of game folder)

### Config Persistence

- Loaded on app init via `init_app()`
- Any state change calls `update_config(config)` Tauri command
- Config is JSON-serialized with `serde`

### Error Handling Pattern

```rust
// Rust errors are returned as Result<T, String>
pub fn some_operation() -> Result<String, String> {
  fs::read_to_string(&path)
    .map_err(|e| format!("Failed to read: {}", e))?
}

// React handles errors via try/catch + toast notification
try {
  await tauriCommands.someOperation();
  toast.success("Done");
} catch (error) {
  toast.error(formatError(error));
}
```

### Game Detection (Cross-Platform)

**Windows:** Checks Steam (Program Files x86), Epic (Program Files), GamePass (C:/D:/E: drives)
**macOS:** Checks Steam and Epic in ~/Library/Application Support/
**Linux:** Checks ~/.local/share/Steam and /run/media/ (Steam Deck)

All paths look for FM26 in specific subdirectories (e.g., `fm_Data/StreamingAssets/aa/StandaloneWindows64`)

---

## Integration Points & Data Flows

### Apply Mods Flow

1. User clicks "Apply" → calls `apply_mods()` Tauri command
2. Rust checks if FM is running (prevent mid-session changes)
3. `create_restore_point()` backs up entire game folder
4. `find_conflicts()` detects conflicting mods
5. If conflicts found, show `ConflictsDialog` (user can disable mods or proceed)
6. `install_mod()` iterates enabled mods in order, copying files (last wins)
7. Return success + list of applied mods

### Import Mod Flow

1. User drags ZIP or selects folder
2. `extract_zip()` or validation of folder structure
3. `find_mod_root()` locates manifest or mod files
4. If no manifest, prompt via `ModMetadataDialog` for metadata
5. `generate_manifest()` creates manifest.json
6. Copy mod to `{app-data}/mods/{mod-name}/`
7. Refresh mod list

---

## Common Pitfalls & Solutions

### When Adding Tauri Commands

1. Define in `main.rs` with `#[tauri::command]` attribute
2. Add to `tauriCommands` object in `useTauri.ts` with matching signature
3. Import in React component via `tauriCommands.yourNewCommand()`
4. **Always** handle errors with try/catch or .catch() for promises

### When Modifying File Paths

- **Never hardcode paths** - use `config.rs` functions:
  - `get_app_data_dir()` for app data
  - `get_mods_dir()` for mods folder
  - `get_target_for_type()` to route mods by type
  - `get_fm_user_dir()` for FM user data (respects custom user dir)

### When Updating Mod Installation Logic

- Remember: **Last mod in load order wins** (modify `enabled_mods` order carefully)
- Always create restore point before applying (safety first)
- Test on all three platforms (Windows path separators differ)

### When Adding UI Elements

- Import from `src/components/ui/` (pre-built shadcn components)
- Use Tailwind classes (no raw CSS needed)
- Wrap interactive elements with `TooltipProvider` for tooltips
- Use `sonner` toast for notifications: `toast.success()`, `toast.error()`

---

## Key Dependencies & Their Roles

| Package                  | Purpose             | Usage                                |
| ------------------------ | ------------------- | ------------------------------------ |
| `@tauri-apps/api`        | IPC bridge          | `invoke()` commands, file operations |
| `react` + `@types/react` | UI framework        | Component state, hooks               |
| `tailwindcss`            | Styling             | CSS classes (no raw CSS)             |
| `lucide-react`           | Icons               | Clean, consistent icon library       |
| `sonner`                 | Toasts              | User notifications                   |
| `serde`/`serde_json`     | Serialization       | Config/manifest JSON handling (Rust) |
| `zip`                    | Archive handling    | Extract imported mod ZIPs (Rust)     |
| `walkdir`                | Directory traversal | Recursive file operations (Rust)     |

---

## Configuration & Build Files to Know

- **tauri.conf.json:** Window settings (1200x800, no decorations, overlay title bar), bundle targets (nsis/dmg/appimage)
- **tsconfig.json:** Path alias `@/` → `src/`
- **tailwind.config.js:** Includes shadcn/ui preset + extends with dark mode
- **postcss.config.js:** Processes Tailwind CSS
- **Cargo.toml:** Rust v2021 edition, key deps: tauri, serde, zip, walkdir

---

## Testing & Debugging Tips

### Frontend

- Open dev console: Press F12 in dev mode window
- Check Tauri logs in terminal running `npm run dev`
- Use React DevTools browser extension (works in Tauri windows)

### Backend

- Add `tracing::info!()` or `tracing::error!()` statements
- Logs appear in terminal + `{app-data}/logs/` files
- Use `RUST_LOG=debug` environment variable for verbose output

### Cross-Platform Testing

- Windows: Test Game Pass path detection (check multiple drives)
- macOS: Test Gatekeeper bypass handling, path with spaces
- Linux: Verify Steam Deck path `/run/media/` detection
