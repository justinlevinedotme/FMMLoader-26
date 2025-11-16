---
name: h-implement-graphics-pack-support
branch: feature/graphics-pack-support
status: pending
created: 2025-11-16
---

# Graphics Pack Support Implementation

## Problem/Goal
Implement support for large FM graphics packs (often 10GB+ face/logo/kit packs) in a sustainable, efficient way. These packs need to be:
- Extracted from various archive formats (including multi-part archives)
- Properly routed into the FM user directory under `graphics/`
- Organized into correct subdirectories (logos/, faces/, kits/) with preserved structure
- Handled efficiently with progress tracking for large file operations

Graphics packs are fundamental to FM modding and need robust, user-friendly handling in FMMLoader.

## Success Criteria
- [ ] Support extraction of large archives (10GB+) including multi-part .rar/.zip/.7z files
- [ ] Correctly route graphics into appropriate subdirectories (logos/clubs, logos/competitions, faces/, kits/, etc.)
- [ ] Preserve directory structure from source packs without flattening or renaming
- [ ] **Non-blocking architecture**: Use async operations/background tasks to prevent UI freezing during large file operations
- [ ] **Real-time progress feedback**: Show extraction progress, file count, current operation, estimated time
- [ ] **Performance optimized**: Handle 5GB+ packs efficiently even on lower-spec machines
- [ ] **Memory efficient**: Stream large archives instead of loading entirely into memory
- [ ] Validate graphics pack contents before installation
- [ ] Detect and handle conflicts with existing graphics (warn user, offer options)
- [ ] Support rollback/uninstall of graphics packs
- [ ] Auto-detect pack type and structure (faces, logos, kits, mixed)
- [ ] Handle mixed packs (graphics + editor data) with proper type separation
- [ ] Follow ASSET_ROUTING_SPEC.md and TAURI_PLUGIN_STRATEGY.md (prefer existing Tauri plugins)
- [ ] No hardcoded paths - use existing path resolution functions

## Context Manifest

### How Archive Extraction Currently Works: Current ZIP Handling Implementation

**Entry Point:** The current mod import flow starts when a user selects a file via the `import_mod` command in `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/main.rs` (lines 146-258). This command is invoked from the frontend at `/Users/jstn/Documents/GitHub/FMMLoader-26/src/App.tsx` (lines 374-394).

**Current Extraction Flow for ZIP Files:**

When a ZIP file is selected for import, the system performs these operations **synchronously and blocking**:

1. **File Detection** (main.rs lines 170-174): The code checks if the source is a file (not directory) and identifies ZIP files by extension using `source.extension().and_then(|s| s.to_str())`.

2. **Temporary Directory Creation** (main.rs lines 176-177): Creates a temporary directory using `std::env::temp_dir().join(format!("fmmloader_import_{}", uuid::Uuid::new_v4()))`. This pattern is critical - the app already uses temp directories for extraction staging.

3. **ZIP Extraction Call** (main.rs line 178): Invokes `extract_zip(&source, &temp_dir)?` which is defined in `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/import.rs` (lines 6-50).

**The extract_zip Function - Current Blocking Behavior:**

Located in `import.rs`, this function demonstrates the **exact problem** we need to solve:

```rust
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    fs::create_dir_all(dest_dir)?;

    // THIS LOOP BLOCKS THE ENTIRE UI
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        // Directory or file creation
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)?;
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;  // BLOCKING I/O
        }

        #[cfg(unix)]
        { /* Permission handling */ }
    }

    Ok(dest_dir.to_path_buf())
}
```

**Why This Blocks:** The `for i in 0..archive.len()` loop processes every file sequentially. For a 10GB graphics pack with thousands of PNG/XML files, this can take **minutes** with zero user feedback. The `io::copy(&mut file, &mut outfile)` call is pure blocking I/O - the Tauri command handler thread is completely frozen during this operation.

**Current Archive Handling Limitations:**

1. **ZIP Only:** The `zip` crate (v0.6) is the only archive library in use (Cargo.toml line 23). Multi-part RAR/7z files mentioned in the spec are **not currently supported**.

2. **No Progress Tracking:** There's no mechanism to report progress during extraction. The UI shows a loading spinner via `setLoading(true)` (App.tsx line 313), but no percentage/file count updates.

3. **No Streaming:** The entire ZIP is read into memory via `ZipArchive::new(file)`. For multi-gigabyte archives, this can cause memory pressure.

