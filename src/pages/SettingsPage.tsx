import type { FormEvent } from "react";
import { useEffect, useState } from "react";
import { Button } from "@astryxdesign/core/Button";
import { Banner } from "@astryxdesign/core/Banner";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { Selector } from "@astryxdesign/core/Selector";
import { VStack } from "@astryxdesign/core/VStack";
import { api } from "../lib/api";
import { notifyAppearance } from "../lib/appearance";
import { APP_VERSION, LAUNCHER_CHANGELOG } from "../lib/launcherChangelog";
import { useI18n } from "../i18n";
import type { LauncherSettings, Locale } from "../lib/types";

function patchSettings(
  current: LauncherSettings,
  patch: Partial<LauncherSettings>,
): LauncherSettings {
  const next = { ...current, ...patch };
  notifyAppearance(next);
  return next;
}

export function SettingsPage() {
  const { t, locale, setLocale } = useI18n();
  const [settings, setSettings] = useState<LauncherSettings | null>(null);
  const [javas, setJavas] = useState<string[]>([]);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    api.getSettings().then((s) => {
      setSettings(s);
      notifyAppearance(s);
    });
    api.detectJavaInstalls().then(setJavas).catch(() => setJavas([]));
  }, []);

  async function onSave(e: FormEvent) {
    e.preventDefault();
    if (!settings) return;
    const threads = Math.min(64, Math.max(4, settings.download_threads || 16));
    const saved = await api.saveSettings({ ...settings, download_threads: threads, locale });
    setSettings(saved);
    notifyAppearance(saved);
    setMsg(t("saved"));
  }

  if (!settings) return <Text color="secondary">{t("loading")}</Text>;

  return (
    <VStack gap={4} style={{ maxWidth: 560 }} className="euml-page">
      <VStack gap={2}>
        <Text type="display-3">{t("settingsTitle")}</Text>
        <Text color="secondary">{t("settingsHint")}</Text>
      </VStack>
      {msg && <Banner status="success" title={msg} />}

      <Card padding={4}>
        <form onSubmit={onSave}>
          <VStack gap={3}>
            <Selector
              label={t("language")}
              value={locale}
              onChange={(v) => setLocale(v as Locale)}
              options={[
                { value: "en", label: "English" },
                { value: "zh", label: "中文" },
                { value: "de", label: "Deutsch" },
              ]}
            />
            <Selector
              label={t("downloadSource")}
              description={t("downloadSourceHint")}
              value={settings.download_source ?? "official"}
              onChange={(v) => setSettings({ ...settings, download_source: v })}
              options={[
                { value: "official", label: t("downloadSourceOfficial") },
                { value: "bmclapi", label: t("downloadSourceBmclapi") },
              ]}
            />
            <TextInput
              label={t("downloadThreads")}
              description={t("downloadThreadsHint")}
              value={String(settings.download_threads ?? 16)}
              onChange={(v) => setSettings({ ...settings, download_threads: Number(v) || 16 })}
            />
            <TextInput
              label={t("instancesPath")}
              value={settings.instances_path ?? ""}
              onChange={(v) => setSettings({ ...settings, instances_path: v || null })}
            />
            <TextInput
              label={t("dedicatedPath")}
              value={settings.dedicated_path ?? ""}
              onChange={(v) => setSettings({ ...settings, dedicated_path: v || null })}
            />
            <TextInput
              label={t("javaPath")}
              value={settings.java_path ?? ""}
              onChange={(v) => setSettings({ ...settings, java_path: v || null })}
            />
            <TextInput
              label={t("cfKey")}
              type="password"
              value={settings.curseforge_api_key ?? ""}
              onChange={(v) => setSettings({ ...settings, curseforge_api_key: v || null })}
            />
            <Button type="submit" label={t("save")} variant="primary" />
          </VStack>
        </form>
      </Card>

      <Card padding={4}>
        <form onSubmit={onSave}>
          <VStack gap={3}>
            <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
              {t("appearanceTitle")}
            </Text>
            <Text color="secondary" type="supporting">
              {t("appearanceHint")}
            </Text>
            <div style={{ display: "flex", gap: 12, alignItems: "flex-end", flexWrap: "wrap" }}>
              <label style={{ display: "flex", flexDirection: "column", gap: 6, minWidth: 120 }}>
                <Text type="supporting" weight="semibold">
                  {t("accentColor")}
                </Text>
                <input
                  type="color"
                  value={settings.accent ?? "#1370f0"}
                  onChange={(e) =>
                    setSettings(patchSettings(settings, { accent: e.target.value }))
                  }
                  style={{ width: 48, height: 36, border: "none", padding: 0, cursor: "pointer" }}
                />
              </label>
              <div style={{ flex: 1, minWidth: 180 }}>
                <TextInput
                  label={t("accentHex")}
                  value={settings.accent ?? "#1370f0"}
                  onChange={(v) =>
                    setSettings(patchSettings(settings, { accent: v || "#1370f0" }))
                  }
                />
              </div>
            </div>
            <div style={{ display: "flex", gap: 12, alignItems: "flex-end", flexWrap: "wrap" }}>
              <label style={{ display: "flex", flexDirection: "column", gap: 6, minWidth: 120 }}>
                <Text type="supporting" weight="semibold">
                  {t("backgroundColor")}
                </Text>
                <input
                  type="color"
                  value={settings.background_color ?? "#f5f5f4"}
                  onChange={(e) =>
                    setSettings(patchSettings(settings, { background_color: e.target.value }))
                  }
                  style={{ width: 48, height: 36, border: "none", padding: 0, cursor: "pointer" }}
                />
              </label>
              <div style={{ flex: 1, minWidth: 180 }}>
                <TextInput
                  label={t("backgroundColorHex")}
                  value={settings.background_color ?? ""}
                  onChange={(v) =>
                    setSettings(patchSettings(settings, { background_color: v || null }))
                  }
                />
              </div>
            </div>
            <TextInput
              label={t("backgroundImage")}
              description={t("backgroundImageHint")}
              value={settings.background_image ?? ""}
              onChange={(v) =>
                setSettings(patchSettings(settings, { background_image: v || null }))
              }
            />
            {(settings.background_image || settings.background_color) && (
              <Button
                type="button"
                label={t("clearBackground")}
                variant="secondary"
                onClick={() =>
                  setSettings(
                    patchSettings(settings, {
                      background_image: null,
                      background_color: null,
                    }),
                  )
                }
              />
            )}
            <Selector
              label={t("fontFamily")}
              value={settings.font_family ?? "system"}
              onChange={(v) => setSettings(patchSettings(settings, { font_family: v }))}
              options={[
                { value: "system", label: t("fontSystem") },
                { value: "noto", label: t("fontNoto") },
                { value: "source", label: t("fontSource") },
                { value: "plex", label: t("fontPlex") },
              ]}
            />
            <Selector
              label={t("uiScale")}
              value={String(settings.ui_scale ?? 1)}
              onChange={(v) =>
                setSettings(patchSettings(settings, { ui_scale: Number(v) || 1 }))
              }
              options={[
                { value: "0.9", label: "90%" },
                { value: "1", label: "100%" },
                { value: "1.1", label: "110%" },
                { value: "1.25", label: "125%" },
              ]}
            />
            <Button type="submit" label={t("save")} variant="primary" />
          </VStack>
        </form>
      </Card>

      {javas.length > 0 && (
        <Card padding={3}>
          <Text weight="semibold" display="block" style={{ marginBottom: 8 }}>
            {t("detectedJava")}
          </Text>
          {javas.map((j) => (
            <Text key={j} color="secondary" type="supporting" display="block">
              {j}
            </Text>
          ))}
        </Card>
      )}

      <Card padding={4}>
        <VStack gap={3}>
          <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
            {t("aboutTitle")}
          </Text>
          <Text>
            {t("appName")} {APP_VERSION}
            {LAUNCHER_CHANGELOG[0]?.codename
              ? ` “${LAUNCHER_CHANGELOG[0].codename}”`
              : ""}
          </Text>
          <Text color="secondary" type="supporting">
            {t("aboutHint")}
          </Text>
          <Text weight="semibold" display="block">
            {t("launcherChangelog")}
          </Text>
          {LAUNCHER_CHANGELOG.map((entry) => (
            <VStack
              key={entry.version}
              gap={2}
              style={{
                marginBottom: 12,
                paddingBottom: 12,
                borderBottom: "1px solid color-mix(in srgb, var(--color-border-primary, #d6d3d1) 70%, transparent)",
              }}
            >
              <VStack gap={1}>
                <Text weight="semibold">
                  v{entry.version}
                  {entry.codename ? ` — ${entry.codename}` : ""} · {entry.date}
                </Text>
                <Text color="secondary" type="supporting" display="block">
                  {entry.summary}
                </Text>
              </VStack>
              {entry.sections.map((section) => (
                <VStack key={section.title} gap={1}>
                  <Text weight="semibold" type="supporting" display="block">
                    {section.title}
                  </Text>
                  {section.items.map((item) => (
                    <Text
                      key={item}
                      color="secondary"
                      type="supporting"
                      display="block"
                      style={{ paddingLeft: 8 }}
                    >
                      · {item}
                    </Text>
                  ))}
                </VStack>
              ))}
            </VStack>
          ))}
        </VStack>
      </Card>
    </VStack>
  );
}
