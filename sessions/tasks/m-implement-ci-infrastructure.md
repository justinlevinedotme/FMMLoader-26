---
name: m-implement-ci-infrastructure
branch: feature/ci-infrastructure
status: pending
created: 2025-01-17
---

# CI/CD Infrastructure Implementation

## Problem/Goal
Set up comprehensive CI/CD infrastructure for FMMLoader-26 to ensure code quality, consistent workflows, and streamlined collaboration. This includes automated testing, linting, branch protections, and standardized templates for issues and pull requests.

## Success Criteria

**CI/CD Workflows:**
- [ ] GitHub Actions workflow runs on all PRs (build, test, lint)
- [ ] Rust backend: cargo check, cargo test, cargo clippy passing
- [ ] Frontend: npm build, npm test, ESLint passing
- [ ] Workflow fails on any errors, blocking merge
- [ ] Successful build required before push to main

**Branch Protections:**
- [ ] Main branch requires PR reviews before merge
- [ ] Main branch requires status checks to pass (CI workflow)
- [ ] Direct pushes to main disabled
- [ ] Build must pass before merge allowed

**PR Templates:**
- [ ] Template prompts for description of changes
- [ ] Template includes checklist (tests added, docs updated, etc.)
- [ ] Template asks about breaking changes

**Issue Templates:**
- [ ] Bug report template (steps to reproduce, expected vs actual behavior)
- [ ] Feature request template (use case, proposed solution)
- [ ] Templates auto-label issues appropriately

**Linting Integration:**
- [ ] ESLint + Prettier configured for TypeScript/React frontend
- [ ] Cargo fmt + clippy configured for Rust backend
- [ ] Pre-commit hooks optional (documented but not forced)
- [ ] CI enforces formatting standards

## Context Manifest

### How The Current Build System Works

**Build Architecture - Tauri v2 with Dual Technology Stack:**

FMMLoader26 is a Tauri v2 desktop application combining a Rust backend (in `src-tauri/`) with a React/TypeScript frontend (in `src/`). The build system operates in two parallel tracks that come together for final packaging.

**Frontend Build Flow:**

The frontend uses Vite as the build tool, configured in `vite.config.ts`. When you run `npm run build`, TypeScript compilation happens first via `tsc` (configured in `tsconfig.json` with strict mode enabled, including `noUnusedLocals` and `noUnusedParameters`). The tsconfig specifies ES2020 as the target and uses the bundler module resolution. After TypeScript compilation, Vite builds the React application, processing JSX with the `@vitejs/plugin-react` plugin. The build output goes to the `dist/` directory, which is then referenced by Tauri's config at `tauri.conf.json` via the `frontendDist` setting.

The frontend has a path alias configured (`@/*` maps to `./src/*`) that works in both TypeScript and Vite, allowing imports like `import { Button } from "@/components/ui/button"`. This is critical for CI because any build must respect these path mappings.

**Backend Build Flow:**

The Rust backend is defined in `src-tauri/Cargo.toml`. It's a standard Rust 2021 edition project with multiple dependencies including Tauri plugins (dialog, fs, os, process, shell, updater), serde for serialization, and several utility crates (zip, walkdir, regex, chrono, uuid, reqwest, tracing). The backend compiles to native code for each platform target.

The main entry point is `src-tauri/src/main.rs`, which sets up a modular architecture with separate modules for config, conflicts, game detection, import, logging, mod management, name fix functionality, restore points, and types. Each module exposes Tauri commands that the frontend can invoke.

**Integrated Build via Tauri:**

When you run `npm run tauri build`, Tauri orchestrates both builds. First, it runs the `beforeBuildCommand` from `tauri.conf.json`, which is `npm run build`, compiling the frontend. Then it compiles the Rust backend with cargo. Finally, it packages everything into platform-specific bundles (DMG for macOS, MSI/EXE for Windows, AppImage/DEB for Linux). The Tauri config specifies `createUpdaterArtifacts: true`, which generates signing artifacts for the built-in updater system.

**Development Workflow:**

For development, `npm run dev` starts Vite's dev server on port 1420 with HMR enabled. The `npm run tauri` command provides access to Tauri CLI features. There's also `npm run build:debug` and `npm run build:release` for creating debug and release Tauri builds respectively.

**Platform-Specific Concerns:**

The existing `build.yml` workflow already handles cross-platform builds. It builds for macOS (both ARM64 and x86_64), Windows, and Linux (Ubuntu 22.04). On Ubuntu, it installs webkit2gtk, libappindicator3, and other dependencies. The workflow uses `tauri-apps/tauri-action@v0` which handles all the complexity of building and signing releases. This is ONLY for tagged releases (tags matching `v*`), NOT for CI/testing on PRs.

