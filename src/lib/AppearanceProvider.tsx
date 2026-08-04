import type { ReactNode } from "react";
import { useEffect } from "react";
import { api } from "./api";
import {
  APPEARANCE_EVENT,
  applyAppearance,
} from "./appearance";
import type { LauncherSettings } from "./types";

type AppearanceDetail = Pick<
  LauncherSettings,
  "accent" | "background_color" | "background_image" | "font_family" | "ui_scale"
>;

export function AppearanceProvider({ children }: { children: ReactNode }) {
  useEffect(() => {
    let cancelled = false;
    api
      .getSettings()
      .then((s: LauncherSettings) => {
        if (!cancelled) applyAppearance(s);
      })
      .catch(() => {
        /* browser / missing backend */
      });

    const onCustom = (e: Event) => {
      applyAppearance((e as CustomEvent<AppearanceDetail>).detail);
    };
    window.addEventListener(APPEARANCE_EVENT, onCustom);
    return () => {
      cancelled = true;
      window.removeEventListener(APPEARANCE_EVENT, onCustom);
    };
  }, []);

  return children;
}
