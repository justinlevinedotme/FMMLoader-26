---
id: T-0002
title: "Chore: repo cleanup"
status: in_progress
priority: medium
owner: ""
created_at: 2025-11-18
updated_at: 2025-11-18
tags: ["cleanup"]
---

## 0. User story / problem

The repository has many README variants and other files that seem unused. You want a cleanup pass to remove redundant docs, unused build scripts, and other cruft so the repo is lean and only keeps what is necessary.

---

## 1. Context gathered by AI(s)

### 1.1 Relevant files / dirs

- Root docs: `README.md`, `BUILD.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `PLATFORM_SUPPORT.md`, `SECURITY_REVIEW.md`, `privacy.md`.
- Build scripts: `build-local.sh`, `build-local.bat`; package scripts in `package.json` (`build`, `build:debug`, `build:release`, `lint`, `format`, etc.).
- Generated or dependency dirs present historically: `dist/`, `node_modules/`, `sessions/`, `.claude/.backup-202511112022217/`.
- Source: `src/` (frontend), `src-tauri/` (Rust backend), `index.html`, `vite.config.ts`, `tsconfig*.json`.
- CI/CD: `.github/workflows/build.yml` (tagged Tauri release), `ci.yml` (build/test/lint), `claude.yml` (bot), `jalco-repoAI-auto-sync.yml` (task sync).
- Additional docs/assets: `.github/BRANCH_PROTECTION.md`, `.github/copilot-instructions.md`, `.github/imgs/`, `.github/ISSUE_TEMPLATE/*`, `.github/PULL_REQUEST_TEMPLATE.md`.
- Examples/content: previously `example mods/` (removed per cleanup).

### 1.2 Summary of current behavior

- Tauri v2 + React/Vite app. Builds run TS compile then Tauri bundle (`npm run build:release` for optimized, `build:debug` for faster) with helper scripts `build-local.{sh,bat}` referenced in docs.
- Tag-based release workflow (`.github/workflows/build.yml`) packages and publishes installers for macOS (ARM/Intel), Windows, Linux on `v*` tags.
- CI (`.github/workflows/ci.yml`) runs npm build/lint/format and Rust check/test/clippy/fmt on pushes/PRs to `main`.
- `dist/` and `node_modules/` exist in the repo root, implying generated artifacts/dependencies may be committed or leftover locally.
- Docs are duplicated/overlapping: `BUILD.md` (quick start/outputs) vs `BUILD_INSTRUCTIONS.md` (prereqs + installs); several standalone docs may overlap with README.
- Extras: `example mods/` sample content; `.claude/.backup-...` backup data; `sessions/` appears tooling-generated.

### 1.3 Risks / constraints / assumptions

- Dropping docs without merging could lose unique info (platform deps vs quick start); safer to consolidate before removing.
- Removing `dist/`/`node_modules/` is normally safe, but confirm they aren’t intentionally committed for distribution/testing.
- Release automation depends on `build.yml`; pruning workflows without replacement would break tagged releases.
- `build-local` scripts are referenced in docs; removing them requires doc updates to keep a working local build path.
- Tooling artifacts (`.claude/...`, `sessions/`) might be harmless to drop, but validate no automation consumes them.

### 1.4 Open questions for the human

- Docs: merge `BUILD.md` and `BUILD_INSTRUCTIONS.md` into one guide and delete the duplicate. No other docs called out to keep unchanged.
- Distribution: releases are via GitHub (see `build.yml`); assume Tauri installers are the only distribution—OK to remove npm/package publish remnants if present.
- Workflows: no additional release channels beyond `build.yml` (release) and `ci.yml` (tests) to preserve.
- Generated/tooling artifacts: OK to delete committed `dist/`, `node_modules/`, `.claude/.backup-...`, ensure they’re ignored; do not delete `sessions/`.
- Samples/assets: remove `example mods/` and references (e.g., in README); documentation will move to a separate repo.

---

## 2. Proposed success criteria (AI draft)

- Consolidate build docs: merge `BUILD.md` and `BUILD_INSTRUCTIONS.md` into a single up-to-date build guide; remove the redundant file; ensure README (or linked doc) still points to the correct build instructions.
- Confirm distribution scope: align build/release docs and workflows to GitHub tag-based Tauri installers; remove any npm/package publish remnants (scripts/config/workflows) if present.
- CI/release workflows: retain `build.yml` (tagged releases) and `ci.yml` (tests); remove or archive unused CI workflows if found; verify release CI paths remain intact.
- Generated/tooling artifacts: remove committed `dist/`, `node_modules/`, `.claude/.backup-...`; ensure `.gitignore` covers these; preserve `sessions/`.
- Samples/assets cleanup: remove `example mods/` directory and any README references or links to it.

---

## 3. Approved success criteria (human-edited)

- Consolidate build docs: merge `BUILD.md` and `BUILD_INSTRUCTIONS.md` into a single up-to-date build guide; remove the redundant file; ensure README (or linked doc) still points to the correct build instructions.
- Confirm distribution scope: align build/release docs and workflows to GitHub tag-based Tauri installers; remove any npm/package publish remnants (scripts/config/workflows) if present.
- CI/release workflows: retain `build.yml` (tagged releases) and `ci.yml` (tests); remove or archive unused CI workflows if found; verify release CI paths remain intact.
- Generated/tooling artifacts: remove committed `dist/`, `node_modules/`, `.claude/.backup-...`; ensure `.gitignore` covers these; preserve `sessions/`.
- Samples/assets cleanup: remove `example mods/` directory and any README references or links to it.


---

## 4. Implementation log / notes

- 2025-11-18 – intake: created task and captured open questions
- 2025-11-18 – implementation: merged BUILD docs into single BUILD.md (added platform prereqs, setup, build/release/CI scope), removed BUILD_INSTRUCTIONS.md, and linked README to BUILD.md.
- 2025-11-18 – implementation: removed generated/tooling artifacts (dist, node_modules, .claude/.backup-202511112022217), added .claude backup ignore, preserved sessions/.
- 2025-11-18 – implementation: removed `example mods/` and README references; confirmed no npm/package publish workflows present.
- 2025-11-18 – implementation: renamed task file/title to “Chore: repo cleanup”; attempted to rename branch to `chore/repo-cleanup` but branch creation/rename failed due to `.git/refs` permissions in this environment.

---

## 5. Completion checklist & review

### 5.1 Human review notes

### 5.2 Follow-up tasks (spinoffs)
