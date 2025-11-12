---
name: m-fix-update-logging-improvements
branch: fix/m-fix-update-logging-improvements
status: pending
created: 2025-11-11
---

# Fix Update Logging with Robust Console and File Logging

## Problem/Goal
The current update system lacks comprehensive logging for version information. We need to enhance the update logging to provide more robust console output and file logging that includes the current app version and the version found in the latest.json release file.

## Success Criteria
- [ ] Console logs display current app version and latest available version
- [ ] Update process logs to file with version information
- [ ] Logging includes clear information from latest.json in release
- [ ] Log messages are clear, informative, and help with debugging update issues

## Context Manifest

### How the Update System Currently Works

The FMMLoader26 application has a dual-architecture update checking system that operates across both the React frontend (TypeScript) and Rust backend, with update checking handled entirely on the frontend using the Tauri plugin-updater package.

**Update Check Flow - Frontend (TypeScript):**

When the application starts, the `useUpdater` hook (located at `/Users/jstn/Documents/GitHub/FMMLoader-26/src/hooks/useUpdater.ts`) initializes and automatically triggers an update check after a 3-second delay. This hook manages all update-related state including checking status, download progress, installation status, and maintains an in-memory logs array for the UI.

The update check process begins by calling the `check()` function from `@tauri-apps/plugin-updater` (version 2.9.0). This function reaches out to the configured endpoint at `https://github.com/justinlevinedotme/FMMLoader-26/releases/latest/download/latest.json` (configured in `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/tauri.conf.json` at lines 63-67). The endpoint URL is hardcoded in both the Tauri config and is also logged at application startup in the Rust backend.

The `check()` function returns either `null` (if no update is available) or an `Update` object with the following structure based on the TypeScript definitions:

```typescript
class Update {
  currentVersion: string;    // Current running app version
  version: string;           // Latest available version from latest.json
  date?: string;            // Release date from latest.json
  body?: string;            // Release notes/body from latest.json
  rawJson: Record<string, unknown>;  // Complete JSON from latest.json
  // ... methods for download/install
}
```

**Current Logging Implementation:**

The `useUpdater` hook has an `addLog` callback (lines 30-38) that creates timestamped log messages. Each log message follows the format `[HH:MM:SS] message text`. These logs serve two purposes:

1. **Console Output**: Every log is written to the browser console via `console.log()` with a `[Updater]` prefix
2. **In-Memory State**: Logs are appended to the `status.logs` array which is displayed in the UI via the UpdateBanner component

Currently, the logs include:
- "Manual update check initiated by user" or "Automatic update check started"
- The endpoint URL being checked
- "No update available - app is up to date" (when update is null)
- When update is found: current version, latest version, release date, and update body
- Download progress information (bytes downloaded, percentage)
- Installation status messages
- Error messages with full error details

**What's Missing - File Logging:**

The current implementation logs to console and maintains in-memory logs for UI display, but these logs are NOT persisted to the file system. The frontend has no direct mechanism to write to log files. However, the Rust backend has a comprehensive file logging system.

**Backend Logging Infrastructure (Rust):**

The Rust backend (`/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/logging.rs`) uses the `tracing` crate with file appenders. The logging system:

1. **Log File Location**: Logs are stored at platform-specific paths managed by `get_logs_dir()` function:
   - macOS: `~/Library/Application Support/FMMLoader26/logs/`
   - Windows: `%APPDATA%/FMMLoader26/logs/`
   - Linux: `~/.local/share/FMMLoader26/logs/`

2. **Log File Format**: Files are named `fmmloader.YYYY-MM-DD` with daily rotation (using `tracing_appender::rolling::daily`)

3. **Log Format**: Each line includes timestamp, log level, module name, and message:
   ```
   [2m2025-11-11T23:09:37.692470Z[0m [32m INFO[0m [2mfmmloader26[0m[2m:[0m Application version: 1.0.1
   ```

4. **Initialization Logging**: On startup, the backend logs system information including version, OS, architecture, and the updater endpoint URL (see `main.rs` lines 405-407 and `logging.rs` lines 41-53).

5. **Log Retention**: Old log files are automatically cleaned up, keeping only the last 10 files (see `cleanup_old_logs` function).

The backend currently logs the app version and updater endpoint at startup but does NOT log any update check activity, download progress, or installation events because all update operations happen on the frontend using the Tauri plugin.

**Version Information Access:**

