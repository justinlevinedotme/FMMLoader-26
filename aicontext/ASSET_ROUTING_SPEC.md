# ASSET_ROUTING_SPEC.md  
_FMMLoader 26 — Unified Asset Detection & Routing Specification_

This specification defines how FMMLoader must detect, classify, extract, validate, and install all Football Manager 26 asset types into the correct folders inside the **FM User Directory**.

It exists so all AI agents (Codex, Claude Code, Copilot, local LLMs, etc.) behave consistently and do not hardcode paths or re-implement routing logic.

---

## 1. FM User Directory Locations

FMMLoader must treat the following as the canonical **FM26 User Directory** locations:

### macOS
```text
~/Library/Application Support/Sports Interactive/Football Manager 26/
````

### Windows

```text
%USERPROFILE%/Documents/Sports Interactive/Football Manager 26/
```

### Linux

```text
~/.local/share/Sports Interactive/Football Manager 26/
```

Within the FM User Directory, FMMLoader must ensure these folders exist (creating them if needed):

```text
graphics/
editor data/
```

> NOTE: These paths are examples for documentation and user help only. Agents MUST NOT hardcode them in code; path resolution MUST go through the existing Rust functions described in Section 8.

---

## 2. Asset Types & Routing Rules

FMMLoader classifies assets by **extension**, **directory structure**, and **content signatures**, then routes them into the appropriate child directory of the FM User Directory.

---

### 2.1 Graphics Assets

**Destination:**

```text
<FM User Directory>/graphics/
```

**Accepted graphics file types:**

* Images: `.png`, `.jpg`, `.jpeg`, `.tga`, `.bmp`
* Graphics config: `.xml` (ONLY when used as a graphics config, e.g. `config.xml` for faces/logos)

**Common subfolders:**

```text
graphics/faces/
graphics/logos/
graphics/kits/
graphics/3d kits/
graphics/backgrounds/
graphics/screens/
```

**Routing rules:**

* Preserve the directory hierarchy exactly as provided by the mod.
* Do NOT flatten folder structures (e.g., don’t collapse `faces/players/…` into one directory).
* Do NOT rename files.
* Do NOT overwrite existing files unless the user explicitly approves (or the UI flow clearly indicates replacement).

---

### 2.2 Editor Data Assets

**Destination:**

```text
<FM User Directory>/editor data/
```

**Accepted editor data file types:**

* `.fmf` (Football Manager editor data files)
* `.xml` (ONLY if specifically intended as editor data, not graphics)

**Typical content includes:**

* Real name fixes
* League structure or competition rebuilds
* Transfer rules and windows
* Player/club attribute changes
* Custom nations or databases

**Routing rules:**

* Do NOT modify, unpack, or “peek inside” `.fmf` content.
* Do NOT place `.fmf` files in `graphics/`.
* Do NOT place pure graphics assets into `editor data/`.

---

## 3. Archive Handling

Many mods are shipped as compressed archives and may combine multiple asset types. FMMLoader must always extract and then classify.

---

### 3.1 Supported Archive Types

* `.zip`
* `.rar`
* `.7z`
* `.tar.gz`
* `.tar.xz`

**Required behavior:**

1. Extract the archive to a temporary location.
2. Classify extracted contents as graphics and/or editor data.
3. Route each file or folder into its correct destination (see Section 2).

---

### 3.2 Multi-Part Archives

Example:

```text
pack.part01.rar
pack.part02.rar
pack.part03.rar
```

FMMLoader must:

* Only extract `pack.part01.rar`.
* Let the decompression tool automatically read subsequent parts.
* Handle missing or corrupted segments gracefully (clear error to the user).

---

### 3.3 Mixed Packs (Graphics + Editor Data)

A single pack may contain both types, e.g.:

```text
/MyFMAddonPack/
    /faces/
    /logos/
    database.fmf
    name_fixes.fmf
