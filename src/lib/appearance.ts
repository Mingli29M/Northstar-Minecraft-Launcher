import { convertFileSrc } from "@tauri-apps/api/core";
import { cacheSet } from "./cache";
import { setSettingsSnapshot } from "./settingsStore";
import type { LauncherSettings } from "./types";
import { syncWindowTransparency } from "./windowTransparency";

const BASE_FONT_PX = 16;
const FONT_LINK_ID = "northstar-appearance-fonts";
const STYLE_ID = "northstar-appearance-vars";

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

const FONT_STYLESHEETS: Record<string, string> = {
  "source-han-sans":
    "https://fonts.googleapis.com/css2?family=Noto+Sans+SC:wght@400;500;600;700&display=swap",
  "source-han-serif":
    "https://fonts.googleapis.com/css2?family=Noto+Serif+SC:wght@400;600;700&display=swap",
  noto: "https://fonts.googleapis.com/css2?family=Noto+Sans+SC:wght@400;500;600;700&display=swap",
  source:
    "https://fonts.googleapis.com/css2?family=Source+Sans+3:wght@400;500;600;700&display=swap",
  plex: "https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&display=swap",
};

/** Last applied fingerprint — skip no-op re-applies that used to freeze the UI. */
let lastFingerprint = "";
let lastBgKey = "";

function ensureFontStylesheet(fontKey: string) {
  const href = FONT_STYLESHEETS[fontKey];
  const existing = document.getElementById(FONT_LINK_ID) as HTMLLinkElement | null;
  if (!href) {
    existing?.remove();
    return;
  }
  if (existing) {
    if (existing.getAttribute("href") !== href) existing.href = href;
    return;
  }
  const link = document.createElement("link");
  link.id = FONT_LINK_ID;
  link.rel = "stylesheet";
  link.href = href;
  document.head.appendChild(link);
}

function cssUrl(value: string): string {
  return `url("${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}")`;
}

function resolveBackgroundImage(raw: string | null | undefined): string | null {
  if (!raw?.trim()) return null;
  const value = raw.trim();
  if (
    /^https?:\/\//i.test(value) ||
    value.startsWith("data:") ||
    value.startsWith("blob:") ||
    value.startsWith("asset:") ||
    value.startsWith("http://asset.localhost") ||
    value.startsWith("https://asset.localhost")
  ) {
    return cssUrl(value);
  }
  try {
    return cssUrl(convertFileSrc(value));
  } catch {
    return null;
  }
}

function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
  const m = hex.trim().match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
  if (!m) return null;
  let h = m[1];
  if (h.length === 3) h = h.split("").map((c) => c + c).join("");
  const n = parseInt(h, 16);
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}

function rgba(hex: string, alpha: number, fallback: string): string {
  const rgb = hexToRgb(hex);
  if (!rgb) return fallback;
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
}

function setBg(
  el: HTMLElement,
  bg: string | null,
  image: string | null,
  opts?: { paintImage?: boolean },
) {
  const paintImage = opts?.paintImage ?? true;
  if (bg) {
    el.style.setProperty("background-color", bg, "important");
  } else if (image) {
    el.style.setProperty("background-color", "transparent", "important");
  } else {
    el.style.removeProperty("background-color");
  }
  if (image && paintImage) {
    // Optional color becomes a translucent wash layered above the wallpaper.
    const layered =
      bg && bg !== "transparent"
        ? `linear-gradient(${bg}, ${bg}), ${image}`
        : image;
    el.style.setProperty("background-image", layered, "important");
    el.style.setProperty("background-size", "cover", "important");
    el.style.setProperty("background-position", "center", "important");
    el.style.setProperty("background-attachment", "fixed", "important");
    el.style.setProperty("background-repeat", "no-repeat", "important");
    // Color is in the gradient layer; keep the fill clear so it does not mask the image.
    el.style.setProperty("background-color", "transparent", "important");
  } else {
    el.style.removeProperty("background-image");
    el.style.removeProperty("background-size");
    el.style.removeProperty("background-position");
    el.style.removeProperty("background-attachment");
    el.style.removeProperty("background-repeat");
  }
}

/** Paint only the shell surfaces — keep the list tiny to avoid style thrash. */
function paintShellBackground(
  bg: string | null,
  image: string | null,
  wallpaperTint: string | null,
) {
  const key = `${bg ?? ""}|${image ?? ""}|${wallpaperTint ?? ""}`;
  if (key === lastBgKey) return;
  lastBgKey = key;

  // Frosted panels need a clear shell over the translucent window backplate.
  document.documentElement.classList.add("euml-frosted-ui");
  document.documentElement.classList.toggle("euml-has-wallpaper", Boolean(image));
  document.documentElement.classList.toggle("euml-has-color-wash", Boolean(bg) && !image);

  // Base fill / wallpaper on body/#root only. Chrome stays clear so panel opacity shows.
  const layered = [document.body, document.getElementById("root")].filter(
    (el): el is HTMLElement => Boolean(el),
  );
  for (const el of layered) {
    // `bg` is already opacity-applied (color wash) from applyAppearance.
    setBg(el, image ? wallpaperTint : bg, image, { paintImage: true });
  }

  const chrome: HTMLElement[] = [];
  for (const id of ["astryx-app-shell-main", "astryx-app-shell-nav", "astryx-app-shell-aside"]) {
    const el = document.getElementById(id);
    if (el) chrome.push(el);
  }
  const theme = document.querySelector<HTMLElement>("[data-astryx-theme]");
  if (theme) chrome.push(theme);

  for (const el of chrome) {
    setBg(el, null, null, { paintImage: false });
  }
}

