# Tasks: Tauri i18n Integration

## Phase 1: Foundations
- [x] Add Rust dependencies/config for `tauri-plugin-i18n` and `tauri-plugin-locale`; register plugins in `src-tauri/src/main.rs` with minimal startup overhead.
- [x] Establish backend message code pattern (errors/success) and surface codes instead of hardcoded English strings where applicable.

## Phase 2: Frontend i18n scaffolding
- [ ] Add i18n loader/utilities in React: lazy-load JSON from `src/locales/<lang>.json`, cache bundles, expose translate helper with nested keys and missing-key fallback (key or English).
- [ ] Implement locale detection + persistence flow: use `tauri-plugin-locale` on startup, fallback to `en` if unsupported, and persist user choice in existing config file (same place as dark mode) to override detection on next launch.
- [ ] Seed base locale files for en, ko, tr, pt-PT, de with initial key structure and samples for UI/common messages.

## Phase 3: UI integration
- [ ] Wire i18n provider through the app (main entry + hooks) and replace user-facing strings (navigation, settings, dialogs, toasts/notifications, error surfaces) with translation keys.
- [ ] Add Settings tab language switcher: show available locales, apply at runtime without restart, persist selection to config, and update UI immediately.

## Phase 4: Backend code mapping
- [ ] Map Rust command result codes to frontend translation keys (success/error surfaces) and ensure critical paths (config, mods, graphics, name-fix) present localized strings.

## Phase 5: Hardening and docs
- [ ] Add dev-facing notes/docs: how to add a new locale, key conventions, how to map backend codes, and how to handle missing keys.
- [ ] Add smoke checks: verify detection fallback to English, runtime switching, persistence across restart, and lazy-load/caching behavior; document manual test steps or add lightweight automated checks if feasible.
