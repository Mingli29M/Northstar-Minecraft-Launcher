import { api } from "./api";
import { notifySettings } from "./appearance";
import { getSettingsSnapshot } from "./settingsStore";
import type { Instance, LauncherSettings } from "./types";

/** Prefer Launch's last selected instance, else first in the list. */
export function preferredInstanceId(
  instances: Instance[],
  settings?: Pick<LauncherSettings, "last_instance_id"> | null,
): string {
  const last = settings?.last_instance_id;
  if (last && instances.some((i) => i.id === last)) return last;
  return instances[0]?.id ?? "";
}

/** Load instances + settings and resolve the shared target version id. */
export async function loadPreferredInstanceId(): Promise<{
  instances: Instance[];
  instanceId: string;
  settings: LauncherSettings;
}> {
  const [instances, settings] = await Promise.all([
    api.listInstances(),
    api.getSettings(),
  ]);
  return {
    instances,
    settings,
    instanceId: preferredInstanceId(instances, settings),
  };
}

/** Persist Launch/Download/Servers selection as the shared target version. */
export async function rememberPreferredInstance(instanceId: string): Promise<void> {
  if (!instanceId) return;
  try {
    // Prefer the live snapshot so we never rewrite a pending Appearance change
    // (e.g. Start button position) from a stale getSettings cache/disk read.
    const settings = getSettingsSnapshot() ?? (await api.getSettings());
    if (settings.last_instance_id === instanceId) return;
    const next = { ...settings, last_instance_id: instanceId };
    // Preserve layout flags from the live snapshot even if a concurrent save
    // returns an older disk payload.
    const snap = getSettingsSnapshot();
    const saved = await api.saveSettings(next);
    const merged = {
      ...saved,
      launch_start_position:
        snap?.launch_start_position ?? saved.launch_start_position,
      launch_only_selected:
        snap?.launch_only_selected ?? saved.launch_only_selected,
      last_instance_id: instanceId,
    };
    notifySettings(merged);
  } catch {
    /* ignore persistence failures */
  }
}
