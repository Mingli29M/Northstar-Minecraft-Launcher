import { convertFileSrc } from "@tauri-apps/api/core";
import type { LauncherSettings } from "./types";

const BASE_FONT_PX = 16;

const FONT_STACKS: Record<string, string> = {
  system:
    'system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  "source-han-sans":
    '"Noto Sans SC", "Noto Sans", "Source Han Sans SC", "Source Han Sans", system-ui, sans-serif',
  "source-han-serif":
    '"Noto Serif SC", "Noto Serif", "Source Han Serif SC", "Source Han Serif", serif',
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

function setVar(el: HTMLElement, name: string, value: string | null) {
  if (value == null || value === "") {
    el.style.removeProperty(name);
  } else {
    // Important so we win over Astryx/theme-neutral :root / .xj0fimd token sheets.
    el.style.setProperty(name, value, "important");
  }
}

/** Theme scopes that define Astryx CSS variables (must all be updated). */
function themeRoots(): HTMLElement[] {
  const roots: HTMLElement[] = [document.documentElement];
  const appRoot = document.getElementById("root");
  if (appRoot) roots.push(appRoot);
  document.querySelectorAll<HTMLElement>(".xj0fimd").forEach((el) => {
    if (!roots.includes(el)) roots.push(el);
  });
  return roots;
}

function paintShellBackground(bg: string | null, image: string | null) {
  const targets: HTMLElement[] = [document.body];
  const appRoot = document.getElementById("root");
  if (appRoot) targets.push(appRoot);
  const shellMain = document.getElementById("astryx-app-shell-main");
  if (shellMain) targets.push(shellMain);
  shellMain?.parentElement && targets.push(shellMain.parentElement);

  for (const el of targets) {
    if (bg) {
      el.style.setProperty("background-color", bg, "important");
    } else {
      el.style.removeProperty("background-color");
    }
    if (image) {
      el.style.setProperty("background-image", image, "important");
      el.style.setProperty("background-size", "cover", "important");
      el.style.setProperty("background-position", "center", "important");
      el.style.setProperty("background-attachment", "fixed", "important");
      el.style.setProperty("background-repeat", "no-repeat", "important");
    } else {
      el.style.removeProperty("background-image");
      el.style.removeProperty("background-size");
      el.style.removeProperty("background-position");
      el.style.removeProperty("background-attachment");
      el.style.removeProperty("background-repeat");
    }
  }
}

/** Apply appearance settings as CSS variables on all Astryx theme roots. */
export function applyAppearance(
  settings: Pick<
    LauncherSettings,
    | "accent"
    | "background_color"
    | "background_image"
    | "font_family"
    | "ui_scale"
    | "ui_panel_opacity"
  > | null | undefined,
) {
  const accent = settings?.accent?.trim() || "#1370f0";
  const bg = settings?.background_color?.trim() || null;
  const image = resolveBackgroundImage(settings?.background_image);
  const fontKey = settings?.font_family?.trim() || "source-han-sans";
  const stack = FONT_STACKS[fontKey] ?? FONT_STACKS["source-han-sans"];
  const scale = settings?.ui_scale ?? 1;
  const clamped = [0.9, 1, 1.1, 1.25].includes(scale) ? scale : 1;
  const opacityRaw = settings?.ui_panel_opacity ?? 0.92;
  const opacity = Math.min(1, Math.max(0.55, Number(opacityRaw) || 0.92));
  const opacityPct = `${Math.round(opacity * 100)}%`;

  for (const root of themeRoots()) {
    setVar(root, "--color-accent", accent);
    setVar(root, "--color-text-accent", accent);
    setVar(root, "--color-icon-accent", accent);
    setVar(root, "--color-border-blue", accent);
    setVar(root, "--color-icon-blue", accent);
    setVar(root, "--font-family-body", stack);
    setVar(root, "--font-family-heading", stack);
    setVar(root, "--euml-panel-opacity", String(opacity));
    setVar(root, "--euml-panel-opacity-pct", opacityPct);

    if (bg) {
      setVar(root, "--color-background-body", bg);
      // Soften surface/card so wallpaper can show through when opacity < 1
      setVar(root, "--color-background-surface", bg);
    } else {
      setVar(root, "--color-background-body", null);
      setVar(root, "--color-background-surface", null);
    }
  }

  document.body.style.fontFamily = stack;
  document.documentElement.style.fontSize = `${BASE_FONT_PX * clamped}px`;
  paintShellBackground(bg, image);
}

export const APPEARANCE_EVENT = "northstar:appearance";

export function notifyAppearance(
  settings: Parameters<typeof applyAppearance>[0],
) {
  applyAppearance(settings);
  window.dispatchEvent(new CustomEvent(APPEARANCE_EVENT, { detail: settings }));
}
