import { Effect, getCurrentWindow } from "@tauri-apps/api/window";

let lastEnabled: boolean | null = null;

/**
 * Toggle OS-level translucency so the desktop wallpaper can show through
 * the launcher window (Windows Acrylic / Mica / Blur; no-op outside Tauri).
 */
export async function syncWindowTransparency(opacity: number): Promise<void> {
  const enabled = opacity < 0.98;
  if (lastEnabled === enabled) return;
  lastEnabled = enabled;

  try {
    const win = getCurrentWindow();
    // Shadows break transparent windows on Windows.
    await win.setShadow(false).catch(() => undefined);
    if (enabled) {
      // First supported effect wins; list covers Win10/11 + macOS fallbacks.
      await win.setEffects({
        effects: [Effect.Acrylic, Effect.Blur, Effect.Mica, Effect.HudWindow],
      });
    } else {
      await win.clearEffects();
    }
  } catch {
    lastEnabled = null;
  }
}