### What Testing Infrastructure Currently Exists

**Rust Testing:**

The Rust backend HAS existing tests. Both `src-tauri/src/import.rs` and `src-tauri/src/mod_manager.rs` contain `#[cfg(test)]` modules with unit tests. For example, `mod_manager.rs` includes tests like `test_get_current_platform()` which verifies platform detection logic. The `import.rs` file has multiple tests (I counted 11 test functions there).

These tests follow standard Rust testing conventions - they're embedded in the source files within `#[cfg(test)]` modules, so they only compile during `cargo test` runs. The tests use the standard `#[test]` attribute and assertion macros like `assert!` and `assert_eq!`.

To run Rust tests, you use `cargo test` from the `src-tauri/` directory. This compiles the test modules and executes all test functions. The tests are ACTUAL functional tests that currently exist in the codebase, not placeholders.

**Frontend Testing:**

The frontend has NO test infrastructure. There are:
- No test files (no `.test.ts`, `.test.tsx`, `.spec.ts`, or `.spec.tsx` files)
- No `__tests__/` directories
- No testing framework dependencies in package.json (no Jest, Vitest, React Testing Library, etc.)
- No test scripts in package.json

This is a greenfield situation for frontend testing. If we want to add frontend tests to CI, we'd need to first install a testing framework (Vitest would be the natural choice given the project uses Vite), configure it, and write some basic tests.

**Code Quality Tools - What Exists:**

The project has ZERO linting or formatting configuration files. Specifically:
- No `.eslintrc.*` or `eslint.config.*`
- No `.prettierrc.*` or `prettier.config.*`
- No `rustfmt.toml` for Rust formatting
- No `.editorconfig`

However, the TypeScript compiler IS configured with linting-style flags in `tsconfig.json`: `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`, `noFallthroughCasesInSwitch: true`. This means TypeScript compilation itself enforces some code quality standards, but there's no separate ESLint checking for code style, best practices, or React-specific patterns.

For Rust, the standard `cargo fmt` and `cargo clippy` commands would work out of the box without custom configuration. Clippy is Rust's official linter and fmt is the official formatter.

### What CI/CD Infrastructure Currently Exists

**Existing Workflows:**

There is ONE GitHub Actions workflow: `.github/workflows/build.yml`. This workflow is ONLY for releases, not for PR validation. It triggers on tag pushes (`tags: v*`) and builds signed, release-ready binaries for all platforms using a matrix strategy. The workflow:
1. Sets up Node.js with npm caching
2. Installs Rust toolchain with platform-specific targets
3. Installs system dependencies (Ubuntu only)
4. Runs `npm ci` for reproducible installs
5. Uses `tauri-apps/tauri-action@v0` which handles building and release publishing
6. Requires secrets: GITHUB_TOKEN, TAURI_SIGNING_PRIVATE_KEY, TAURI_SIGNING_PRIVATE_KEY_PASSWORD

This workflow does NOT run on PRs, does NOT run tests, does NOT check linting, and does NOT validate code quality. It's purely for creating release artifacts.

**Existing Templates:**

The repository HAS GitHub templates but they're incomplete:

**Pull Request Template** (`.github/PULL_REQUEST_TEMPLATE.md`): EXISTS and is reasonably complete. It includes sections for Summary, Changes (checklist with code improvements, new feature, bug fix, documentation), Verification steps, and Notes. This provides a foundation but could be enhanced with reminders about testing and breaking changes.

**Issue Templates** (`.github/ISSUE_TEMPLATE/`):
- `bug_report.yml`: EXISTS and is well-structured with fields for description, platform, version, and optional logs. Uses YAML form syntax with proper labels.
- `question.yml`: EXISTS and is minimal but functional with a single question textarea.
- `feature_request.yml`: EXISTS but is EMPTY (0 bytes). This needs to be created from scratch.

**Branch Protection:**

There are NO branch protections visible from the repository. The main branch is `main` (confirmed from git status), but there's no evidence of:
- Required PR reviews
- Required status checks
- Restrictions on direct pushes
- Required linear history
- Required conversation resolution

These would need to be configured via GitHub repository settings, not via code in the repository.

### Build Commands and Entry Points

**Package.json Scripts:**

```json
"scripts": {
  "dev": "vite",
  "build": "tsc && vite build",
  "build:debug": "npm run build && tauri build --debug",
  "build:release": "npm run build && tauri build",
  "preview": "vite preview",
  "tauri": "tauri"
}
```

For CI purposes:
- `npm run build` - Compiles TypeScript then builds frontend (suitable for CI validation)
- Frontend alone doesn't need the full Tauri build for validation
- TypeScript compilation happens separately before Vite build

**Cargo Commands:**

