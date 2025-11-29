#!/usr/bin/env tsx
/**
 * I18n Configuration Generator
 *
 * Automatically generates i18n configuration across the codebase based on
 * locale JSON files in src/locales/. This eliminates manual updates in 6 locations.
 *
 * Updates:
 * 1. src/lib/i18n.tsx - SUPPORTED_LOCALES array and normalizeLocale function
 * 2. src/App.tsx - Flag imports and ALL_LOCALE_OPTIONS array
 * 3. src/components/tabs/SettingsTab.tsx - localeOptions array
 * 4. crowdin.yml - languages_mapping
 *
 * Usage: pnpm run generate:i18n
 */

import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Locale configuration with metadata
interface LocaleConfig {
  code: string;
  countryCode: string;
  label: string;
  contributor?: string;
}

// Locale to country code mapping for flags
const LOCALE_TO_COUNTRY: Record<string, string> = {
  en: 'US',
  'en-GB': 'GB',
  ko: 'KR',
  tr: 'TR',
  'pt-PT': 'PT',
  de: 'DE',
  it: 'IT',
  nl: 'NL',
  es: 'ES',
  fr: 'FR',
  ja: 'JP',
  'zh-CN': 'CN',
  'zh-TW': 'TW',
};

// Human-readable labels for each locale
const LOCALE_LABELS: Record<string, string> = {
  en: 'English (US)',
  'en-GB': 'English (UK)',
  ko: '한국어',
  tr: 'Türkçe',
  'pt-PT': 'Português (Portugal)',
  de: 'Deutsch',
  it: 'Italiano',
  nl: 'Nederlands',
  es: 'Español',
  fr: 'Français',
  ja: '日本語',
  'zh-CN': '简体中文',
  'zh-TW': '繁體中文',
};

/**
 * Scan src/locales/ and extract locale codes
 */
function discoverLocales(): LocaleConfig[] {
  const localesDir = path.join(__dirname, '../src/locales');
  const files = fs.readdirSync(localesDir);

  const locales: LocaleConfig[] = files
    .filter((file) => file.endsWith('.json') && file !== 'README.json')
    .map((file) => {
      const code = file.replace('.json', '');
      const countryCode = LOCALE_TO_COUNTRY[code] || code.toUpperCase().replace(/-.*/, '');
      const label = LOCALE_LABELS[code] || code;

      return { code, countryCode, label };
    })
    .sort((a, b) => {
      // Sort: en first, then en-GB, then alphabetically
      if (a.code === 'en') return -1;
      if (b.code === 'en') return 1;
      if (a.code === 'en-GB') return -1;
      if (b.code === 'en-GB') return 1;
      return a.code.localeCompare(b.code);
    });

  return locales;
}

/**
 * Generate the normalizeLocale function code
 */
function generateNormalizeLocaleFunction(locales: LocaleConfig[]): string {
  const cases = locales
    .map((locale) => {
      const code = locale.code;
      const lower = code.toLowerCase();
      if (code.includes('-')) {
        // For locales with dashes like en-GB, pt-PT
        return `  if (lower === '${lower}' || lower.startsWith('${lower}')) return '${code}';`;
      } else {
        // For simple locales like en, de, etc.
        return `  if (lower === '${lower}' || lower.startsWith('${lower}-')) return '${code}';`;
      }
    })
    .join('\n');

  return `export const normalizeLocale = (input?: string | null): SupportedLocale | null => {
  if (!input) return null;
  const lower = input.toLowerCase();
${cases}
  return null;
};`;
}

/**
 * Update src/lib/i18n.tsx
 */
function updateI18nLib(locales: LocaleConfig[]) {
  const filePath = path.join(__dirname, '../src/lib/i18n.tsx');
  let content = fs.readFileSync(filePath, 'utf-8');

  // Update SUPPORTED_LOCALES array
  const localeArray = locales.map((l) => `'${l.code}'`).join(', ');
  content = content.replace(
    /export const SUPPORTED_LOCALES = \[.*?\] as const;/s,
    `export const SUPPORTED_LOCALES = [${localeArray}] as const;`
  );

  // Update normalizeLocale function
  const normalizeFunc = generateNormalizeLocaleFunction(locales);
  content = content.replace(
    /export const normalizeLocale = \(input\?: string \| null\): SupportedLocale \| null => \{[\s\S]*?\n\};/,
    normalizeFunc
  );

  fs.writeFileSync(filePath, content, 'utf-8');
  console.log(`✓ Updated src/lib/i18n.tsx`);
}

