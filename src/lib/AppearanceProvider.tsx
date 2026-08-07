import type { ReactNode } from "react";
import { useEffect, useRef } from "react";
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
  const latest = useRef<AppearanceDetail | null>(null);

  useEffect(() => {
    let cancelled = false;
    let tries = 0;

    const apply = (s: AppearanceDetail | null | undefined) => {
      if (!s || cancelled) return;
      latest.current = s;
      applyAppearance(s);
    };

    api
      .getSettings()
      .then((s: LauncherSettings) => apply(s))
      .catch(() => {
        /* browser / missing backend */
      });

    // Theme host may mount a tick later — a few quiet retries, then stop.
    // Do NOT observe the whole DOM: paintShellBackground writes inline styles
    // and would re-enter applyAppearance forever (UI freeze / "not loading").
    const boot = window.setInterval(() => {
      tries += 1;
      if (latest.current) applyAppearance(latest.current);
      if (tries >= 4) window.clearInterval(boot);
    }, 400);

    const onCustom = (e: Event) => {
      apply((e as CustomEvent<AppearanceDetail>).detail);
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
