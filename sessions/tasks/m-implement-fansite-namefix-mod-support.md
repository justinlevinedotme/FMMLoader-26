---
name: m-implement-fansite-namefix-mod-support
branch: feat/multiple-namefix-support
status: done
created: 2025-11-11
completed: 2025-11-13
---

# Implement Fansite Name Fix Mod Support

## Problem/Goal
Extend the existing name fix mod support in FMMLoader (Football Manager Mod Loader) to handle additional name fix mods from various fansite sources. Currently, the loader supports one open source name fix mod via GitHub download. This task adds the capability to import and manage name fix mods from RAR or ZIP files (downloaded by users from fansites) alongside the existing GitHub one.

## Success Criteria
- [x] FMMLoader can import name fix mods from ZIP files via file picker
- [x] Multiple name fix sources can be stored and managed (user can switch between them)
- [x] Users can see which name fix is currently installed and select from available ones
- [x] Existing GitHub name fix mod support continues to work without regression

## Context Manifest

### How the Current Name Fix System Works

**Architecture Overview:**

FMMLoader26 is a Tauri v2 application with a React/TypeScript frontend and Rust backend. The application manages Football Manager 2026 mods by copying files to specific game directories based on mod type. The current name fix implementation is a specialized feature that operates differently from regular mods—it directly modifies the game's database files rather than being managed through the standard mod system.

**Current Name Fix Implementation Flow:**

When a user clicks "Install Name Fix" in the Utilities tab, the application initiates a multi-step process that downloads, extracts, and installs a single open-source name fix mod from GitHub. Here's the complete flow:

1. **Frontend Trigger** (App.tsx lines 445-460): The user clicks the "Install Name Fix" button, which calls `tauriCommands.installNameFix()`. This invokes the Rust backend command `install_name_fix` through Tauri's IPC bridge.

2. **Backend Processing** (name_fix.rs): The Rust backend handles the entire installation:
   - **Download Phase** (lines 179-202): Uses `reqwest` HTTP client to download a ZIP archive from a hardcoded GitHub URL: `https://github.com/jo13310/NameFixFM26/archive/refs/tags/v1.0.zip`
   - **Extraction Phase** (lines 204-226): Opens the ZIP archive using the `zip` crate, searches for the specific file `FM26-open-names.lnc`, and extracts it to memory
   - **Backup Creation** (lines 228-266): Before making any changes, creates a backup of all files that will be deleted or modified in `{AppData}/FMMLoader26/name_fix_backup/`
   - **Installation Phase** (lines 334-365):
     - Writes the extracted `.lnc` file to `{GameDB}/lnc/all/FM26-open-names.lnc`
     - Calls `delete_licensing_files()` to remove 27 specific licensing files across three subdirectories
   - **Database Directory Resolution** (lines 52-168): The `get_db_dir()` function navigates from the game's StreamingAssets target path to the database directory, with platform-specific logic for Windows/macOS/Linux

3. **File Operations**: The name fix modifies the game's database by:
   - Adding one new file: `FM26-open-names.lnc` in the `lnc/all/` directory
   - Deleting 27 licensing files from three subdirectories:
     - 12 files from `lnc/all/` (licensing club names, fake names, unlicensed content)
     - 1 file from `edt/permanent/` (fake.edt)
     - 14 files from `dbc/permanent/` and `dbc/language/` (licensing databases)

4. **Status Checking** (lines 171-176): The `check_installed()` function simply checks if `FM26-open-names.lnc` exists in the expected location to determine installation status.

5. **Uninstallation** (lines 381-391): Restores all backed-up files and removes the installed `.lnc` file, completely reversing the installation.

**Key Architectural Decisions:**

The name fix is implemented as a standalone utility rather than a regular mod because:
- It requires precise file deletion (not just addition)
- It operates on database files in a different directory structure than regular mods
- It needs special backup/restore logic separate from the mod manager's restore points
- It's a "set it and forget it" utility rather than something users toggle frequently

**Database Directory Structure:**

The FM26 database lives at: `{GameRoot}/shared/data/database/db/2600/` on Windows/Linux, or within the macOS app bundle at a deeply nested path. Within this directory:
- `lnc/all/` - Language/name correction files (.lnc format)
- `edt/permanent/` - Editor data files (.edt format)
- `dbc/permanent/` and `dbc/language/` - Database change files (.dbc format)

**Configuration and State Management:**

The application stores its configuration in `{AppData}/FMMLoader26/config.json` with the structure defined in types.rs. This includes:
- `target_path`: The game's StreamingAssets directory (required for name fix)
- `user_dir_path`: The FM user directory for save files/mods
- `enabled_mods`: Array of enabled mod names (name fix is separate from this)
- `dark_mode`: UI preference

**Frontend UI Integration:**

