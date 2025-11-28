if ('__TAURI__' in window) {
    var __TAURI_PLUGIN_i18n__ = (function () {
        'use strict';
        async function e(t) {
            return window.__TAURI_INTERNALS__.invoke(e);
        }
        'function' == typeof SuppressedError && SuppressedError;
        class t {
            static async translate() {
                const n = await e('plugin:i18n|translate');
                return new t(n);
            }

            async setLocale(t) {
                const n = await e('plugin:i18n|set_locale', {
                    locale: t,
                });
                return n;
            }
            async getLocale() {
                return await e('plugin:i18n|get_locale');
            }
            async getAvailableLocales() {
                return await e('plugin:i18n|get_available_locales');
            }


        }
        return t;
    })();
    Object.defineProperty(window.__TAURI__, 'i18n', {
        value: __TAURI_PLUGIN_RUSQLITE2__,
    });
}