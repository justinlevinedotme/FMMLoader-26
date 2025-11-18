# Contributing to FMMLoader26

Thank you for your interest in contributing to FMMLoader26! This document provides guidelines and instructions for contributing to the project.

## Tech Stack

FMMLoader26 is a desktop application built with:

**Frontend:**
- React 18 with TypeScript
- Vite (build tool)
- Tailwind CSS for styling
- shadcn/ui component library

**Backend:**
- Rust 2021 edition
- Tauri v2 framework
- Tokio for async operations

## Development Setup

### Prerequisites

- Node.js (LTS version recommended)
- npm or yarn
- Rust (latest stable)
- Cargo
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`
  - **Windows**: Microsoft C++ Build Tools

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/justinlevinedotme/FMMLoader-26.git
   cd FMMLoader-26
   ```

2. Install frontend dependencies:
   ```bash
   npm install
   ```

3. Start the development server:
   ```bash
   npm run dev
   ```

This will start both the Vite dev server and the Tauri development window.

## Code Quality

### Frontend

We use ESLint and Prettier to maintain code quality:

```bash
# Check linting
npm run lint

# Auto-fix linting issues
npm run lint:fix

# Check formatting
npm run format

# Auto-format code
npm run format:fix
```

**Important:** All code must pass ESLint and Prettier checks before being merged.

### Backend (Rust)

We use `cargo clippy` and `cargo fmt` for Rust code:

```bash
# Navigate to Tauri directory
cd src-tauri

# Check formatting
cargo fmt --check

# Auto-format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test
```

**Important:** All Rust code must:
- Pass `cargo clippy` with no warnings
- Be formatted with `cargo fmt`
- Pass all tests with `cargo test`

## Testing

### Frontend Tests
Currently, we don't have frontend tests set up. Contributions to add testing infrastructure are welcome!

### Backend Tests
Run Rust tests with:
```bash
cd src-tauri
cargo test
```

New features should include appropriate test coverage.

## Building

### Development Build
```bash
npm run build:debug
```

### Production Build
```bash
npm run build:release
```

## Branch Naming Conventions

All branches must follow these naming conventions. The pre-push hook automatically validates branch names using the `validate-branch-name` package configured in `package.json`.

**Branch name format**: `<type>/<description-with-hyphens>`

**Allowed types:**
- `feature/*` - New features (e.g., `feature/add-user-auth`)
- `fix/*` - Bug fixes (e.g., `fix/login-error`)
- `bugfix/*` - Bug fixes (alternative, e.g., `bugfix/null-pointer`)
- `hotfix/*` - Urgent production fixes (e.g., `hotfix/security-patch`)
- `docs/*` - Documentation changes (e.g., `docs/api-reference`)
- `refactor/*` - Code refactoring (e.g., `refactor/extract-utils`)
- `test/*` - Test additions/changes (e.g., `test/integration-suite`)
- `chore/*` - Maintenance tasks (e.g., `chore/update-dependencies`)

**Rules:**
- Branch names must be **lowercase**
- Use **hyphens** to separate words (not underscores or spaces)
- The `main` branch is always allowed

**Example valid branch names:**
```bash
git checkout -b feature/graphics-pack-import
git checkout -b fix/namefix-validation
git checkout -b docs/contributing-guide
```

**Invalid branch names** (will be rejected by pre-push hook):
```bash
git checkout -b add-feature           # Missing type prefix
git checkout -b my-branch             # Doesn't match pattern
git checkout -b FEATURE/test          # Wrong case (must be lowercase)
git checkout -b feature/Add_Feature   # Uppercase and underscores not allowed
```

**Configuration**: Branch naming rules are defined in `package.json` under the `validate-branch-name` key. To modify the rules, edit that configuration.

## Pull Request Process

1. **Fork the repository** and create a new branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the code style guidelines above.

