# FMMLoader26 - Project Documentation

This is a Football Manager 2026 mod manager built with Tauri v2 + React. This file provides high-level architectural guidance for developers and AI assistants working on this codebase.

## Architecture Overview

### Tech Stack
- **Frontend**: React 18, TypeScript, Vite, Tailwind CSS, shadcn/ui
- **Backend**: Rust, Tauri v2
- **Key Dependencies**: tokio (async runtime), zip (archive handling), walkdir (directory traversal)

### Core Capabilities

1. **Mod Management**: Import, enable/disable, and apply mods with automatic backup/restore
2. **Graphics Pack Support**: Large-scale graphics pack installation with intelligent type detection
3. **Name Fix System**: Support for FMScout and Sortitoutsi name fixes
4. **Conflict Detection**: Identify and resolve file conflicts between mods
5. **Asset Routing**: Automatically route different content types to correct FM26 directories

## Graphics Pack Support (Added 2025-11-17)

FMMLoader supports installation of large FM graphics packs (faces, logos, kits) with the following features:

### Core Components

**Graphics Analyzer Module** (`src-tauri/src/graphics_analyzer.rs`):
- Analyzes pack contents and detects type (Faces, Logos, Kits, Mixed, Unknown)
- Confidence scoring based on directory structure and config.xml analysis
- Supports both flat packs (PNGs at root) and structured packs (with subdirectories)
- Mixed pack splitting - separates multi-type packs into type-specific directories

**Async Import with Progress** (`src-tauri/src/import.rs`):
- Asynchronous ZIP extraction using tokio to prevent UI freezing
- Real-time progress tracking via Tauri events
- Zip bomb protection (50GB max size, 500k files max)
- Progress updates every 100 files for optimal performance

**Graphics Pack Registry** (`src-tauri/src/config.rs`):
- Tracks installed graphics packs with metadata (name, type, file count, install date)
- Persisted to config.json for session persistence
- Referenced in `src-tauri/src/types.rs` as GraphicsPackMetadata and GraphicsPacksRegistry

### Tauri Commands

**analyze_graphics_pack** (async):
- Analyzes ZIP file contents to determine pack type
- Returns GraphicsPackAnalysis with type, confidence, file count, size
- Used by frontend to show confirmation dialog before installation

**import_graphics_pack_with_type** (async):
- Installs graphics pack to specified directory
- Emits progress events during extraction and copying
- Handles conflict detection and pack registration

**list_graphics_packs**:
- Returns list of installed graphics packs from registry

**validate_graphics**:
- Scans existing graphics directory for misplaced packs
- Returns list of issues with suggested corrections

**migrate_graphics_pack** (async):
- Moves graphics pack to correct subdirectory
- Creates backup before migration

### User Experience

**GraphicsPackConfirmDialog** (`src/components/GraphicsPackConfirmDialog.tsx`):
- Shows detected pack type with confidence percentage
- Displays full installation path preview
- Warns users about low-confidence detections (<50%)
- Offers split option for mixed packs

**Progress Tracking** (`src/App.tsx`):
- Listens for "graphics-extraction-progress" events
- Shows file count and percentage during extraction/copying
- Updates toast notifications with progress

### Security & Performance

**Zip Bomb Protection**:
- 50GB maximum extraction size limit
- 500,000 file count limit
- Early termination on limit exceeded

**Memory Efficiency**:
- Extraction runs in spawn_blocking to avoid blocking async runtime
- Progress events emitted every 100 files (not per file)
- Temporary directory cleanup after analysis

**Disk Space Protection**:
- Estimates extraction size (5x compression ratio)
- Warns if pack will extract to >20GB

### Integration Points

Graphics packs follow the asset routing rules defined in `aicontext/ASSET_ROUTING_SPEC.md`:
- Graphics assets route to `<FM User Directory>/graphics/`
- Subdirectories: `faces/`, `logos/`, `kits/`
- Directory hierarchy preserved exactly (no flattening)
- File naming preserved (no renaming)

Path resolution uses existing functions from `src-tauri/src/game_detection.rs`:
- `get_fm_user_dir()` - Returns platform-specific FM26 user directory
- Paths are never hardcoded - always resolved at runtime

## Async Architecture

Graphics pack support introduced the first async Tauri commands in the codebase. Pattern for future async operations:

```rust
#[tauri::command]
async fn async_operation(app: tauri::AppHandle, param: String) -> Result<T, String> {
    // Use tokio::task::spawn_blocking for CPU-intensive work
    tokio::task::spawn_blocking(move || {
        // Blocking operations here
        // Emit progress events via app.emit()
    }).await.map_err(|e| e.to_string())?
}
```

Event emission pattern:
```rust
app.emit("event-name", ProgressPayload { /* data */ })
    .map_err(|e| e.to_string())?;
```

Frontend event listening:
```typescript
const unlisten = await listen<ProgressPayload>("event-name", (event) => {
    // Handle progress update
});
```

## Development Guidelines

### For Graphics Pack Features

- Reference `sessions/tasks/h-implement-graphics-pack-support/README.md` for implementation context
- Test with packs >5GB to verify async performance
- Ensure progress events don't fire too frequently (use counters, emit every N files)
- Always clean up temporary directories after operations
- Use existing path resolution functions - never hardcode paths

### For New Tauri Plugins

Follow `aicontext/TAURI_PLUGIN_STRATEGY.md`:
1. Check if Tauri core solves it (events, dialogs, webviews)
2. Check official Tauri v2 plugins (fs, shell, process, etc.)
3. Check community plugins
4. Only then write custom commands

### For Asset Routing

Consult `aicontext/ASSET_ROUTING_SPEC.md` for:
- FM User Directory locations per platform
- Asset type detection rules
- Installation path routing
- File type validation

## Additional Guidance

@sessions/CLAUDE.sessions.md

This file provides instructions for Claude Code for working in the cc-sessions framework.
