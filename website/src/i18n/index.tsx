import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  dictionaries,
  LOCALE_STORAGE_KEY,
  type Locale,
  type MessageKey,
} from "./messages";

type I18nValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: MessageKey, vars?: Record<string, string | number>) => string;
};

const I18nContext = createContext<I18nValue | null>(null);

function readStoredLocale(): Locale {
  try {
    const raw = localStorage.getItem(LOCALE_STORAGE_KEY);
    if (raw === "zh" || raw === "de" || raw === "en") return raw;
  } catch {
    /* ignore */
  }
  const nav = navigator.language.toLowerCase();
  if (nav.startsWith("zh")) return "zh";
  if (nav.startsWith("de")) return "de";
  return "en";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(() =>
    typeof window === "undefined" ? "en" : readStoredLocale(),
  );

  useEffect(() => {
    document.documentElement.lang = locale === "zh" ? "zh-CN" : locale;
  }, [locale]);

  const setLocale = (next: Locale) => {
    setLocaleState(next);
    try {
      localStorage.setItem(LOCALE_STORAGE_KEY, next);
    } catch {
      /* ignore */
    }
  };

  const value = useMemo<I18nValue>(
    () => ({
      locale,
      setLocale,
      t: (key, vars) => {
        let text = dictionaries[locale][key] ?? dictionaries.en[key] ?? key;
        if (vars) {
          for (const [k, v] of Object.entries(vars)) {
            text = text.replaceAll(`{${k}}`, String(v));
          }
        }
        return text;
      },
    }),
    [locale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within I18nProvider");
  return ctx;
}
