---
name: h-fix-multiple-toast-notifications-on-import
branch: fix/multiple-toast-notifications-on-import
status: pending
created: 2025-11-11
---

# Fix Multiple Toast Notifications on Mod Import

## Problem/Goal
When importing mods, multiple duplicate toast notifications are appearing instead of a single notification. This creates a poor user experience and indicates that the toast triggering logic is being called multiple times or from multiple locations during the import process.

## Success Criteria
- [ ] Only one toast notification appears per mod import operation (single or batch)
- [ ] Toast notification still displays all relevant information (success/error messages, mod name)
- [ ] No duplicate toasts appear during edge cases (multiple rapid imports, concurrent operations)
- [ ] Error notifications continue to work correctly without duplication

## Context Manifest

### How Toast Notifications Work in This Application

The application uses **Sonner** (v2.0.7) as the toast notification library. Sonner is a modern, accessible toast component for React that provides a clean API and built-in styling. The Toaster component is configured in `/src/components/ui/sonner.tsx` and rendered once in the main App component at line 1205 of `/src/App.tsx`.

**Toast Triggering Pattern:**
Throughout the application, toasts are triggered using the `toast` function imported from 'sonner' at line 5 of App.tsx. The toast API provides methods like:
- `toast.success()` - Green success notifications
- `toast.error()` - Red error notifications
- `toast.warning()` - Yellow warning notifications
- `toast.info()` - Info notifications
- `toast.loading()` - Loading state with spinner

**Key Design Detail:** Sonner automatically handles toast deduplication when you provide an `id` parameter. For example, in the `applyMods()` function (lines 287-291), the code uses `{ id: "apply-mods" }` to ensure the loading toast can be updated to a success/error toast without creating duplicates.

### The Import Flow and Where Toasts Are Triggered

**Entry Points for Mod Import:**

1. **Manual Import Button** (line 604-609 in App.tsx):
   - User clicks "Import" button in the UI
   - Triggers `handleImportClick()` function (lines 325-344)
   - Opens file dialog using Tauri's dialog plugin
   - If user selects a file, calls `handleImport(selected)`

2. **Drag-and-Drop** - **THIS IS THE PROBLEM SOURCE** (lines 493-518 in App.tsx):
   - Set up in `useEffect` hook on component mount (lines 482-546)
   - Listens to TWO different Tauri events that BOTH trigger imports:
     - **`tauri://file-drop`** (lines 493-502): Legacy Tauri v1 event
     - **`tauri://drag-drop`** (lines 508-518): Tauri v2 event
   - Both event handlers call `handleImport()` when files are dropped
   - **ROOT CAUSE:** When a file is dropped, Tauri v2 fires BOTH events for backward compatibility
   - This means `handleImport()` gets called TWICE for a single drag-drop action
   - Each call to `handleImport()` triggers its own toast notification

**The handleImport() Function Flow** (lines 346-366):

When `handleImport(sourcePath)` is called, it:

1. Logs the import attempt via `addLog()`
2. Calls the Rust backend via `tauriCommands.importMod(sourcePath)` (line 349)
3. **On Success:**
   - Logs success message
   - **Triggers toast:** `toast.success(\`Successfully imported: ${result}\`)` (line 351)
   - Calls `loadMods()` to refresh the mod list (line 352)
4. **On Error:**
   - If error is "NEEDS_METADATA": Opens metadata dialog AND triggers `toast.info()` (line 360)
   - Otherwise: Logs error AND triggers `toast.error()` (line 363)

**The Rust Backend Import Flow** (`src-tauri/src/main.rs`, lines 146-258):

The `import_mod` command:
1. Resolves the source path (handles ZIP extraction, single files, directories)
2. Checks for manifest.json
3. If no manifest and no metadata provided, returns `Err("NEEDS_METADATA")`
4. If metadata provided, generates manifest
5. Copies mod files to the mods directory
6. Returns the mod name on success

**Important:** The backend does NOT emit any events or trigger any frontend callbacks. It's a synchronous command that returns a Result. This means the duplication happens entirely on the frontend.

### Why Multiple Toasts Appear

**Primary Issue: Double Event Listeners**

In `App.tsx` lines 493-518, the application registers listeners for both:
- `tauri://file-drop` - The old Tauri v1 drag-drop event
- `tauri://drag-drop` - The new Tauri v2 drag-drop event

When a user drags and drops a file in Tauri v2, the framework fires BOTH events for backward compatibility. This causes `handleImport()` to be called twice in rapid succession with the same file path.

**Flow of a Drag-Drop Import:**
1. User drags file over application window → `tauri://drag-over` fires → `setIsDragging(true)`
2. User drops file → **BOTH events fire:**
   - `tauri://file-drop` handler executes → calls `handleImport(files[0])`
   - `tauri://drag-drop` handler executes → calls `handleImport(paths[0])`
3. First `handleImport()` call:
   - Imports mod successfully
   - Shows toast: "Successfully imported: ModName"
4. Second `handleImport()` call (milliseconds later):
   - Tries to import same mod
   - Backend returns error: "Mod 'ModName' already exists" (line 249 of main.rs)
   - Shows toast: "Import failed: Mod 'ModName' already exists"

**Result:** User sees two toasts - one success, one error - for a single drag-drop action.