/**
 * Update src/App.tsx (flag imports and ALL_LOCALE_OPTIONS)
 */
function updateAppTsx(locales: LocaleConfig[]) {
  const filePath = path.join(__dirname, '../src/App.tsx');
  let content = fs.readFileSync(filePath, 'utf-8');

  // Update flag imports
  const flagImports = [...new Set(locales.map((l) => l.countryCode))].sort().join(', ');
  content = content.replace(
    /import \{ [A-Z, ]+ \} from 'country-flag-icons\/react\/3x2';/,
    `import { ${flagImports} } from 'country-flag-icons/react/3x2';`
  );

  // Update ALL_LOCALE_OPTIONS array
  const optionsArray = locales
    .map(
      (l) =>
        `  { value: '${l.code}', flag: ${l.countryCode}, label: '${l.label}', contributor: ${l.contributor ? `'${l.contributor}'` : "'AI'"} }`
    )
    .join(',\n');

  content = content.replace(
    /const ALL_LOCALE_OPTIONS: \{[\s\S]*?\}\[\] = \[[\s\S]*?\];/,
    `const ALL_LOCALE_OPTIONS: {
  value: SupportedLocale;
  flag: typeof US;
  label: string;
  contributor?: string;
}[] = [
${optionsArray},
];`
  );

  fs.writeFileSync(filePath, content, 'utf-8');
  console.log(`✓ Updated src/App.tsx`);
}

/**
 * Update src/components/tabs/SettingsTab.tsx
 */
function updateSettingsTab(locales: LocaleConfig[]) {
  const filePath = path.join(__dirname, '../src/components/tabs/SettingsTab.tsx');
  let content = fs.readFileSync(filePath, 'utf-8');

  // Update localeOptions array
  const optionsArray = locales
    .map((l) => `    { value: '${l.code}', label: '${l.label}' }`)
    .join(',\n');

  content = content.replace(
    /const localeOptions: \{ value: SupportedLocale; label: string \}\[\] = \[[\s\S]*?\];/,
    `const localeOptions: { value: SupportedLocale; label: string }[] = [
${optionsArray},
  ];`
  );

  fs.writeFileSync(filePath, content, 'utf-8');
  console.log(`✓ Updated src/components/tabs/SettingsTab.tsx`);
}

/**
 * Update crowdin.yml languages_mapping
 */
function updateCrowdinYml(locales: LocaleConfig[]) {
  const filePath = path.join(__dirname, '../crowdin.yml');
  let content = fs.readFileSync(filePath, 'utf-8');

  // Generate languages_mapping entries (exclude 'en' since it's the source)
  const mappingEntries = locales
    .filter((l) => l.code !== 'en')
    .map((l) => `        ${l.code}: ${l.code}`)
    .join('\n');

  // Replace both frontend and backend mappings
  content = content.replace(
    /(languages_mapping:\s+locale_with_underscore:\s*\n)([\s\S]*?)(\n\s*\n|\n\s*#)/g,
    (match, prefix, oldMapping, suffix) => {
      return `${prefix}${mappingEntries}${suffix}`;
    }
  );

  fs.writeFileSync(filePath, content, 'utf-8');
  console.log(`✓ Updated crowdin.yml`);
}

/**
 * Main execution
 */
function main() {
  console.log('🌍 Generating i18n configuration...\n');

  const locales = discoverLocales();
  console.log(`Found ${locales.length} locales: ${locales.map((l) => l.code).join(', ')}\n`);

  try {
    updateI18nLib(locales);
    updateAppTsx(locales);
    updateSettingsTab(locales);
    updateCrowdinYml(locales);

    console.log('\n✅ All i18n configuration files updated successfully!');
    console.log('\n📝 Updated files:');
    console.log('  1. src/lib/i18n.tsx');
    console.log('  2. src/App.tsx');
    console.log('  3. src/components/tabs/SettingsTab.tsx');
    console.log('  4. crowdin.yml');
  } catch (error) {
    console.error('\n❌ Error updating files:', error);
    process.exit(1);
  }
}

main();
