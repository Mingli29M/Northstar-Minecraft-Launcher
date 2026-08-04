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
import { APP_VERSION, LAUNCHER_CHANGELOG } from "../lib/launcherChangelog";
import { useI18n } from "../i18n";
import type { LauncherSettings, Locale } from "../lib/types";

export function SettingsPage() {
  const { t, locale, setLocale } = useI18n();
  const [settings, setSettings] = useState<LauncherSettings | null>(null);
  const [javas, setJavas] = useState<string[]>([]);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    api.getSettings().then(setSettings);
    api.detectJavaInstalls().then(setJavas).catch(() => setJavas([]));
  }, []);

  async function onSave(e: FormEvent) {
    e.preventDefault();
    if (!settings) return;
    const threads = Math.min(64, Math.max(4, settings.download_threads || 16));
    const saved = await api.saveSettings({ ...settings, download_threads: threads, locale });
    setSettings(saved);
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
