# Code Quality & Linting Guide

This project uses a comprehensive linting and formatting setup to maintain consistent, high-quality code across both the frontend (React/TypeScript) and backend (Rust).

## Table of Contents
- [Quick Start](#quick-start)
- [Available Commands](#available-commands)
- [Frontend Linting (ESLint + Prettier)](#frontend-linting-eslint--prettier)
- [Backend Linting (Clippy + Rustfmt)](#backend-linting-clippy--rustfmt)
- [Editor Integration](#editor-integration)
- [CI/CD Integration](#cicd-integration)

## Quick Start

### Run all linters
```bash
npm run lint
```

### Auto-fix all formatting issues
```bash
npm run format
```

### Check formatting without making changes
```bash
npm run format:check
```

## Available Commands

### Frontend (JavaScript/TypeScript)
- `npm run lint:js` - Run ESLint on all JavaScript/TypeScript files
- `npm run lint:js:fix` - Auto-fix ESLint issues where possible
- `npm run format:js` - Format all frontend code with Prettier
- `npm run format:check:js` - Check if frontend code is properly formatted

### Backend (Rust)
- `npm run lint:rust` - Run Clippy on Rust code
- `npm run format:rust` - Format Rust code with rustfmt
- `npm run format:check:rust` - Check if Rust code is properly formatted

### Combined
- `npm run lint` - Run both frontend and backend linters
- `npm run format` - Format both frontend and backend code
- `npm run format:check` - Check formatting for all code

## Frontend Linting (ESLint + Prettier)

### ESLint Configuration
ESLint is configured in `eslint.config.js` with:
- **TypeScript support** via `typescript-eslint`
- **React rules** for React best practices
- **React Hooks rules** to ensure proper hook usage
- **React Refresh** for Vite HMR compatibility
- **Strict type checking** using TypeScript compiler options

Key rules enforced:
- No unused variables (with `_` prefix exception)
- Consistent type imports
- No console.log (console.warn and console.error allowed)
- Prefer const over let
- Prefer template literals over string concatenation
- React prop-types disabled (using TypeScript instead)

### Prettier Configuration
Prettier is configured in `.prettierrc` with:
- Single quotes for strings
- Semicolons required
- 2-space indentation
- 100-character line width
- Trailing commas in ES5-compatible locations
- Unix line endings (LF)

## Backend Linting (Clippy + Rustfmt)

### Clippy Configuration
Clippy is configured in `src-tauri/Cargo.toml` with:
- **Pedantic lints** enabled for better code quality
- **Nursery lints** for experimental but useful checks
- Warnings for `unwrap()`, `expect()`, `panic!()`, `todo!()`, and `unimplemented!()`

Configuration files:
- `src-tauri/Cargo.toml` - Clippy lint levels
- `src-tauri/clippy.toml` - Additional Clippy settings

### Rustfmt Configuration
Rustfmt is configured in `src-tauri/rustfmt.toml` with:
- 100-character line width
- 4-space indentation
- Unix line endings
- Organized imports by crate
- Edition 2021 formatting rules

## Editor Integration

### VS Code
The project includes recommended VS Code settings in `.vscode/`:

1. **Install recommended extensions**:
   - ESLint
   - Prettier
   - Rust Analyzer
   - Tauri
   - Tailwind CSS IntelliSense
   - EditorConfig

2. **Settings are pre-configured** for:
   - Format on save
   - ESLint auto-fix on save
   - Clippy integration with Rust Analyzer

### Other Editors
The `.editorconfig` file provides basic settings for all editors that support EditorConfig:
- Consistent indentation
- Line endings
- Charset
- Trimming trailing whitespace

## CI/CD Integration

### GitHub Actions Example
Add this to your CI workflow to enforce code quality:

```yaml
- name: Lint frontend
  run: npm run lint:js

- name: Check frontend formatting
  run: npm run format:check:js

- name: Lint backend
  run: npm run lint:rust

- name: Check backend formatting
  run: npm run format:check:rust
```

### Pre-commit Hook (Optional)
You can add a pre-commit hook to automatically lint staged files:

```bash
#!/bin/sh
# .git/hooks/pre-commit

echo "Running linters..."
npm run lint:js || exit 1
npm run format:check || exit 1
echo "All checks passed!"
```

## Customization

### Modifying ESLint Rules
Edit `eslint.config.js` to adjust linting rules. See [ESLint Rules](https://eslint.org/docs/rules/) for available options.

### Modifying Prettier Rules
Edit `.prettierrc` to change formatting preferences. See [Prettier Options](https://prettier.io/docs/en/options.html).

### Modifying Clippy Rules
Edit the `[lints.clippy]` section in `src-tauri/Cargo.toml`. See [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/).

### Modifying Rustfmt Rules
Edit `src-tauri/rustfmt.toml`. See [Rustfmt Configuration](https://rust-lang.github.io/rustfmt/).

## Troubleshooting

### ESLint errors about TypeScript config
Make sure your `tsconfig.json` is valid and includes all necessary files.

### Clippy warnings overwhelming
You can temporarily allow specific lints by adding `#[allow(clippy::lint_name)]` above functions or modules.

### Format conflicts between Prettier and ESLint
This shouldn't happen as `eslint-config-prettier` disables conflicting rules. If you see conflicts, make sure Prettier runs after ESLint.

## Best Practices

1. **Run linters before committing** - Use `npm run lint` and `npm run format`
2. **Fix issues incrementally** - Don't disable lints without understanding why they exist
3. **Use TypeScript strictly** - The `tsconfig.json` has strict mode enabled
4. **Handle errors properly in Rust** - Avoid `unwrap()` and `expect()` in production code
5. **Keep dependencies updated** - Regularly update linting tools for new rules and fixes