The name fix UI is in the "Utilities" tab (App.tsx lines 868-990) and shows:
- Installation status (installed/not installed) with icons
- Description of what it fixes (AC Milan, Inter, Lazio, Japanese names, etc.)
- Install/Uninstall buttons with loading states
- Link to the source GitHub repository
- Warning if game directory isn't set

The frontend maintains three state variables for the name fix:
- `nameFixInstalled`: Boolean showing current installation status
- `checkingNameFix`: Boolean for loading state during status checks
- `installingNameFix`: Boolean for loading state during install/uninstall

### For New Feature Implementation: Multiple Name Fix Sources

**Challenge:**

Currently, the system is hardcoded to support exactly one name fix mod from a single GitHub URL. We need to extend this to support multiple name fix mods imported from local RAR/ZIP files (that users download from fansites) while maintaining the existing GitHub one.

**Required Changes:**

1. **Configuration Extension**: The config.rs needs to store information about multiple name fix sources. This could be a list of name fix configurations, each containing:
   - Source name/identifier
   - Local file path or stored archive location
   - Expected filename(s) extracted from the archive
   - Installation status
   - Metadata (description, author, source URL for reference)

2. **Backend Refactoring**: The name_fix.rs module needs significant changes:
   - Add import logic to handle local RAR/ZIP files (similar to regular mod import)
   - Add RAR extraction support (currently only ZIP is supported)
   - Support multiple name fix sources stored locally (library of name fixes)
   - Track which name fix is currently installed in the backup directory
   - Handle different file structures—not all name fixes may be single .lnc files
   - Modify the FILES_TO_DELETE list or make it configurable per name fix source

3. **Installation Strategy**: We need to decide:
   - Can multiple name fixes coexist? (Likely NO—they probably conflict by design)
   - If mutually exclusive, should installing one auto-uninstall others?
   - How to handle backups when switching between name fixes?
   - Should we restore original files before installing a different name fix?

4. **Frontend UI Changes**: The Utilities tab needs to:
   - Show a list of imported name fix mods (plus the built-in GitHub one)
   - Display which one (if any) is currently installed
   - Allow users to switch between them
   - Add import functionality (drag-and-drop or file picker) to add new name fixes
   - Show metadata for each name fix (name, author, source)

5. **Import and Storage Management**: Need to determine:
   - Where to store imported name fix archives? (In FMMLoader26 AppData directory?)
   - How to detect/identify what's in a name fix archive?
   - Should we validate/verify name fix packages before accepting them?
   - How to handle metadata for imported name fixes (manual entry vs auto-detection)?

**Integration Points:**

- `name_fix.rs`: Core installation logic must become source-agnostic
- `types.rs`: Add new types for NameFixSource, NameFixConfig
- `config.rs`: Store available name fix sources and installation state
- `App.tsx`: UI for managing multiple name fix sources
- `useTauri.ts`: New commands for listing/selecting name fix sources

**Potential Pitfalls:**

- Name fixes from different sources may have conflicting file deletions
- RAR format support requires additional dependencies (need to add RAR extraction crate)
- Archive structures may vary between sources (nested folders, different filenames)
- Users may import non-name-fix archives by mistake
- Users may try to install incompatible name fixes simultaneously
- Storage management: imported archives take up disk space

### Technical Reference Details

#### Core Function Signatures

**Rust Backend (name_fix.rs):**

```rust
// Current public interface
pub fn check_installed(target_path: Option<&str>) -> Result<bool, String>
pub fn install() -> Result<String, String>
pub fn uninstall() -> Result<String, String>

// Internal helpers that will need refactoring
fn get_db_dir(target_path: Option<&str>) -> Result<PathBuf, String>
fn download_name_fix() -> Result<Vec<u8>, String>  // Keep for GitHub one
fn import_name_fix_archive(source_path: &str) -> Result<NameFixData, String>  // NEW
fn extract_from_zip(zip_data: &[u8]) -> Result<Vec<u8>, String>
fn extract_from_rar(rar_path: &str) -> Result<Vec<u8>, String>  // NEW
fn create_backups(db_dir: &Path) -> Result<(), String>
fn restore_from_backup(db_dir: &Path) -> Result<(), String>
fn delete_licensing_files(db_dir: &Path) -> Result<(), String>

// Constants that will need to become configurable per name fix source
const NAME_FIX_RELEASE_URL: &str = "..."  // Still used for built-in GitHub one
const NAME_FIX_FILE: &str = "FM26-open-names.lnc"  // Default, may vary
const FILES_TO_DELETE: &[(&str, &[&str])] = &[...]  // May vary per source
```

**Tauri Commands (main.rs lines 350-363):**