3. **Test your changes** thoroughly:
   - Run the development build and test functionality
   - Ensure all linters pass (`npm run lint`, `cargo clippy`)
   - Ensure all tests pass (`cargo test`)

4. **Commit your changes** with clear, descriptive commit messages:
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

   We follow conventional commit format:
   - `feat:` - New features
   - `fix:` - Bug fixes
   - `docs:` - Documentation changes
   - `style:` - Code style changes (formatting, etc.)
   - `refactor:` - Code refactoring
   - `test:` - Adding or updating tests
   - `chore:` - Maintenance tasks

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Create a Pull Request** against the `main` branch.

### PR Requirements

Before your PR can be merged, it must:

- ✅ Pass all CI checks (build, lint, test)
- ✅ Have a clear description of changes
- ✅ Reference any related issues
- ✅ Be reviewed and approved by a maintainer

### CI Pipeline

Our CI automatically runs on all pull requests:
- Frontend build (`npm run build`)
- ESLint checks (`npm run lint`)
- Prettier formatting check (`npm run format`)
- Rust compilation (`cargo check`)
- Rust tests (`cargo test`)
- Rust linting (`cargo clippy`)
- Rust formatting check (`cargo fmt --check`)

**PRs cannot be merged if CI fails.**

## Code Style Guidelines

### TypeScript/React
- Use functional components with hooks
- Use TypeScript types/interfaces (avoid `any`)
- Follow React best practices (avoid unnecessary re-renders, proper dependency arrays)
- Keep components focused and single-purpose
- Use meaningful variable and function names

### Rust
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Use meaningful error types and proper error handling
- Document public APIs with doc comments (`///`)
- Avoid unwrap() in production code - use proper error handling
- Keep functions focused and modular

### General
- Write clear, self-documenting code
- Add comments for complex logic
- Keep lines under 100 characters when practical
- Use consistent indentation (2 spaces for TS/JS, 4 spaces for Rust)

## Pre-commit Hooks

This project uses **husky** and **lint-staged** to automatically check code quality before commits and enforce branch naming conventions before pushes.

### Automatic Setup

When you run `npm install`, husky will automatically set up the git hooks via the `prepare` script in `package.json`. No additional setup is required!

### What Gets Checked

**Pre-commit hook** (runs on `git commit`):
- **TypeScript/React files** (`*.{ts,tsx,js,jsx}`): ESLint auto-fix + Prettier formatting
- **JSON/CSS/Markdown files**: Prettier formatting
- **Rust files** (`src-tauri/src/**/*.rs`): `cargo fmt`

**Pre-push hook** (runs on `git push`):
- **Branch naming validation**: Ensures your branch follows the naming conventions (configured in `package.json`)
- **Frontend build check**: Runs `npm run build` to catch TypeScript/build errors
- **Rust build check**: Runs `cargo check` to verify Rust code compiles

### How It Works

**Pre-commit**: Only runs on **staged files**, so it's fast and won't modify uncommitted changes.

**Pre-push**: Runs full build checks to catch errors before pushing. This takes longer but provides fast feedback before CI runs.

If any checks fail, the commit/push will be blocked, and you'll see helpful error messages.

### Skipping Hooks (Not Recommended)

If you absolutely need to skip hooks (e.g., for a work-in-progress commit), you can use:
```bash
git commit --no-verify
git push --no-verify
```

**Warning:** Skipping hooks may result in CI failures. Only skip when necessary and ensure you fix issues before creating a PR.

## Getting Help

- **Discord**: Join our [Discord server](https://discord.gg/AspRvTTAch)
- **Issues**: Check existing [issues](https://github.com/justinlevinedotme/FMMLoader-26/issues)
- **Discussions**: Start a [discussion](https://github.com/justinlevinedotme/FMMLoader-26/discussions)

## License

By contributing to FMMLoader26, you agree that your contributions will be licensed under the project's license.

---

Thank you for contributing to FMMLoader26! 🎉
