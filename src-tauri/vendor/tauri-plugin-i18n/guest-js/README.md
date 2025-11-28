# Tauri Plugin SQLite - JavaScript Bindings (@razein/tauri-plugin-i18n)

This package provides the JavaScript/TypeScript bindings for the `@razein97/tauri-plugin-i18n` Tauri plugin.

## Installation

You need to install both the Rust Core plugin and these JavaScript bindings.

See the [main plugin README](../../README.md) for instructions on setting up the Rust Core plugin in your `Cargo.toml`.

Install the JavaScript bindings using your preferred package manager:

```bash
# Using pnpm
pnpm add @razein97/tauri-plugin-i18n

# Using npm
npm install @razein97/tauri-plugin-i18n

# Using yarn
yarn add @razein97/tauri-plugin-i18n
```

### Rust bindings

Install the rust package using cargo:

```sh
cargo add tauri-plugin-i18n
```

## Usage

Import the `I18n` class and use the `getInstance` method to initialize the plugin.

```typescript
import I18n from '@razein97/tauri-plugin-i18n';

async function initializePlugin() {
  // Load translations
  await I18n.getInstance().load();

  // Example
  //Get available locales
  const locales = await I18n.getAvailableLocales();
  console.log('Locales:', locales);
}

initializePlugin();
```

Refer to the [main plugin README](../../README.md) for detailed API documentation.