The Rust backend is in `src-tauri/`, so all cargo commands need to run from that directory:
- `cargo check` - Fast compile check without building (best for CI quick feedback)
- `cargo build` - Full build
- `cargo test` - Run all tests (including the existing ones in import.rs and mod_manager.rs)
- `cargo clippy` - Linting (would need `-- -D warnings` to fail CI on warnings)
- `cargo fmt --check` - Check formatting without modifying files

**Full Application Build:**

For complete builds (not needed for basic CI, only for releases):
- Tauri config at `src-tauri/tauri.conf.json` orchestrates the process
- Running from root: `npm run tauri build`
- The existing release workflow uses `tauri-apps/tauri-action@v0` which is the recommended approach

### Project Structure and Conventions

**Directory Layout:**

```
/
├── .github/
│   ├── workflows/
│   │   └── build.yml (release workflow only)
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.yml (complete)
│   │   ├── feature_request.yml (EMPTY - needs creation)
│   │   └── question.yml (minimal but functional)
│   ├── PULL_REQUEST_TEMPLATE.md (exists, functional)
│   ├── CONTRIBUTING.md (outdated - references Python, needs updating)
│   └── CODE_OF_CONDUCT.md
├── src/ (React/TypeScript frontend)
│   ├── components/
│   ├── hooks/
│   ├── lib/
│   └── App.tsx
├── src-tauri/ (Rust backend)
│   ├── src/
│   │   ├── main.rs
│   │   ├── import.rs (HAS TESTS)
│   │   ├── mod_manager.rs (HAS TESTS)
│   │   └── [other modules]
│   ├── Cargo.toml
│   └── tauri.conf.json
├── sessions/ (cc-sessions task framework)
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
└── postcss.config.js
```

**File Organization Conventions:**

- Frontend: Components use PascalCase filenames (e.g., `TitleBar.tsx`)
- UI components are in `src/components/ui/` (shadcn/ui pattern)
- Hooks are in `src/hooks/`
- Rust modules use snake_case (e.g., `mod_manager.rs`)
- Tests are embedded in Rust source files, not separate test files

**Technology Stack Summary:**

Frontend:
- React 18 with TypeScript
- Vite for building and dev server
- Tailwind CSS with PostCSS
- shadcn/ui component library
- Path aliases via `@/*` mapping

Backend:
- Rust 2021 edition
- Tauri v2 framework
- Multiple Tauri plugins
- Tracing for logging
- Serde for serialization

**Dependencies to Consider for CI:**

The project uses npm (not yarn or pnpm), evidenced by the workflow using `npm ci` and package-lock.json would exist. For reproducible CI builds, always use `npm ci` not `npm install`.

Rust dependencies are locked in `Cargo.lock` (which exists in src-tauri/). Both lockfiles should be committed and respected in CI.

### What Needs to Be Created for Complete CI/CD

**New GitHub Actions Workflow** (`.github/workflows/ci.yml`):

This workflow needs to:
1. Trigger on pull requests to main and pushes to main
2. Run on Ubuntu (fastest for CI, covers core functionality)
3. Set up Node.js with npm caching
4. Set up Rust toolchain (stable)
5. Frontend checks:
   - `npm ci` for reproducible installs
   - `npm run build` to verify TypeScript compilation and frontend builds
6. Backend checks:
   - `cargo check` (fast validation)
   - `cargo test` (run existing tests)
   - `cargo clippy -- -D warnings` (fail on clippy warnings)
   - `cargo fmt --check` (verify formatting)
7. Exit with failure if ANY check fails

**Linting Configuration to Create:**

For ESLint (TypeScript/React):
- Install: `@typescript-eslint/parser`, `@typescript-eslint/eslint-plugin`, `eslint-plugin-react`, `eslint-plugin-react-hooks`
- Create `eslint.config.js` (flat config, modern style) or `.eslintrc.cjs`
- Configure rules for TypeScript, React, and hooks
- Add script: `"lint": "eslint src --ext ts,tsx"`

For Prettier (optional but recommended):
- Install: `prettier`
- Create `.prettierrc` with basic config
- Add script: `"format:check": "prettier --check src"`
- Consider `eslint-config-prettier` to prevent conflicts

For Rust:
- No files needed, just use `cargo fmt` and `cargo clippy` defaults
- Could create `rustfmt.toml` for custom formatting if desired
- Could create `clippy.toml` for custom lint rules if desired

**Issue Template to Create:**

The `feature_request.yml` template is empty and needs content. Based on the bug_report.yml pattern, it should include:
- Title prefix
- Labels: `[enhancement]` or `[feature]`
- Description field (required)
- Use case explanation (optional)
- Proposed solution (optional)
- Additional context (optional)

**Documentation Updates:**