**Secondary Issue: React.StrictMode**

The application uses React.StrictMode in `/src/main.tsx` (lines 6-10). In development mode, StrictMode intentionally double-invokes effects to help identify side effects. However, the event listener cleanup is properly implemented (lines 533-538), so this should not cause issues in production builds. The StrictMode double-invocation only affects the initial mount, not subsequent drag-drop events.

**Third Potential Issue: useEffect Dependencies**

The main `useEffect` hook (lines 482-546) has an empty dependency array `[]`, which is correct for setup code. However, the `handleImport` function used in the event listeners is defined in the component body (lines 346-366), which means it captures the current closure of state and functions. If the component were to re-render and re-register event listeners without proper cleanup, this could lead to duplicate listeners. Currently, the cleanup function IS returned (lines 533-538), but there's a subtle bug: the cleanup function is only returned inside the try block, so if initialization fails, the listeners are never cleaned up.

### State Management and Re-renders

**Config State and loadMods** (lines 548-553):

There's a second useEffect that calls `loadMods()` whenever `config` changes:
```typescript
useEffect(() => {
  if (config) {
    void loadMods();
  }
}, [config]);
```

After a successful import, `handleImport()` calls `loadMods()` directly (line 352). This doesn't cause duplicate toasts because the toast is triggered BEFORE loadMods, not as a side effect of loading mods. However, this pattern means the mod list gets loaded twice after each import - once from handleImport, and potentially again if the config changes during the process.

**Toaster Component Rendering:**

The `<Toaster />` component is rendered exactly once at line 1205, just before the closing `</TooltipProvider>`. This is the correct pattern. The Toaster component itself manages its own internal state for displaying multiple toasts and should not cause duplication issues.

### Technical Details for Implementation

#### File Locations

**Frontend Files:**
- `/src/App.tsx` - Main application component containing import logic and event listeners
  - Lines 346-366: `handleImport()` function
  - Lines 368-381: `handleMetadataSubmit()` function
  - Lines 493-518: Drag-drop event listener setup (THE BUG)
  - Line 1205: Toaster component rendering

- `/src/components/ui/sonner.tsx` - Toaster component configuration
  - Uses Sonner v2.0.7
  - Configured with dark theme and custom color classes

- `/src/main.tsx` - Application entry point
  - Lines 6-10: React.StrictMode wrapper (development behavior)

- `/src/hooks/useTauri.ts` - Tauri command wrapper
  - Lines 111-122: `importMod` command definition

**Backend Files:**
- `/src-tauri/src/main.rs` - Main Tauri command handlers
  - Lines 146-258: `import_mod` command implementation
  - Line 249: Error message for duplicate mods

- `/src-tauri/src/import.rs` - Import helper functions
  - ZIP extraction, manifest detection, mod type detection

#### Toast API Usage Pattern

All toast calls in the codebase:
- Line 280: `toast.warning()` - No ID (one-off warning)
- Line 287: `toast.loading()` - WITH ID "apply-mods" (good pattern)
- Line 291: `toast.success()` - WITH ID "apply-mods" (updates loading toast)
- Line 294: `toast.error()` - WITH ID "apply-mods" (updates loading toast)
- Line 310: `toast.success()` - No ID (mod removed)
- Line 321: `toast.error()` - No ID (remove failed)
- **Line 351: `toast.success()` - No ID (IMPORT SUCCESS - DUPLICATES HERE)**
- **Line 360: `toast.info()` - No ID (METADATA NEEDED - POTENTIAL DUPLICATE)**
- **Line 363: `toast.error()` - No ID (IMPORT FAILED - SHOWS AFTER SUCCESS)**
- Line 451: `toast.success()` - No ID (name fix installed)
- Line 456: `toast.error()` - No ID (name fix install failed)
- Line 468: `toast.success()` - No ID (name fix uninstalled)
- Line 473: `toast.error()` - No ID (name fix uninstall failed)

**Pattern Analysis:** The `applyMods()` function demonstrates the correct pattern for preventing duplicates - use the same `id` for loading, success, and error states so Sonner updates the same toast instead of creating new ones.

### Solution Approaches

**Option 1: Remove Legacy Event Listener (RECOMMENDED)**
Since this is a Tauri v2 application, remove the `tauri://file-drop` listener and only keep `tauri://drag-drop`. This is the cleanest solution.

**Option 2: Add Toast ID for Import Operations**
Add a unique ID to import toast calls so duplicate calls update the same toast instead of creating new ones. However, this doesn't solve the underlying issue of double imports - the backend would still be called twice.

**Option 3: Debounce handleImport Calls**
Add debouncing to prevent rapid duplicate calls to handleImport. This is a band-aid solution that adds complexity.

**Option 4: Track In-Progress Imports**
Add state to track whether an import is currently in progress, and ignore duplicate calls. This prevents double imports but adds state management complexity.

**RECOMMENDED SOLUTION:** Option 1 - Remove the legacy `tauri://file-drop` event listener since the application is built for Tauri v2. This eliminates the root cause entirely. Additionally, consider adding toast IDs for import operations (Option 2) as a defensive measure against any other potential duplication sources.

## User Notes
<!-- Any specific notes or requirements from the developer -->

## Work Log
<!-- Updated as work progresses -->
- [YYYY-MM-DD] Started task, initial research
