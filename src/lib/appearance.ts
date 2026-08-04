import { convertFileSrc } from "@tauri-apps/api/core";
import type { LauncherSettings } from "./types";

const BASE_FONT_PX = 16;

const FONT_STACKS: Record<string, string> = {
  system:
    'system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  noto: '"Noto Sans SC", "Noto Sans", system-ui, sans-serif',
  source: '"Source Sans 3", "Source Sans Pro", system-ui, sans-serif',
  plex: '"IBM Plex Sans", system-ui, sans-serif',
};

function resolveBackgroundImage(raw: string | null | undefined): string | null {
  if (!raw?.trim()) return null;
  const value = raw.trim();
  if (/^https?:\/\//i.test(value) || value.startsWith("data:")) {
    return `url("${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}")`;
  }
  try {
    const src = convertFileSrc(value);
    return `url("${src}")`;
  } catch {
    return `url("${value.replace(/\\/g, "/")}")`;
  }
}

/** Apply appearance settings as CSS variables on documentElement. */
export function applyAppearance(
  settings: Pick<
    LauncherSettings,
    "accent" | "background_color" | "background_image" | "font_family" | "ui_scale"
  > | null | undefined,
) {
  const root = document.documentElement;
  const accent = settings?.accent?.trim() || "#1370f0";
  root.style.setProperty("--color-accent", accent);

  const bg = settings?.background_color?.trim();
  if (bg) {
    root.style.setProperty("--color-background-body", bg);
    document.body.style.backgroundColor = bg;
  } else {
    root.style.removeProperty("--color-background-body");
    document.body.style.backgroundColor = "";
  }

  const image = resolveBackgroundImage(settings?.background_image);
  if (image) {
    document.body.style.backgroundImage = image;
    document.body.style.backgroundSize = "cover";
    document.body.style.backgroundPosition = "center";
    document.body.style.backgroundAttachment = "fixed";
  } else {
    document.body.style.backgroundImage = "";
    document.body.style.backgroundSize = "";
    document.body.style.backgroundPosition = "";
    document.body.style.backgroundAttachment = "";
  }

  const fontKey = settings?.font_family?.trim() || "system";
  const stack = FONT_STACKS[fontKey] ?? FONT_STACKS.system;
  root.style.setProperty("--font-family-body", stack);
  document.body.style.fontFamily = stack;

  const scale = settings?.ui_scale ?? 1;
  const clamped = [0.9, 1, 1.1, 1.25].includes(scale) ? scale : 1;
  root.style.fontSize = `${BASE_FONT_PX * clamped}px`;
}

export const APPEARANCE_EVENT = "northstar:appearance";

export function notifyAppearance(
  settings: Parameters<typeof applyAppearance>[0],
) {
  applyAppearance(settings);
  window.dispatchEvent(new CustomEvent(APPEARANCE_EVENT, { detail: settings }));
}