The application version is defined in two places and must be kept in sync:
1. `/Users/jstn/Documents/GitHub/FMMLoader-26/package.json` - version field (line 3: currently "1.0.2")
2. `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/tauri.conf.json` - version field (line 4: currently "1.0.2")
3. `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/Cargo.toml` - version field (line 3: currently "1.0.2")

The backend provides a Tauri command `get_app_version()` (in `main.rs` lines 33-36) that returns the version from the Rust build using `env!("CARGO_PKG_VERSION")`. This is called by the frontend on initialization and stored in the `appVersion` state in `App.tsx` (line 84).

**How latest.json Works:**

The `latest.json` file is part of Tauri's updater artifact system. When `createUpdaterArtifacts: true` is set in `tauri.conf.json` (line 39), the build process automatically generates this JSON file containing:
- `version`: The new version string
- `date`: ISO timestamp of the release
- `body`: Release notes/changelog
- `platforms`: Platform-specific download URLs and signatures

The Tauri plugin-updater automatically parses this JSON and provides it through the `Update` object's properties. The `rawJson` property contains the complete unprocessed JSON for any additional fields.

**UI Display:**

The `UpdateBanner` component (`/Users/jstn/Documents/GitHub/FMMLoader-26/src/components/UpdateBanner.tsx`) displays when an update is available. It shows:
- Latest version number from `status.latestVersion`
- Download progress during download
- Installation status
- Error messages if any

The banner is positioned at the top of the app (line 562 in `App.tsx`) and only renders when `status.available` is true.

### What Needs to Change for Better Logging

Since the update checking happens entirely in the frontend TypeScript code, but file logging infrastructure only exists in the Rust backend, we need to bridge this gap. The implementation needs to:

**Frontend Enhancements:**

1. **Better Console Logging**: The current `addLog` function should include both current and latest version information prominently when logging update events. Currently it shows these on line 64, but the initial check message (line 50) doesn't show the current version.

2. **Structured Update Information**: When an update is found, log all available information from the `Update` object including any useful data from `rawJson` that might help with debugging (like platform information, download URLs, etc.).

3. **Send Logs to Backend**: Create a new Tauri command to send update-related log entries to the backend for file persistence. Since the frontend can't write files directly, it must invoke a Rust command.

**Backend Implementation:**

1. **New Tauri Command**: Add a new command like `log_update_event(message: String)` or `log_update_check(current_version: String, latest_version: String, status: String, details: String)` that receives update information from the frontend and writes it to the log file using the existing `tracing` infrastructure.

2. **Structured Update Logging**: Use `tracing::info!` to log update events with clear, searchable prefixes like `[UPDATE_CHECK]`, `[UPDATE_DOWNLOAD]`, `[UPDATE_INSTALL]` to make them easy to find in log files.

3. **Version Context**: Always include both current and latest version in log messages for clarity.

**Integration Points:**

The `useUpdater` hook's `addLog` function (line 30-38) is the central point where all update logging flows through. This function should be enhanced to:
1. Continue writing to console and in-memory logs (for UI)
2. Additionally invoke a Tauri command to persist critical update events to file
3. Include version information in every relevant log message

The `checkForUpdates` function (lines 40-87) should log:
- Current app version at the start of every check
- The full URL being checked (already does this at line 50)
- All fields from the `Update` object if found
- Clear indication of no update vs error vs update available

The `downloadAndInstall` function (lines 89-148) should log:
- Version being downloaded
- Download progress milestones (not every chunk, but maybe every 25% to avoid log spam)
- Installation success/failure
- Any relevant information from the `rawJson` that could help debugging

### Technical Reference Details

#### Key Files and Their Roles

**Frontend (TypeScript):**
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/hooks/useUpdater.ts` - Update checking logic, needs modification
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/components/UpdateBanner.tsx` - UI display (no changes needed)
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/hooks/useTauri.ts` - Tauri command wrappers, add new logging command wrapper
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src/App.tsx` - Uses UpdateBanner (no changes needed)

**Backend (Rust):**
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/main.rs` - Add new Tauri command for update logging
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/src/logging.rs` - Existing logging infrastructure (no changes needed)
- `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/tauri.conf.json` - Updater configuration (no changes needed)

#### Update Object Interface (from @tauri-apps/plugin-updater v2.9.0)