The CONTRIBUTING.md references Python and is completely outdated. It needs rewriting to cover:
- The Tauri + React + Rust stack
- How to set up development environment (Node.js + Rust)
- How to run tests (`cargo test`)
- Code style expectations (once ESLint/Prettier are set up)
- How to run the app in dev mode (`npm run tauri dev`)
- Build instructions (`npm run tauri build`)

**Branch Protection Settings:**

These must be configured via GitHub UI (Settings → Branches → Branch protection rules):
- Protect `main` branch
- Require pull request before merging
- Require 1 approval (or configure as needed)
- Require status checks to pass:
  - The new CI workflow checks
- Dismiss stale reviews on new commits
- Require conversation resolution
- Do not allow bypassing the above settings (remove admin bypass if desired)

**Pre-commit Hooks (Optional, Documented but Not Forced):**

The task specifies pre-commit hooks should be optional and documented. Could document use of:
- `husky` for Git hooks
- `lint-staged` for running linters only on staged files
- Document in CONTRIBUTING.md how to set up, but don't force installation

### Technical Reference Details

#### Available npm Scripts

```json
{
  "dev": "vite",                                    // Dev server
  "build": "tsc && vite build",                     // Build frontend
  "build:debug": "npm run build && tauri build --debug",
  "build:release": "npm run build && tauri build",
  "preview": "vite preview",                        // Preview built frontend
  "tauri": "tauri"                                  // Tauri CLI access
}
```

#### Recommended New Scripts to Add

```json
{
  "lint": "eslint src --ext ts,tsx",
  "lint:fix": "eslint src --ext ts,tsx --fix",
  "format": "prettier --write src",
  "format:check": "prettier --check src",
  "test": "vitest",  // If frontend tests are added
  "test:rust": "cd src-tauri && cargo test"
}
```

#### Cargo Commands for CI

All run from `src-tauri/` directory:

```bash
cargo check                     # Fast compile check (good for quick CI)
cargo build                     # Full build
cargo test                      # Run tests (18 tests exist currently)
cargo clippy -- -D warnings     # Lint with warnings as errors
cargo fmt --check               # Check formatting (doesn't modify)
```

#### GitHub Actions Workflow Trigger Patterns

For PR validation workflow:
```yaml
on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
```

For release workflow (already exists):
```yaml
on:
  push:
    tags: ["v*"]
```

#### File Locations for New Configuration

- ESLint config: `eslint.config.js` (root) or `.eslintrc.cjs` (root)
- Prettier config: `.prettierrc` (root) or `prettier.config.js` (root)
- Rust fmt config (optional): `src-tauri/rustfmt.toml`
- Rust clippy config (optional): `src-tauri/clippy.toml`
- CI workflow: `.github/workflows/ci.yml` (NEW FILE)
- Feature request template: `.github/ISSUE_TEMPLATE/feature_request.yml` (OVERWRITE EMPTY FILE)

#### Platform Dependencies for CI Runners

Ubuntu (recommended for CI):
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

Already documented in existing `build.yml` workflow.

#### Node and Rust Versions in Use

Based on system and workflow:
- Node: v25.2.0 locally, workflow uses `node-version: lts/*` (recommended for CI too)
- npm: 11.6.2
- Rust: 1.91.0 locally, workflow uses `dtolnay/rust-toolchain@stable` (recommended for CI)

For CI, use:
- `actions/setup-node@v4` with `node-version: lts/*` and `cache: 'npm'`
- `dtolnay/rust-toolchain@stable` (or specify minimum version like `@1.70`)

#### Expected CI Runtime

Based on the build complexity:
- Frontend build (npm ci + tsc + vite): ~2-3 minutes
- Rust check: ~1-2 minutes
- Rust tests: ~1-2 minutes
- Rust clippy: ~1-2 minutes
- Total expected CI time: ~5-10 minutes on Ubuntu runners

#### Success Criteria Mapping

Task requirement → Implementation:
- "Rust backend: cargo check, cargo test, cargo clippy passing" → Add to CI workflow
- "Frontend: npm build, npm test, ESLint passing" → npm build exists; need to add ESLint and optionally frontend tests
- "Successful build required before push to main" → Branch protection rule requiring CI workflow to pass
- "Main branch requires PR reviews before merge" → Branch protection setting
- "Direct pushes to main disabled" → Branch protection setting
- "ESLint + Prettier configured" → Need to install and configure
- "Cargo fmt + clippy configured" → Work out of box, just add to workflow
- "PR template includes checklist" → Exists, may need enhancement
- "Bug report template" → Exists and is good
- "Feature request template" → Empty, needs creation

## User Notes
<!-- Any specific notes or requirements from the developer -->

## Work Log
<!-- Updated as work progresses -->
- [YYYY-MM-DD] Started task, initial research
