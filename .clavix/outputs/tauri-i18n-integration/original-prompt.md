# Original Prompt (Extracted from Conversation)

Build i18n into the Tauri 2.0 React (TypeScript + Vite) desktop app using `tauri-plugin-i18n` and detect the system locale on startup with `tauri-plugin-locale`, defaulting and falling back to English if detection fails or the locale is unsupported. Support English, Korean, Turkish, Portuguese (pt-PT), and German at launch, storing translations as nested JSON under `src/locales/<lang>.json`.

Provide a language switcher in the Settings tab that applies immediately at runtime without restart, and persist the user’s choice in the existing config file (same storage as dark mode) so it overrides system detection on next launch. Lazy-load locale bundles and cache them after first load to keep startup fast. Translate all user-facing strings: navigation, settings, dialogs, toasts/notifications, error surfaces, and backend-returned messages.

For backend strings, use code-based responses from Rust commands and map them to translated UI strings on the frontend instead of hardcoding English in Rust. Keep plugin initialization lean so it doesn’t slow app startup, and document how to add new locales (path, keys, registration). Handle unsupported/missing locales by falling back to English, and handle missing keys gracefully (e.g., show key or English).

Success means: detected locale or English fallback on startup; runtime switcher in Settings persists through config and works without restart; all user-facing strings and backend-coded messages localized for en/ko/tr/pt-PT/de; lazy loading keeps initial load fast.

---
*Extracted by Clavix on 2025-11-27. See optimized-prompt.md for enhanced version.*