```typescript
interface UpdateMetadata {
  rid: number;
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;
  rawJson: Record<string, unknown>;
}

class Update extends Resource {
  available: boolean;  // deprecated, always true
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;
  rawJson: Record<string, unknown>;

  download(onEvent?: (progress: DownloadEvent) => void): Promise<void>;
  install(): Promise<void>;
  downloadAndInstall(onEvent?: (progress: DownloadEvent) => void): Promise<void>;
}

type DownloadEvent =
  | { event: 'Started'; data: { contentLength?: number } }
  | { event: 'Progress'; data: { chunkLength: number } }
  | { event: 'Finished' };

function check(options?: CheckOptions): Promise<Update | null>;
```

#### Current UpdateStatus Interface (useUpdater.ts lines 5-15)

```typescript
export interface UpdateStatus {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  installing: boolean;
  error: string | null;
  currentVersion: string | null;
  latestVersion: string | null;
  downloadProgress: number;
  logs: string[];  // In-memory only, not persisted
}
```

#### Logging Function Signatures

**Frontend (existing):**
```typescript
const addLog = useCallback((message: string) => {
  const timestamp = new Date().toLocaleTimeString();
  const logMessage = `[${timestamp}] ${message}`;
  console.log(`[Updater] ${logMessage}`);
  setStatus(prev => ({ ...prev, logs: [...prev.logs, logMessage] }));
}, []);
```

**Backend (existing patterns from logging.rs and main.rs):**
```rust
// Basic logging (already used throughout)
tracing::info!("message");
tracing::error!("error: {}", e);
tracing::debug!("debug info: {:?}", data);

// Current startup logging includes version
tracing::info!("Application version: {}", app_version);
tracing::info!("Updater endpoint: https://...");
```

#### New Tauri Command Pattern Needed

**In main.rs** (add new command):
```rust
#[tauri::command]
fn log_update_event(
    event_type: String,
    current_version: String,
    latest_version: Option<String>,
    message: String,
    details: Option<String>
) -> Result<(), String> {
    tracing::info!(
        "[UPDATE_{}] Current: {} | Latest: {} | {} | Details: {}",
        event_type,
        current_version,
        latest_version.unwrap_or_else(|| "N/A".to_string()),
        message,
        details.unwrap_or_else(|| "None".to_string())
    );
    Ok(())
}
```

**In useTauri.ts** (add to tauriCommands object):
```typescript
logUpdateEvent: (
  eventType: string,
  currentVersion: string,
  latestVersion: string | null,
  message: string,
  details?: string
) => safeInvoke<void>("log_update_event", {
  eventType,
  currentVersion,
  latestVersion,
  message,
  details,
}),
```

**Register command in main.rs** (add to invoke_handler at line 416):
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    log_update_event,
])
```

#### Configuration Locations

**Update endpoint configuration:**
- File: `/Users/jstn/Documents/GitHub/FMMLoader-26/src-tauri/tauri.conf.json`
- Lines: 63-67
- Current: `https://github.com/justinlevinedotme/FMMLoader-26/releases/latest/download/latest.json`

**Version definitions (must stay in sync):**
- `package.json` line 3
- `src-tauri/Cargo.toml` line 3
- `src-tauri/tauri.conf.json` line 4
- All currently: "1.0.2"

**Log file storage:**
- macOS: `~/Library/Application Support/FMMLoader26/logs/fmmloader.YYYY-MM-DD`
- Windows: `%APPDATA%/FMMLoader26/logs/fmmloader.YYYY-MM-DD`
- Linux: `~/.local/share/FMMLoader26/logs/fmmloader.YYYY-MM-DD`

#### Implementation Locations

**Where to add enhanced logging in useUpdater.ts:**

1. **Line 50** - After logging endpoint URL, add current version
2. **Lines 64-66** - Already logs version info well, but should also call backend logging
3. **Line 91** - Download start should log versions
4. **Line 102** - Include current/latest versions with download message
5. **Line 110** - Download started event could include versions
6. **Line 128** - Installation success should log versions
7. **Line 79** - Error logging should include current version for context

**Pattern to follow:** For each significant event, call both:
1. `addLog(message)` - for console and UI display
2. `tauriCommands.logUpdateEvent(...)` - for file persistence

This ensures logs appear in three places: browser console (for development), UI logs tab (for user visibility), and persistent log files (for troubleshooting).

## User Notes
<!-- Any specific notes or requirements from the developer -->

## Work Log
<!-- Updated as work progresses -->
- [YYYY-MM-DD] Started task, initial research
