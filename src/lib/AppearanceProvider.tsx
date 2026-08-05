import type { ReactNode } from "react";
import { useEffect } from "react";
import { api } from "./api";
import { APPEARANCE_EVENT, applyAppearance } from "./appearance";
import type { LauncherSettings } from "./types";

type AppearanceDetail = Pick<
  LauncherSettings,
  | "accent"
  | "background_color"
  | "background_image"
  | "font_family"
  | "ui_scale"
  | "ui_panel_opacity"
>;

export function AppearanceProvider({ children }: { children: ReactNode }) {
  useEffect(() => {
    let cancelled = false;
    let tries = 0;

    const applyFromSettings = () => {
      api
        .getSettings()
        .then((s: LauncherSettings) => {
          if (!cancelled) applyAppearance(s);
        })
        .catch(() => {
          /* browser / missing backend */
        });
    };

    applyFromSettings();

    // Theme class roots may mount after first paint — re-apply a few times.
    const boot = window.setInterval(() => {
      tries += 1;
      applyFromSettings();
      if (tries >= 5) window.clearInterval(boot);
    }, 200);

    const onCustom = (e: Event) => {
      applyAppearance((e as CustomEvent<AppearanceDetail>).detail);
    };
    window.addEventListener(APPEARANCE_EVENT, onCustom);
    return () => {
      cancelled = true;
      window.clearInterval(boot);
      window.removeEventListener(APPEARANCE_EVENT, onCustom);
    };
  }, []);

  return children;
}