```

**Routing behavior:**

* All image assets → `graphics/`
* `.fmf` files → `editor data/`
* Graphics `.xml` → `graphics/`
* Editor `.xml` → `editor data/`

**Recommended user messaging:**

* “Graphics and Editor Data detected. Routing assets to the appropriate FM26 folders.”
* “Editor Data (.fmf) found within a graphics pack—moving to `editor data/`.”
* “Graphics directories found in an editor data-only pack—moving to `graphics/`.”

---

## 4. Misplaced Content Detection & Auto-Correction

FMMLoader must detect obvious misplacements and fix them automatically, while informing the user.

### 4.1 Examples of Invalid Placements

* `.fmf` inside `graphics/`
* `faces/`, `logos/`, `kits/` inside `editor data/`
* Compressed archives (`.zip`, `.rar`, `.7z`) placed directly inside `graphics/` or `editor data/`
* Graphics config XML placed in `editor data/`

### 4.2 Correction Rules

* Relocate files to the correct directory based on type.
* Never silently delete user files.
* Avoid overwriting existing files unless explicitly allowed.
* Provide clear feedback about what was corrected.

**Example warning messages:**

* “⚠️ Editor Data detected under `graphics/` — relocating files to `editor data/`.”
* “⚠️ Graphics files detected under `editor data/` — relocating to `graphics/`.”
* “⚠️ Archive file found in target directory — extracting and routing its contents.”

---

## 5. Post-Install Behavior (FM26 Updated)

Football Manager 26 **no longer** uses manual skin-refresh buttons or cache toggles. Graphics handling is restart-based, editor data remains new-save-based.

### 5.1 Graphics (Faces, Logos, Kits, Backgrounds)

To make newly installed graphics visible:

* FM26 must be **fully restarted**.
* A simple return to the main menu is not sufficient; the process must quit and relaunch the game.
* No skin reload button exists in FM26.

**Required user action:**

> “Quit Football Manager 26 completely and restart the game to load your new graphics.”

---

### 5.2 Editor Data (.fmf)

Editor Data behavior remains consistent with prior FM versions:

* Editor Data files are only read when starting a **new save**.
* Existing saves do not update with new `.fmf` content.

**Required user action:**

> “Start a new career/save to apply editor data changes.”

---

## 6. Summary Table

| Asset Type  | Extensions                              | Destination        | Requires Restart? | Requires New Save? |
| ----------- | --------------------------------------- | ------------------ | ----------------- | ------------------ |
| Graphics    | `.png`, `.jpg`, `.tga`, graphics `.xml` | `graphics/`        | **Yes**           | No                 |
| Editor Data | `.fmf`, editor `.xml`                   | `editor data/`     | No                | **Yes**            |
| Mixed Packs | Mixed                                   | Split by type      | Mixed             | Mixed              |
| Archives    | `.zip`, `.rar`, `.7z`, `.tar.*`         | Extract → classify | N/A               | N/A                |

---

## 7. General Agent Behavior Requirements

### 7.1 Agents MUST

* Correctly detect graphics vs editor data.
* Split mixed packs into their appropriate destinations.
* Preserve directory structures.
* Use FMMLoader’s existing path resolution functions (see Section 8).
* Create missing directories (`graphics/`, `editor data/`) when needed.
* Provide clear summaries of what was installed and where.

### 7.2 Agents SHOULD

* Warn users about suspicious or malformed packs.
* Handle corrupt archives in a user-friendly way.
* Keep logs or structured output for UI display.

### 7.3 Agents MUST NOT

* Hardcode FM user directory paths in code.
* Place `.fmf` files into `graphics/`.
* Place graphics assets into `editor data/`.
* Modify, extract, or partially rewrite `.fmf` content.
* Flatten graphics directories.
* Implement an alternate routing system that bypasses existing functions.

---

## 8. Path Resolution & Code-Change Policy (CRITICAL)

To remain robust across FM26 updates and user-customized setups, **all path logic must go through existing Rust functions**. The goal is:

1. **Review and reuse existing code** whenever possible.
2. Only if truly necessary, propose **minimal and localized changes**.

---

### 8.1 Authoritative Functions

FMMLoader already defines canonical path and routing behavior in:

* `game_detection::get_fm_user_dir(user_dir)`
* `mod_manager::get_target_for_type(mod_type, game_target, user_dir)`
* `mod_manager::install_mod(...)`
* `mod_manager::uninstall_mod(...)`

All path logic and asset routing MUST be expressed via these functions where possible.

---

### 8.2 First Step: Review Existing Code

When an agent needs to work with install paths or add a new mod type, it must:

1. **Open and review** `src/mod_manager.rs`.

2. Inspect how `get_target_for_type` currently works:

   ```rust
   pub fn get_target_for_type(
       mod_type: &str,
       game_target: &Path,
       user_dir: Option<&str>,
   ) -> PathBuf {
       let user_path = get_fm_user_dir(user_dir);

       match mod_type {
           "ui" | "bundle" | "skins" => game_target.to_path_buf(),
           "tactics" => user_path.join("tactics"),
           "graphics" => user_path.join("graphics"),
           "editor-data" => user_path.join("editor data"),
           _ => game_target.to_path_buf(),
       }
   }
   ```

3. Attempt to express any desired routing by:

   * Using `get_target_for_type(...)` directly, or
   * Selecting an appropriate existing `mod_type` and `target_subpath` in the manifest so that routing “just works” via the current match arms.

If the desired behavior can be achieved using these existing functions, the agent MUST NOT propose code changes.

---

### 8.3 Only If Necessary: Propose Minimal Changes

If, **after reviewing `mod_manager.rs`**, the agent determines that:

* No existing `mod_type` can produce the correct destination, **and**
* The routing cannot be expressed solely via `get_target_for_type`’s current logic,

then it may propose **minimal, targeted edits**, such as:

* Adding a single new match arm to `get_target_for_type`, e.g.:

  ```rust
  "data" => user_path.join("data"),
  ```

* Adding or updating a **small unit test** in the `#[cfg(test)]` block to cover the new case.

