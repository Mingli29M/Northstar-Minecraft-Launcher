import type { LauncherSettings } from "./types";

type Listener = () => void;

let snapshot: LauncherSettings | null = null;
const listeners = new Set<Listener>();

export function getSettingsSnapshot(): LauncherSettings | null {
  return snapshot;
}

/** Push the latest settings for keep-alive pages and patch helpers. */
export function setSettingsSnapshot(settings: LauncherSettings) {
  snapshot = settings;
  for (const listener of listeners) listener();
}

export function subscribeSettings(listener: Listener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
