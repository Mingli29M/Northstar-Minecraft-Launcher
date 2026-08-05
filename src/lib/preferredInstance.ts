import { api } from "./api";
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
    const settings = await api.getSettings();
    if (settings.last_instance_id === instanceId) return;
    await api.saveSettings({ ...settings, last_instance_id: instanceId });
  } catch {
    /* ignore persistence failures */
  }
}