4. **Single-Threaded:** All extraction happens on one thread synchronously.

**Current File Copy Implementation:**

After extraction, files are copied to their final destinations via `copy_dir_recursive` in `mod_manager.rs` (lines 96-129). This function uses `WalkDir` to recursively traverse directories and `fs::copy(path, &target_path)` for each file. This is **also synchronous and blocking**:

```rust
fn copy_recursive(src: &Path, dst: &Path) -> io::Result<u64> {
    let mut count = 0;

    if src.is_dir() {
        fs::create_dir_all(dst)?;

        for entry in WalkDir::new(src) {  // Blocking iteration
            let entry = entry?;
            let path = entry.path();

            if let Ok(rel_path) = path.strip_prefix(src) {
                let target_path = dst.join(rel_path);

                if path.is_dir() {
                    fs::create_dir_all(&target_path)?;
                } else {
                    fs::create_dir_all(parent)?;
                    fs::copy(path, &target_path)?;  // BLOCKING
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}
```

For 10GB of graphics files being moved from temp to `graphics/`, this second blocking operation compounds the UI freeze issue.

### Existing Path Resolution & Routing: How Graphics Must Be Installed

**Path Resolution Functions (game_detection.rs):**

The application has a robust path resolution system that graphics pack support **must use**:

1. **`get_fm_user_dir(custom_path: Option<&str>) -> PathBuf`** (game_detection.rs lines 102-140): Returns the FM26 user directory where graphics must be installed. Platform-specific paths:
   - **macOS:** `~/Library/Application Support/Sports Interactive/Football Manager 26/`
   - **Windows:** `%USERPROFILE%/Documents/Sports Interactive/Football Manager 26/`
   - **Linux:** `~/.local/share/Sports Interactive/Football Manager 26/`

2. **`get_target_for_type(mod_type, game_target, user_dir)`** (mod_manager.rs lines 131-145): Routes content by type. For graphics packs, we use:
   ```rust
   match mod_type {
       "graphics" => user_path.join("graphics"),
       // ... other types
   }
   ```

**Asset Routing Rules (ASSET_ROUTING_SPEC.md):**

Graphics must go to `<FM User Directory>/graphics/` with these **critical requirements**:

- **Preserve directory hierarchy exactly** - Do NOT flatten folder structures
- **Do NOT rename files**
- **Do NOT overwrite without user approval**
- **Common subdirectories:** `graphics/faces/`, `graphics/logos/`, `graphics/kits/`, `graphics/3d kits/`
- **Supported file types:** `.png`, `.jpg`, `.jpeg`, `.tga`, `.bmp`, graphics `.xml` files

**Mixed Content Detection (import.rs lines 88-173):**

The `auto_detect_mod_type` function shows how to identify graphics packs:

```rust
pub fn auto_detect_mod_type(path: &Path) -> String {
    let mut has_graphics = false;

    if let Ok(entries) = walkdir::WalkDir::new(path).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            if entry_path.is_dir() {
                if let Some(name) = entry_path.file_name() {
                    let name_lower = name.to_string_lossy().to_lowercase();

                    // Graphics detection
                    if ["kits", "faces", "logos", "graphics", "badges"].contains(&name_lower.as_str()) {
                        has_graphics = true;
                    }
                }
            }
        }
    }

    if has_graphics {
        return "graphics".to_string();
    }
    // ... other type checks
}
```

This pattern must be preserved - graphics packs often contain `faces/`, `logos/`, `kits/` subdirectories which trigger type detection.

### Tauri Plugin Ecosystem: Available Tools for Non-Blocking Operations

**Currently Installed Tauri Plugins (Cargo.toml lines 14-20):**

```toml
tauri = { version = "2.0.0", features = ["devtools"] }
tauri-plugin-dialog = "2.0.0"
tauri-plugin-fs = "2.0.0"
tauri-plugin-os = "2.0.0"
tauri-plugin-process = "2.0.0"
tauri-plugin-shell = "2.0.0"
tauri-plugin-updater = "2.0.0"
```

**tauri-plugin-shell** is particularly relevant - it allows spawning external processes. However, per TAURI_PLUGIN_STRATEGY.md, we should prefer Tauri core capabilities before adding custom tools.

**Tauri Core Event System (Available but Not Currently Used for Progress):**

Tauri v2 provides `emit()` for sending events from backend to frontend. The frontend already has event listeners set up (App.tsx lines 669-698):