```rust
#[tauri::command]
fn check_name_fix_installed() -> Result<bool, String>

#[tauri::command]
fn install_name_fix() -> Result<String, String>

#[tauri::command]
fn uninstall_name_fix() -> Result<String, String>
```

**Frontend TypeScript (useTauri.ts lines 140-145):**

```typescript
checkNameFixInstalled: () => safeInvoke<boolean>("check_name_fix_installed")
installNameFix: () => safeInvoke<string>("install_name_fix")
uninstallNameFix: () => safeInvoke<string>("uninstall_name_fix")
```

#### Data Structures

**Current Config (types.rs):**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub target_path: Option<String>,
    pub user_dir_path: Option<String>,
    pub enabled_mods: Vec<String>,
    pub dark_mode: bool,
}
```

**Mod Manifest Structure (types.rs) - for reference:**

```rust
pub struct ModManifest {
    pub name: String,
    pub version: String,
    pub mod_type: String,
    pub author: String,
    pub homepage: String,
    pub description: String,
    pub license: String,
    pub compatibility: Compatibility,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub load_after: Vec<String>,
    pub files: Vec<FileEntry>,
}

pub struct FileEntry {
    pub source: String,
    pub target_subpath: String,
    pub platform: Option<String>,
}
```

#### File System Locations

**Application Data Directory:**
- Windows: `%APPDATA%/FMMLoader26/`
- macOS: `~/Library/Application Support/FMMLoader26/`
- Linux: `~/.local/share/FMMLoader26/`

Contains:
- `config.json` - Application configuration
- `mods/` - Regular mod storage
- `backups/` - File backups from mod operations
- `logs/` - Application logs
- `restore_points/` - Full restore point snapshots
- `name_fix_backup/` - Name fix specific backups

**Game Database Directory (from target_path):**
- Windows: `{GameRoot}/shared/data/database/db/2600/`
- macOS: `{fm.app}/Contents/PlugIns/game_plugin.bundle/Contents/Resources/shared/data/database/db/2600/`
- Linux: `{GameRoot}/shared/data/database/db/2600/`

#### Implementation Locations

**Files to Modify:**
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/name_fix.rs` - Core name fix logic
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/types.rs` - Add new data structures
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/config.rs` - Persist name fix sources
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/main.rs` - Add new Tauri commands
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/hooks/useTauri.ts` - Frontend command interfaces
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/App.tsx` - UI for multiple name fixes

**Dependencies Already Available:**
- `reqwest` (0.11) - HTTP downloads with blocking client (for built-in GitHub name fix)
- `zip` (0.6) - ZIP archive extraction
- `serde`/`serde_json` - Serialization for configs
- `tracing` - Logging throughout

**Dependencies Needed:**
- RAR extraction crate (e.g., `unrar` or `compress-tools`) - For RAR file support

#### Error Handling Patterns

The codebase uses `Result<T, String>` consistently, with errors formatted as user-friendly strings. The frontend displays these via toast notifications (using sonner) and logs them to the activity log. All file operations are wrapped in `.map_err()` calls that format IO errors into descriptive strings.

#### Architectural Patterns to Follow

1. **Separation of Concerns**: Keep download/extraction logic separate from installation logic
2. **Backup Before Modify**: Always create backups before any destructive operations
3. **Atomic Operations**: Use temp directories for downloads, only move to final location on success
4. **Platform Awareness**: Use `#[cfg(target_os = "...")]` for platform-specific paths
5. **Logging**: Use `tracing::info!()` and `tracing::debug!()` throughout for debugging
6. **Config Persistence**: All persistent state goes through config.rs save/load functions

## User Notes

**Implementation Approach Clarification:**
This feature is based on importing local RAR/ZIP files that users have already downloaded from fansites, NOT downloading directly from fansite URLs. The workflow is:
1. User downloads name fix from a fansite (gets a RAR or ZIP file)
2. User drags/drops the archive into FMMLoader or uses file picker
3. FMMLoader imports and stores the name fix
4. User can switch between installed name fixes (including the built-in GitHub one)

This is similar to how regular mod imports work, but adapted for the name fix system.

## Work Log
<!-- Updated as work progresses -->
- [2025-11-13] Task completed successfully
  - Phase 1: Backend implementation
    - Added NameFixSource and NameFixSourceType types
    - Created name_fixes storage directory
    - Implemented import, list, install, and delete functions
    - Added Tauri commands for all operations
    - Maintained backwards compatibility
  - Phase 2: Frontend implementation
    - Added source selection dropdown UI
    - Implemented import from ZIP functionality
    - Added delete source capability
    - Display active name fix status
    - Integrated with existing Utilities tab
  - All success criteria met
  - Note: Drag-and-drop not implemented (file picker works well)
  - Note: RAR support deferred (ZIP-only for now, can add RAR later if needed)
