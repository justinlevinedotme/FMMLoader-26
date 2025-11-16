# TAURI_PLUGIN_STRATEGY.md
_FMMLoader 26 – Tauri v2 Plugin & Tools Strategy_

This document defines how we want AI assistants (Codex, Claude Code, Copilot, etc.) to approach **Tauri 2.x** problems in this repo.

**Core rule:**
> Always try to solve problems using **Tauri core** and **existing plugins** before writing any new custom tools (commands, plugins, or external binaries).

This file is context, not code. It describes the decision-making process we expect before any new Rust APIs or plugins are created.

---

## 1. Goals

1. **Minimize custom Rust surface area.**
2. **Leverage official & well-maintained plugins first.**
3. **Keep security & capabilities sane** (no unnecessary shell/file power).
4. **Keep changes small and incremental**, not framework rewrites.

The result should be: fewer bespoke tools, more reuse of Tauri’s v2 ecosystem.

---

## 2. Default Decision Flow for Any New Feature

When adding a new feature that touches the OS, filesystem, network, or system APIs:

1. **Clarify the requirement**
   - What exactly needs to happen?
   - One-shot task vs long-running background behavior?
   - Only on desktop, or mobile as well?

2. **Check if Tauri core already solves it**
   - Events / `emit` / `listen`
   - Windows / webviews
   - Built-in dialog APIs
   - Frontend-only options (e.g., Web APIs)

3. **Check official Tauri plugins**
   - Official v2 plugins (fs, http, store, sql, log, shell, process, notification, updater, websocket, etc.).
   - If feature fits one of these, **use it instead of writing new commands**.

4. **Check community plugins**
   - v2 “Features & Plugins” list.
   - Awesome Tauri / community plugins (logging, analytics, auth, RPC, BLE, etc.).

5. **Only if none of the above work**
   - Then consider:
     - A small **custom Rust command** in the existing backend, or
     - A **minimal plugin** with very narrow scope.

In other words:

> **Compose first. Reuse second. Extend last.**

---

## 3. Preferred Plugins & Use Cases

When solving a problem, AI should actively try to map it to an existing plugin before inventing new tools.

Some key examples (non-exhaustive):

- **Filesystem / paths / config files**
  - Prefer: `@tauri-apps/plugin-fs` (file operations, baseDir, secure path handling).
  - Use `BaseDirectory` and `path` APIs instead of custom `std::fs` Rust commands where possible.

- **HTTP / remote APIs / webhooks**
  - Prefer: `tauri-plugin-http` (Rust HTTP client exposed to frontend).
  - Avoid writing ad-hoc `reqwest` commands unless there’s a strong reason.

- **Persistent settings / small state**
  - Prefer: `@tauri-apps/plugin-store` for key-value state.
  - Only drop to custom DB or raw `fs` when you really need more structure.

- **SQL / structured data**
  - Prefer: `tauri-plugin-sql` when a local SQL database is appropriate.

- **Shell / spawning processes**
  - Prefer: `@tauri-apps/plugin-shell` if you need to run external commands.
  - Do NOT roll your own process management if shell plugin can do it.

- **Logging**
  - Prefer: `tauri-plugin-log` instead of custom logging plumbing.

- **Update mechanism**
  - Prefer: `tauri-plugin-updater` before inventing custom update flows.

If an existing plugin **does what we need with configuration**, the AI should favor that over designing a custom Rust tool/API.

---

## 4. Capabilities & Security

Tauri v2 provides a **capabilities system** to grant/limit plugin access.

AI should:

- Prefer enabling capabilities **per feature** (per plugin/use case) instead of globally opening everything.
- When suggesting a new plugin:
  - Add **minimal capability files** under `src-tauri/capabilities/`.
  - Enable only the APIs required for that window/webview.

If more capabilities are needed later, expand them **incrementally**, not preemptively.

---

## 5. Required Workflow for AI Before Proposing Custom Tools

Whenever AI is about to propose:

- A new `#[tauri::command]`
- A new plugin
- Additional low-level Rust helpers that touch OS/services

It must:

1. **Review the existing Tauri setup**
   - `src-tauri/src/main.rs` (or equivalent): see which plugins are already in use.
   - `tauri.conf.json` / `tauri.conf.json5`: look at plugins and capabilities.
   - Any plugin usage in frontend code (`@tauri-apps/plugin-*` imports).

2. **Attempt to solve the problem with:**
   - Existing plugins already configured in this app.
   - Official v2 plugins that can be added with a one-line `tauri add <plugin>` and minimal setup.
   - Composition of existing commands + frontend logic.

3. **Only if that fails, propose changes**
   - For **simple tasks**:
     - Suggest a **small new `#[tauri::command]`** in `main.rs` or an existing module.
   - For **complex OS integrations**:
     - Suggest a **minimal plugin**, following Tauri’s plugin development patterns, with:
       - A single crate `tauri-plugin-<name>`
       - A tiny JS API (if needed)
       - Narrow, focused commands

**Important:**  
> AI should always justify *why* an existing plugin cannot solve the problem before proposing custom tools.

---

## 6. Minimal-Change Policy for Plugin/Tool Additions

When adding or suggesting new tooling, the AI must:

- **Prefer configuration over code**
  - First: “Can this be a plugin config / capability change?”
  - Second: “Can I add a known plugin and call its API?”
  - Last: “Do I really need new Rust code?”

- **Keep diffs as small as possible**
  - Don’t restructure `main.rs` or the plugin initialization if not necessary.
  - Don’t create monolithic “god plugins”; keep scope narrow.

- **Integrate with existing patterns**
  - Use the same logging style, error handling style, and folder structure.
  - Respect the project’s existing naming conventions.

---

## 7. Examples of “Good” vs “Bad” AI Behavior

### 7.1 Good

> “You want to read and write per-user config files. This maps cleanly to the `fs` plugin in Tauri v2. Let’s install `@tauri-apps/plugin-fs` and use `BaseDirectory.AppConfig` instead of adding new commands.”

> “You want to persist a small key-value store (last used FM user directory, settings). This is exactly what `@tauri-apps/plugin-store` is for. We can avoid writing a custom config manager.”

### 7.2 Bad

> “Let’s write a custom `#[tauri::command]` that manually opens files with `std::fs` and spawns subprocesses with `std::process::Command` without checking existing plugins or capabilities.”

> “Let’s create a whole new plugin for HTTP calls even though `tauri-plugin-http` already exists and is well maintained.”

---

## 8. How This Interacts With Other Context Files

- **ASSET_ROUTING_SPEC.md**
  - Defines *what* FMMLoader should do with assets/routes.
  - Path logic must **reuse** existing Rust helpers (e.g., `get_fm_user_dir`) and should not reinvent routing.

- **This file (TAURI_PLUGIN_STRATEGY.md)**
  - Defines *how* we build new features on Tauri:
    - Prefer existing plugins.
    - Minimal new tools.
    - Small, careful extensions.

AI should read both when working on any Tauri-related change.

---

## 9. TL;DR for Agents

1. **Check Tauri core + current plugins.** Try to solve the problem there.
2. **Check official v2 plugins** and community plugins that fit the use case.
3. **Only then** propose:
   - A tiny new command, or
   - A minimal plugin with narrow scope.
4. **Keep diffs small.** Do not refactor big chunks of the app unless explicitly asked.

> If you can solve the problem with an existing plugin or configuration change, do that instead of creating new tools.