function syncAppearanceStylesheet(vars: Record<string, string>, fontStack: string) {
  let el = document.getElementById(STYLE_ID) as HTMLStyleElement | null;
  if (!el) {
    el = document.createElement("style");
    el.id = STYLE_ID;
    document.head.appendChild(el);
  }
  const decls = Object.entries(vars)
    .map(([k, v]) => `  ${k}: ${v};`)
    .join("\n");
  const next = `
/* Northstar appearance — unlayered so it beats @layer astryx-theme */
:root,
html[data-astryx-theme],
[data-astryx-theme] {
${decls}
  font-family: ${fontStack};
}
html, body, #root {
  font-family: ${fontStack};
}
`.trim();
  if (el.textContent !== next) el.textContent = next;
}

/** Apply appearance settings as CSS variables that actually reach Astryx. */
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
  const imageRaw = settings?.background_image?.trim() || null;
  const fontKey = settings?.font_family?.trim() || "source-han-sans";
  const scale = settings?.ui_scale ?? 1;
  const clamped = [0.9, 1, 1.1, 1.25].includes(scale) ? scale : 1;
  const opacityRaw = settings?.ui_panel_opacity ?? 0.92;
  const opacity = Math.min(1, Math.max(0.2, Number(opacityRaw) || 0.92));

  const fingerprint = [
    accent,
    bg ?? "",
    imageRaw ?? "",
    fontKey,
    String(clamped),
    String(opacity),
  ].join("\0");
  if (fingerprint === lastFingerprint) return;
  lastFingerprint = fingerprint;

  const image = resolveBackgroundImage(imageRaw);
  const hasWallpaper = Boolean(image);
  const stack = FONT_STACKS[fontKey] ?? FONT_STACKS["source-han-sans"];
  const opacityPct = `${Math.round(opacity * 100)}%`;
  // Only the window backplate is translucent. Cards / lists / menus stay solid
  // so selectors and rows remain readable over the desktop wash.
  const card = "#ffffff";
  const accentMuted = rgba(accent, 0.18, `${accent}33`);
  const baseHex = bg || "#f5f5f4";
  const windowWash = rgba(baseHex, opacity, `rgba(245,245,244,${opacity})`);
  const wallpaperTint = bg ? rgba(bg, opacity, "transparent") : "transparent";
  const mutedSolid = "#f5f5f4";

  ensureFontStylesheet(fontKey);

  const vars: Record<string, string> = {
    "--color-accent": accent,
    "--color-text-accent": accent,
    "--color-icon-accent": accent,
    "--color-border-blue": accent,
    "--color-icon-blue": accent,
    "--color-text-blue": accent,
    "--color-accent-muted": accentMuted,
    "--color-on-accent": "#ffffff",
    "--font-family-body": stack,
    "--font-family-heading": stack,
    "--euml-panel-opacity": String(opacity),
    "--euml-panel-opacity-pct": opacityPct,
    "--color-background-card": card,
    "--color-background-popover": card,
    // Shell stays clear; the window wash / wallpaper is the visible base.
    "--color-background-surface": "transparent",
    "--color-background-muted": mutedSolid,
    "--color-background-secondary": mutedSolid,
    "--color-background-selected": "#e7e5e4",
    "--color-background-body": hasWallpaper ? wallpaperTint : windowWash,
    "--euml-bg-underlay": "transparent",
    // Aliases for older CSS that used non-Astryx token names.
    "--color-background-elevated": card,
    "--color-foreground": "var(--color-text-primary, #1c1917)",
    "--color-foreground-secondary": "var(--color-text-secondary, #78716c)",
    "--color-border": "var(--color-border-primary, #d6d3d1)",
  };

  document.documentElement.classList.toggle("euml-window-translucent", opacity < 0.98);

  syncAppearanceStylesheet(vars, stack);
  document.documentElement.style.fontSize = `${BASE_FONT_PX * clamped}px`;
  paintShellBackground(
    hasWallpaper ? null : windowWash,
    image,
    hasWallpaper ? wallpaperTint : null,
  );
  void syncWindowTransparency(opacity);
}

export const APPEARANCE_EVENT = "northstar:appearance";
export const SETTINGS_EVENT = "northstar:settings";

export function notifyAppearance(
  settings: Parameters<typeof applyAppearance>[0],
) {
  applyAppearance(settings);
  window.dispatchEvent(new CustomEvent(APPEARANCE_EVENT, { detail: settings }));
}

/** Broadcast full settings so keep-alive pages (Launch) can refresh layout opts. */
export function notifySettings(settings: LauncherSettings) {
  // Keep the API cache + live snapshot ahead of disk autosave so other pages
  // (rememberPreferredInstance, Launch reload) cannot clobber pending changes.
  setSettingsSnapshot(settings);
  cacheSet("settings", settings);
  notifyAppearance(settings);
  window.dispatchEvent(new CustomEvent(SETTINGS_EVENT, { detail: settings }));
}
