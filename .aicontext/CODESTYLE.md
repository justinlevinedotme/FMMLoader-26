# CODESTYLE

Guidance for AI contributors on this repo's conventions.

## Languages
- React + TypeScript (Vite, Tailwind, shadcn/ui) in `src/`.
- Rust 2021 (Tauri v2) in `src-tauri/`.

## Formatting & linting
- JS/TS: `npm run lint` (ESLint) and `npm run format` / `npm run format:fix` (Prettier, width 100, single quotes, trailing commas). Markdown is skipped via `.prettierignore`.
- Rust: `(cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings)`. Use `cargo fmt` to fix.
- Hooks: `npx lint-staged` on commit; pre-push runs branch name validation, `npm run build`, then `cargo check`. Don't bypass unless necessary.

## Imports / organization
- Order: node/stdlib -> third-party -> internal alias imports -> relative paths; keep side-effect imports (e.g., CSS) last among externals.
- Prefer the `@/` alias for code in `src` (`paths` configured in `tsconfig.json`); avoid deep `../../` chains.
- Use type-only imports (`import type { Foo }`) when appropriate; keep React hooks and component imports grouped.
- Follow ESLint defaults here (unused vars must be prefixed with `_` to silence).

## Testing / checks
- Minimum before PR: `npm run build`, `npm run lint`, `npm run format`, and Rust checks: `(cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test)`.
- CI mirrors the above (`.github/workflows/ci.yml`). Release builds happen on `v*` tags via `.github/workflows/build.yml`.

## Commit & branch conventions
- Branch names validated by `validate-branch-name`: `feature|fix|bugfix|hotfix|docs|refactor|test|chore/<slug>` or `main`; lowercase with hyphens.
- Use Conventional Commits (`feat: ...`, `fix: ...`, `chore: ...`, etc.). Keep messages concise and scoped.
- Avoid committing generated artifacts (`dist/`, `node_modules/`, `src-tauri/target/`); they're gitignored.