```typescript
const unlistenDrop = await listen<string[]>("tauri://file-drop", (event) => {
    const files = event.payload;
    // ... handle drop
});
```

This pattern shows the app can receive async events. We need to create custom events for extraction progress.

**No Shell Plugin for Archive Extraction Currently:**

While the shell plugin is installed, there's no evidence of using external tools like `7z` or `unrar`. The current ZIP extraction is pure Rust (`zip` crate).

**TAURI_PLUGIN_STRATEGY.md Guidance:**

The spec (lines 64-90) tells us to:
1. Check Tauri core first (events, background tasks)
2. Check official v2 plugins (fs, shell already installed)
3. Only add custom tools if needed

For graphics packs, we should:
- Use Rust's async/await with Tauri commands (no new plugins needed)
- Use Tauri's event system to emit progress updates
- Consider external binaries (via shell plugin) only for RAR/7z if Rust crates prove insufficient

### No Async Patterns Currently Exist: The Synchronous Baseline

**All Tauri Commands Are Synchronous:**

Reviewing all commands in main.rs (lines 23-449), **none use async/await**. Every command is defined as:

```rust
#[tauri::command]
fn some_command(...) -> Result<T, String> {
    // Synchronous operations
}
```

Example - the mod import command (main.rs lines 146-258):
```rust
#[tauri::command]
fn import_mod(
    source_path: String,
    mod_name: Option<String>,
    // ... parameters
) -> Result<String, String> {
    // All blocking operations
    let source = PathBuf::from(&source_path);
    let mod_root = if source.is_file() {
        let temp_dir = std::env::temp_dir().join(...);
        extract_zip(&source, &temp_dir)?;  // BLOCKS HERE
        find_mod_root(&temp_dir)?
    } else {
        find_mod_root(&source)?
    };

    copy_dir_recursive(&mod_root, &dest_dir)?;  // BLOCKS HERE
    Ok(final_mod_name)
}
```

The entire import - from extraction to copying - happens in the command handler thread, blocking the UI until completion.

**Frontend Loading States Are Boolean Only:**

The frontend uses a simple `loading` boolean state (App.tsx lines 100, 313-327):

```typescript
const [loading, setLoading] = useState(false);

const applyMods = async () => {
    try {
        setLoading(true);  // Show spinner
        addLog("Applying mods...");
        toast.loading("Applying mods...", { id: "apply-mods" });
        const result = await tauriCommands.applyMods();
        toast.success("Mods applied successfully!", { id: "apply-mods" });
    } catch (error) {
        toast.error(`Failed to apply mods: ${error}`, { id: "apply-mods" });
    } finally {
        setLoading(false);  // Hide spinner
    }
};
```

There's no progress percentage, no file count updates, no ETAs - just a binary "loading or not loading" state. The `toast.loading()` call shows a generic loading message with no progress indicator.

**Name Fix Example (Largest Current File Operation):**

The name fix feature (name_fix.rs) provides the closest parallel to graphics packs, but it's **still synchronous**:

- Downloads a ZIP from GitHub using `reqwest::blocking::Client` (lines 184-204)
- Extracts files synchronously (lines 206-228)
- Copies files synchronously (lines 956-1021)
- All operations block the command handler

The difference is name fix files are much smaller (~1-10MB), so blocking isn't noticeable. For 10GB graphics packs, this pattern **will not work**.

### What Needs to Change: Required Architectural Additions

To support 10GB+ graphics packs without freezing the UI, we need to introduce:

1. **Async Command Handlers:**
   - Convert blocking commands to `async fn`
   - Use `tokio::task::spawn_blocking` for CPU-intensive operations
   - Use `async-std` or `tokio` for I/O operations

2. **Progress Event System:**
   - Define custom event payloads (e.g., `ExtractionProgress { current: usize, total: usize, current_file: String }`)
   - Emit events from background tasks using Tauri's `app.emit()` or window-specific emit
   - Frontend listens for these events and updates progress UI

3. **Multi-Part Archive Support:**
   - Current `zip` crate doesn't handle multi-part archives
   - Need to evaluate: `compress-tools` (supports RAR/7z via external binaries), `sevenz-rust` (pure Rust 7z), or shell plugin with `7z`/`unrar` CLI tools

4. **Streaming/Chunked Processing:**
   - Instead of loading entire ZIP into memory, process files in chunks
   - For file copying, use buffered I/O with periodic yield points for progress updates

