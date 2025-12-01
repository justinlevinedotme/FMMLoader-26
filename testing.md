## Testing & Debug Utilities

This project includes quick utilities to verify mod path resolution and open UI dialogs without running full flows.

### Prerequisites
- Node.js 18+
- Rust toolchain (for the Tauri backend/CLI)
- Install deps: `pnpm install`

### Path Preview / Routing Checks
- **Script**: `pnpm run test:paths`
  - Uses `FMML_TEST_APPDATA=.tmp/fmml-test-appdata` to avoid touching real user directories.
  - Prints the base target for each mod type and resolved paths for sample subpaths.
- **Custom file overrides**: `pnpm run test:paths -- --file graphics/faces/face.png --file tactics/433.fmf`
  - Each `--file` is treated as a `target_subpath`; the script shows where it would land per mod type.
<!-- fmml_path_debug helper binary removed -->
  - No files are written; it only computes paths.

### UI Debug Playground
- **Normal dev app**: `pnpm run tauri dev`
- **Force debug UI on**: `pnpm run tauri:debugui` (sets `VITE_ENABLE_DEBUG_UI=true`)
- The “UI Debug / Playground” card (dev-only) provides toggles to open Metadata, Conflicts, Restore Points, Settings, Mod Details, and Graphics Confirm dialogs without the usual flows.

### Frontend Dialog Tests
- **Run**: `pnpm run test:ui`
- Stack: Vitest + Testing Library + jsdom (configured in `vite.config.ts`, setup in `src/setupTests.ts`).
- Coverage: ModMetadataDialog basic submit path, ConflictsDialog data render, GraphicsPackConfirmDialog path display.

### Backend Tests
- **Rust**: `cd src-tauri && cargo test`
  - Includes path override, path preview helper, and `get_target_for_type` resolution tests.
