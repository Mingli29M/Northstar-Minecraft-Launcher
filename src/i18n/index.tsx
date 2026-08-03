import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { api } from "../lib/api";
import { dictionaries, type Locale, type MessageKey } from "./messages";

type I18nValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: MessageKey, vars?: Record<string, string | number>) => string;
};

const I18nContext = createContext<I18nValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>("en");

  useEffect(() => {
    api
      .getSettings()
      .then((s) => {
        if (s.locale === "zh" || s.locale === "en" || s.locale === "de") setLocaleState(s.locale);
      })
      .catch(() => undefined);
  }, []);

  const setLocale = (next: Locale) => {
    setLocaleState(next);
    api
      .getSettings()
      .then((s) => api.saveSettings({ ...s, locale: next }))
      .catch(() => undefined);
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
  if (!ctx) throw new Error("useI18n outside provider");
  return ctx;
}