5. **Frontend Progress Components:**
   - Replace boolean `loading` state with structured progress state
   - Add progress bar component (possibly using existing shadcn/ui components)
   - Show: current file being extracted, X/Y files complete, MB transferred, ETA

### Technical Reference Details

#### Current Dependencies (Cargo.toml)

```toml
[dependencies]
zip = "0.6"                    # ZIP extraction only
walkdir = "2"                  # Directory traversal
reqwest = { version = "0.11", features = ["blocking", "json"] }  # HTTP (blocking mode)
uuid = { version = "1", features = ["v4"] }  # Temp directory naming
chrono = "0.4"                 # Timestamps
```

**Missing for graphics packs:**
- No async runtime (tokio/async-std)
- No RAR/7z support
- No progress tracking utilities

#### Key File Paths for Implementation

**Backend (Rust):**
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/import.rs` - Archive extraction logic (must be made async)
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/mod_manager.rs` - File copying logic (must emit progress events)
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/main.rs` - Command handlers (add new async graphics command)
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/types.rs` - Type definitions (add progress event types)

**Frontend (TypeScript/React):**
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/App.tsx` - Main UI (add progress state, event listeners)
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/hooks/useTauri.ts` - Command wrappers (add graphics pack commands)
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/components/` - Create new ProgressDialog.tsx component

**Configuration:**
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/Cargo.toml` - Add async dependencies
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/tauri.conf.json` - Add capabilities if using new plugins

#### Existing Pattern: File Copy with Count Tracking

The `copy_recursive` function (mod_manager.rs lines 96-129) returns a count of files copied:

```rust
pub fn copy_recursive(src: &Path, dst: &Path) -> io::Result<u64> {
    let mut count = 0;

    for entry in WalkDir::new(src) {
        // ... copy logic
        if path.is_file() {
            fs::copy(path, &target_path)?;
            count += 1;  // Increment per file
        }
    }

    Ok(count)
}
```

This count is already tracked - we just need to **emit it periodically** instead of only returning it at the end.

#### Existing Pattern: Temporary Directory Cleanup

The import flow (main.rs lines 176-211) creates temp directories that are automatically cleaned up after manifest generation. We should follow this pattern for graphics extraction:

1. Extract to `std::env::temp_dir().join("fmmloader_graphics_{uuid}")`
2. Validate/detect structure
3. Copy to final destination with progress tracking
4. Clean up temp directory (or let OS handle it on restart)

#### Frontend Event Listening Example

The app already listens for file-drop events (App.tsx lines 669-678):

```typescript
const unlistenDrop = await listen<string[]>("tauri://file-drop", (event) => {
    const files = event.payload;
    if (files && files.length > 0) {
        void handleImport(files[0]);
    }
});
```

We can create custom event listeners following the same pattern:

```typescript
const unlistenProgress = await listen<ProgressEvent>("graphics-extraction-progress", (event) => {
    setProgressState(event.payload);
});
```

#### Toast Notification Pattern

The app uses `sonner` for toast notifications (App.tsx lines 315-327):

```typescript
toast.loading("Applying mods...", { id: "apply-mods" });
// ... operation
toast.success("Mods applied successfully!", { id: "apply-mods" });
```

For graphics packs, we can use the same pattern but update the loading toast with progress:

```typescript
toast.loading(`Extracting: 450/1000 files (45%)`, { id: "graphics-import" });
```

### Performance Considerations from Testing Feedback

**User-Reported Performance Issues:**

From task description:
- 5.3GB FMG graphics pack **struggles on high-spec laptop**
- **No user feedback** during extraction/file moving
- **Frontend freezes** with spinner during processing
- Lower-spec machines will struggle even more

**Architectural Requirements for Performance:**

1. **Non-blocking is critical** - Even on high-spec machines, 5GB+ freezes the UI. Must use async/background tasks.

2. **Memory efficiency** - Streaming extraction instead of loading entire archive prevents out-of-memory errors on 8GB RAM machines.

3. **Feedback every N files** - Don't emit progress events for every single file (too frequent). Emit every 50-100 files or every 50MB to balance responsiveness vs overhead.

4. **Chunked file operations** - For multi-GB file copies, use buffered I/O (e.g., 1MB chunks) to allow yielding to other tasks.

5. **Cancellation support** - Users might want to cancel a 30-minute graphics pack extraction. Async tasks should check for cancellation signals.

### Archive Format Support Requirements

**Multi-Part Archive Handling (ASSET_ROUTING_SPEC.md lines 133-147):**

The spec requires supporting:
```
pack.part01.rar
pack.part02.rar
pack.part03.rar
```

**Current ZIP-Only Limitation:**

The `zip` crate v0.6 does **not** support:
- Multi-part ZIP files
- RAR archives (single or multi-part)
- 7z archives

**Implementation Options:**

1. **Add `compress-tools` crate:** Provides unified interface for ZIP/RAR/7z but requires external binaries (`7z`, `unrar`)
2. **Add `sevenz-rust` crate:** Pure Rust 7z support (no external deps)
3. **Shell plugin approach:** Use `tauri-plugin-shell` to invoke system `7z`/`unrar` commands
4. **Hybrid:** Keep `zip` crate for simple archives, add `compress-tools` for complex multi-part

**Recommendation:** Use `compress-tools` with shell plugin to invoke external tools. Graphics pack extractors often distribute with `7z` command-line tool anyway, and this provides maximum format compatibility without bloating the binary with pure-Rust implementations of every format.

### Graphics Pack Validation Requirements

**Structure Validation (from ASSET_ROUTING_SPEC.md):**

Before installation, must validate:
- Contains image files (`.png`, `.jpg`, `.jpeg`, `.tga`, `.bmp`) or graphics `.xml`
- Has recognizable structure (`faces/`, `logos/`, `kits/` directories)
- No malformed paths (e.g., absolute paths, `..` references)
- No executables or suspicious file types

**Size Validation:**

- Warn if graphics pack >10GB (might be corrupted or wrong file)
- Ensure sufficient disk space before extraction (check temp dir + final destination)

**Conflict Detection:**

The app has existing conflict detection (conflicts.rs, main.rs lines 272-283). For graphics packs:
- Check if `graphics/` directory already has content
- Warn about potential overwrite conflicts
- Offer backup before installation (similar to name_fix backup pattern)

### Existing Backup/Restore Pattern (from name_fix.rs)

The name fix feature demonstrates a backup pattern we should replicate:

```rust
fn create_backups(db_dir: &Path) -> Result<(), String> {
    let app_data_dir = get_app_data_dir();
    let backup_dir = app_data_dir.join("name_fix_backup");

    // Clean up old backup if it exists
    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)?;
    }

    fs::create_dir_all(&backup_dir)?;

    // Copy files to backup
    for file in files_to_backup {
        fs::copy(&source_file, &backup_file)?;
    }

    Ok(())
}
```

For graphics packs:
- Create `graphics_backup_{timestamp}` in app data directory
- Back up existing `graphics/` content before installing new pack
- Maintain restore points (similar to existing restore_points feature in restore.rs)
- Allow rollback via "Restore" dialog (already exists in UI)

### Summary: Implementation Strategy

**Phase 1: Add Async Infrastructure**
- Add tokio to Cargo.toml
- Convert `extract_zip` to async
- Create progress event types
- Emit basic progress events

**Phase 2: Frontend Progress UI**
- Add progress state to App.tsx
- Create ProgressDialog component
- Listen for extraction progress events
- Update UI with file count, percentage, current file

**Phase 3: Multi-Format Support**
- Add `compress-tools` crate
- Implement multi-part archive detection
- Use shell plugin to invoke `7z`/`unrar` for complex archives
- Maintain ZIP-only fast path for simple archives

**Phase 4: Optimization & Validation**
- Implement streaming extraction for memory efficiency
- Add cancellation support
- Add pre-installation validation (structure, file types, disk space)
- Implement backup/restore for graphics directory

**Phase 5: Graphics-Specific Routing**
- Respect graphics subdirectory structure (`faces/`, `logos/`, `kits/`)
- Handle mixed packs (graphics + editor data)
- Implement conflict detection specific to graphics files
- Add verification after installation

## User Notes
- Consult aicontext/ASSET_ROUTING_SPEC.md for routing rules
- Consult aicontext/TAURI_PLUGIN_STRATEGY.md for implementation approach
- Prefer existing Tauri v2 plugins before writing custom tools
- Must preserve exact directory hierarchy from packs
- Must handle multi-part archives gracefully

**Performance Feedback from Testing:**
- Current implementation struggles with 5.3GB FMG graphics pack on high-spec laptop
- No user feedback during extraction/file moving operations
- Frontend freezes with spinner during processing
- Need subprocess/async approach to prevent UI blocking
- Lower-spec machines will struggle even more - optimization critical

## Work Log
- [2025-11-16] Created task structure
