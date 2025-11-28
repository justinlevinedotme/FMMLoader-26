# Optimized Prompt (Clavix Enhanced)

Implement internationalization in the Tauri 2.0 React (TypeScript + Vite) desktop app using `tauri-plugin-i18n`, with system locale detection via `tauri-plugin-locale`. Default and fallback to English; support en, ko, tr, pt-PT, de at launch. Store translations as nested JSON files under `src/locales/<lang>.json`.

On startup, detect locale and load if supported; otherwise use English. Provide a Settings-tab language switcher that applies immediately without restart, persists the selection in the existing config file (same place as dark mode), and overrides detection on subsequent launches. Lazy-load locale bundles and cache after first load to keep startup fast.

Translate all user-facing strings: navigation, settings, dialogs, toasts/notifications, error surfaces, and backend-returned messages. Have Rust commands return codes; map those codes to localized strings in the frontend rather than embedding English in Rust. Keep plugin initialization lean to avoid slowing startup. Document how to add new locales (path, key structure, registration). Handle missing/unsupported locales by falling back to English and missing keys by showing a safe fallback (key or English) without crashing.

Success criteria: detected locale or English fallback on startup; runtime language switching without restart; persisted choice survives restart; all UI and backend-coded messages localized for en/ko/tr/pt-PT/de; lazy loading preserves fast initial load and avoids repeated fetches.

---

## Optimization Improvements Applied

1. **[ADDED][Completeness]** Explicit launch locales, storage path for JSON, and fallback behaviors (unsupported detection, missing keys).
2. **[CLARIFIED][Clarity]** Distinguished Rust code responses vs. frontend mappings to avoid hardcoded backend strings.
3. **[STRUCTURED][Structure]** Ordered the flow: detection/init → runtime switch/persistence → translation scope → backend mapping → performance and docs.
4. **[EXPANDED][Actionability]** Stated persistence override of detection, lazy-load + cache expectations, and requirement for immediate runtime switch.
5. **[SCOPED][Specificity]** Defined success criteria with concrete behaviors (startup locale, no-restart switch, coverage across specified languages).

---
*Optimized by Clavix on 2025-11-27. This version is ready for implementation.*