Constraints for changes:

* Do NOT restructure `mod_manager.rs`.
* Do NOT introduce new, parallel routing systems.
* Do NOT bypass `get_fm_user_dir(user_dir)` when building user paths.
* Keep diffs as small and localized as possible.

If more invasive changes seem required, the agent should clearly state this in its explanation rather than implementing them by default.

---

### 8.4 Path Resolution Rules

Agents MUST:

* Use `get_fm_user_dir(user_dir)` to resolve the FM User Directory root.
* Use `get_target_for_type(mod_type, game_target, user_dir)` to determine the final target directory for each mod type.
* Treat the hardcoded example paths in Section 1 as **documentation only**, not as implementation details.

Agents MUST NOT:

* Hardcode strings like:

  * `~/Library/Application Support/Sports Interactive/Football Manager 26/`
  * `%USERPROFILE%/Documents/Sports Interactive/Football Manager 26/`
  * `~/.local/share/Sports Interactive/Football Manager 26/`
* Implement their own routing tables outside of `get_target_for_type`.

---

### 8.5 When Unsure

If an agent is unsure how to route a given asset type, it must:

1. Re-read `src/mod_manager.rs` and the tests at the bottom of that file.
2. Try to map the desired behavior onto an existing `mod_type` and `target_subpath`.
3. Only then, and only if absolutely necessary, propose a minimal new match arm + test.

The default mindset should always be:

> “There is probably already a way to route this via `get_target_for_type` — reuse that first.”

---

This file is the authoritative specification for how FMMLoader 26 agents must classify and route assets. Any new agent behavior relating to file paths or install locations must be consistent with this specification and the existing Rust implementation.