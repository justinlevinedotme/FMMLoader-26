---
id: T-0003
title: "Add path preview tooling and UI debug surface"
status: done
priority: high
owner: ""
created_at: 2025-11-18
updated_at: 2025-11-18
tags: ["backend", "frontend", "testing"]
---

## 0. User story / problem

Verifying mod install paths and dialog rendering currently requires importing real mods and clicking through normal flows, risking regressions when path logic changes. We need test-friendly ways to compute paths without touching real user directories and a dev-only UI surface to open dialogs predictably.

---

## 1. Context gathered by AI(s)

### 1.1 Relevant files / dirs

- `src-tauri/src/config.rs` — app data paths, mods/backup/restore dir helpers.
- `src-tauri/src/game_detection.rs` — FM user dir detection and custom path handling.
- `src-tauri/src/mod_manager.rs` — target path resolution for mod types.
- `src-tauri/src/` (new bin/CLI likely alongside existing main) — place for a path-debug CLI.
- `src/` React app: `App.tsx`, dialog components (`ModMetadataDialog`, `ConflictsDialog`, `RestorePointsDialog`, `GraphicsPackConfirmDialog`, settings UI, mod details panel).
- `package.json` / npm scripts — add a test/preview command.

### 1.2 Summary of current behavior

- Path helpers derive app data and install targets internally; no test-only override exists for app data base.
- get_target_for_type tests focus on suffixes rather than full resolved paths.
- UI dialogs open via normal user flows; no dedicated debug screen exists to toggle them directly.

### 1.3 Risks / constraints / assumptions

- Must ensure any test-only app data override is gated (env var) and does not affect production defaults.
- Dev-only UI debug surface must stay hidden in production builds.
- Need to confirm how config is loaded by CLI/Tauri commands to avoid divergence.

### 1.4 Open questions for the human

- Confirm desired env var name for test app data base (proposal: `FMML_TEST_APPDATA`).
- Which dialogs are mandatory on the debug screen beyond the listed set?
- Should the CLI read config from the same path as the app or accept explicit config file override?

---

## 2. Proposed success criteria (AI draft)

1) Test-only app data override: `get_app_data_dir` checks `FMML_TEST_APPDATA` (or agreed name) before OS defaults; covered by Rust unit tests for mods/backup/restore dir helpers using overridden base.  
2) Path resolution tests: `game_detection.rs` unit tests lock in `get_fm_user_dir` behaviors (custom path variants) and `mod_manager.rs` tests assert full resolved targets (including user dir and subpaths) for mod types.  
3) Path preview helper: pure function that, given mod_type, game_target, user_dir, returns base install dir and mapping of `FileEntry.target_subpath` -> resolved path without FS writes; exposed via new Tauri command `preview_mod_install`.  
4) CLI: dev/test `fmml-path-debug` in `src-tauri` that reads config (with `--target-path` / `--user-dir` overrides) and prints base install directories for supplied `--mod-type` values.  
5) NPM script: `npm run test:paths` (or similar) runs the path preview/CLI checks to validate pathing pre-release.  
6) UI debug surface: dev-only screen (e.g., `import.meta.env.DEV`) with toggles/buttons to open major dialogs (metadata, conflicts, restore points, graphics pack confirm, settings, mod details) and display which are active.  
7) (Optional) Component tests: Vitest + Testing Library ensure each core dialog renders when `open={true}` and key actions/required fields behave (metadata dialog submit requirements).  

Reply APPROVE to accept.

---

## 3. Approved success criteria (human-edited)

1) Test-only app data override: `get_app_data_dir` checks `FMML_TEST_APPDATA` before OS defaults; covered by Rust unit tests for mods/backup/restore dir helpers using overridden base.  
2) Path resolution tests: `game_detection.rs` unit tests lock in `get_fm_user_dir` behaviors (custom path variants) and `mod_manager.rs` tests assert full resolved targets (including user dir and subpaths) for mod types.  
3) Path preview helper: pure function that, given mod_type, game_target, user_dir, returns base install dir and mapping of `FileEntry.target_subpath` -> resolved path without FS writes; exposed via new Tauri command `preview_mod_install`.  
4) CLI: dev/test `fmml-path-debug` in `src-tauri` that reads config (with `--target-path` / `--user-dir` overrides) and prints base install directories for supplied `--mod-type` values.  
5) NPM script: `npm run test:paths` (or similar) runs the path preview/CLI checks to validate pathing pre-release.  
6) UI debug surface: dev-only screen (e.g., `import.meta.env.DEV`) with toggles/buttons to open major dialogs (metadata, conflicts, restore points, graphics pack confirm, settings, mod details) and display which are active.  
7) (Optional) Component tests: Vitest + Testing Library ensure each core dialog renders when `open={true}` and key actions/required fields behave (metadata dialog submit requirements).  

---

## 4. Implementation log / notes

- 2025-11-18 – intake-planner: created task draft from user request.
- 2025-11-18 – implementation: added test-only app data override, path preview helper/command/CLI, expanded path resolution tests, dev-only UI debug surface, and ran `cargo fmt`/`cargo test`.
- 2025-11-18 – implementation: adjusted `npm run test:paths` to point at `src-tauri/Cargo.toml` and correct CLI bin name (`fmml_path_debug`); verified the script prints expected base paths.
- 2025-11-18 – implementation: set `default-run = "fmmloader26"` in `src-tauri/Cargo.toml` so `npm run tauri dev` targets the main app binary when multiple bins exist.
- 2025-11-18 – implementation: enhanced `fmml_path_debug` to preview default/sample file mappings (or `--file` overrides), so `npm run test:paths` now prints resolved file targets per mod type.
- 2025-11-18 – implementation: added `npm run tauri:debugui` / `VITE_ENABLE_DEBUG_UI=true` flag to show the UI debug playground even outside default dev mode.
- 2025-11-18 – implementation: added dialog component tests with Vitest/Testing Library for metadata, conflicts, and graphics confirm dialogs; new script `npm run test:ui`.

---

## 5. Completion checklist & review

### 5.1 Human review notes

### 5.2 Follow-up tasks (spinoffs)

Completed.
