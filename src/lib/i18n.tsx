/* eslint-disable react-refresh/only-export-components */
import { invoke } from '@tauri-apps/api/core';
import {
  PropsWithChildren,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

type Messages = Record<string, unknown>;

export const SUPPORTED_LOCALES = ['en', 'en-GB', 'ko', 'tr', 'pt-PT', 'de', 'it', 'nl'] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

const FALLBACK_LOCALE: SupportedLocale = 'en';

export const normalizeLocale = (input?: string | null): SupportedLocale | null => {
  if (!input) return null;
  const lower = input.toLowerCase();
  if (lower === 'en-gb' || lower.startsWith('en-gb')) return 'en-GB';
  if (lower === 'en' || lower.startsWith('en-')) return 'en';
  if (lower === 'ko' || lower.startsWith('ko-')) return 'ko';
  if (lower === 'tr' || lower.startsWith('tr-')) return 'tr';
  if (lower === 'de' || lower.startsWith('de-')) return 'de';
  if (lower === 'pt-pt' || lower === 'pt' || lower.startsWith('pt-')) return 'pt-PT';
  if (lower === 'it' || lower.startsWith('it-')) return 'it';
  if (lower === 'nl' || lower.startsWith('nl-')) return 'nl';
  return null;
};

const LOCALE_MODULES = import.meta.glob('../locales/*.json', {
  eager: true,
});

const extractLocaleFromKey = (key: string): SupportedLocale | null => {
  const match =
    key.match(/\.\/?\.\/locales\/([^/]+)\.json$/) ?? key.match(/\.{2}\/locales\/([^/]+)\.json$/);
  if (!match) return null;
  const candidate = match[1];
  return SUPPORTED_LOCALES.includes(candidate as SupportedLocale)
    ? (candidate as SupportedLocale)
    : null;
};

export const AVAILABLE_LOCALES = new Set(
  Object.keys(LOCALE_MODULES)
    .map(extractLocaleFromKey)
    .filter((loc): loc is SupportedLocale => Boolean(loc))
);

const loadLocale = async (locale: SupportedLocale): Promise<Messages> => {
  const key = `../locales/${locale}.json`;
  const mod = LOCALE_MODULES[key];

  if (!mod) {
    console.warn(`[i18n] Locale '${locale}' not found in bundle`);
    return {};
  }

  const messages = (mod as { default?: Messages }).default ?? (mod as Messages);
  return messages;
};

export const detectSystemLocale = async (): Promise<SupportedLocale | null> => {
  const candidates: (string | null | undefined)[] = [];

  // Try plugin:locale common command names; fall back to navigator
  for (const command of ['plugin:locale|getLocale', 'plugin:locale|get']) {
    try {
      const result = await invoke<string>(command);
      candidates.push(result);
      break;
    } catch {
      // ignore and try next
    }
  }

  if (typeof navigator !== 'undefined') {
    candidates.push(navigator.language);
    candidates.push(...(navigator.languages ?? []));
  }

  for (const cand of candidates) {
    const normalized = normalizeLocale(cand ?? undefined);
    if (normalized) return normalized;
  }

  return null;
};

const getNested = (messages: Messages, key: string): unknown => {
  return key.split('.').reduce<unknown>((acc, part) => {
    if (acc && typeof acc === 'object' && part in acc) {
      return (acc as Record<string, unknown>)[part];
    }
    return undefined;
  }, messages);
};

const formatMessage = (
  value: unknown,
  params?: Record<string, string | number | undefined>
): string => {
  if (typeof value !== 'string') return '';
  if (!params) return value;
  return Object.entries(params).reduce((msg, [k, v]) => {
    return msg.replace(new RegExp(`{${k}}`, 'g'), String(v));
  }, value);
};

interface I18nContextValue {
  locale: SupportedLocale;
  fallbackLocale: SupportedLocale;
  availableLocales: SupportedLocale[];
  t: (key: string, params?: Record<string, string | number | undefined>) => string;
  isLoading: boolean;
  setLocale: (locale: SupportedLocale) => void;
}

const I18nContext = createContext<I18nContextValue | undefined>(undefined);

interface ProviderProps extends PropsWithChildren {
  locale?: SupportedLocale;
  fallbackLocale?: SupportedLocale;
  onLocaleChange?: (locale: SupportedLocale) => void;
}

export function I18nProvider({
  locale,
  fallbackLocale = FALLBACK_LOCALE,
  onLocaleChange,
  children,
}: ProviderProps) {
  const currentLocale = locale ?? FALLBACK_LOCALE;
  const [messages, setMessages] = useState<Messages>({});
  const [fallbackMessages, setFallbackMessages] = useState<Messages>({});
  const [isLoading, setIsLoading] = useState(true);
  const controlledLocale = useRef<SupportedLocale>(locale ?? FALLBACK_LOCALE);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      if (cancelled) return;
      setIsLoading(true);
      const [primary, fallback] = await Promise.all([
        loadLocale(currentLocale),
        fallbackLocale === currentLocale ? Promise.resolve(null) : loadLocale(fallbackLocale),
      ]);

      if (cancelled) return;
      setMessages(primary);
      if (fallback) setFallbackMessages(fallback);
      else setFallbackMessages(primary);
      setIsLoading(false);
    };

    load();

    return () => {
      cancelled = true;
    };
  }, [currentLocale, fallbackLocale]);

  const translate = useCallback(
    (key: string, params?: Record<string, string | number | undefined>) => {
      const primary = getNested(messages, key);
      const fallback = getNested(fallbackMessages, key);
      const raw = primary ?? fallback ?? key;
      return formatMessage(raw, params);
    },
    [messages, fallbackMessages]
  );

  const value = useMemo<I18nContextValue>(
    () => ({
      locale: currentLocale,
      fallbackLocale,
      availableLocales: SUPPORTED_LOCALES.slice(),
      t: translate,
      isLoading,
      setLocale: (next) => {
        if (controlledLocale.current === next) return;
        controlledLocale.current = next;
        if (onLocaleChange) {
          onLocaleChange(next);
        } else {
          console.warn('[i18n] No onLocaleChange handler provided; locale not updated');
        }
      },
    }),
    [currentLocale, fallbackLocale, isLoading, onLocaleChange, translate]
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error('useI18n must be used within an I18nProvider');
  }
  return ctx;
}

export const ensureSupportedLocale = (candidate?: string | null): SupportedLocale => {
  return normalizeLocale(candidate) ?? FALLBACK_LOCALE;
};
