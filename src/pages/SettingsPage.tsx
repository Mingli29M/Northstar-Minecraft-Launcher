import type { DragEvent, FormEvent } from "react";
import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@astryxdesign/core/Button";
import { Banner } from "@astryxdesign/core/Banner";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { Selector } from "@astryxdesign/core/Selector";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { api } from "../lib/api";
import { notifySettings } from "../lib/appearance";
import { APP_VERSION, LAUNCHER_CHANGELOG } from "../lib/launcherChangelog";
import { useI18n } from "../i18n";
import type { LauncherSettings, Locale } from "../lib/types";

const IMAGE_EXTS = ["png", "jpg", "jpeg", "webp", "gif", "bmp", "avif"];

function backgroundPreviewSrc(raw: string | null | undefined): string | null {
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
    return value;
  }
  try {
    return convertFileSrc(value);
  } catch {
    return null;
  }
}

/** Free-text field: allow remote/data URLs only — local files must go through Browse/import. */
function sanitizeBackgroundTextInput(value: string): string | null {
  const v = value.trim();
  if (!v) return null;
  if (/^https?:\/\//i.test(v) || v.startsWith("data:") || v.startsWith("blob:")) return v;
  // Reject absolute filesystem paths typed by hand (arbitrary local file read via asset://).
  if (v.startsWith("/") || /^[A-Za-z]:[\\/]/.test(v) || v.startsWith("\\\\")) return null;
  if (v.startsWith("asset:") || v.includes("asset.localhost")) return v;
  return null;
}

async function fileToBackgroundValue(file: File): Promise<string> {
  const withPath = file as File & { path?: string };
  if (withPath.path) {
    return api.importBackgroundImage(withPath.path);
  }
  return await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ""));
    reader.onerror = () => reject(reader.error ?? new Error("read failed"));
    reader.readAsDataURL(file);
  });
}

const LICENSE_SUMMARY = `Copyright (c) 2026 Northstar contributors. All rights reserved.

ALL RIGHTS RESERVED

This software and its source code are proprietary. No license is granted
to copy, modify, distribute, sublicense, or use this software except as
expressly permitted in writing by the copyright holders.

Permission to view this repository (if made public or shared privately)
does not constitute a grant of any rights under copyright or otherwise.

Third-party dependencies remain under their respective licenses.`;

const LICENSE_URL =
  "https://github.com/Mingli29M/Northstar-Minecraft-Launcher/blob/main/LICENSE";

type SettingsSection = "general" | "appearance" | "java" | "backups" | "about";

function patchSettings(
  current: LauncherSettings,
  patch: Partial<LauncherSettings>,
): LauncherSettings {
  const next = { ...current, ...patch };
  // Live-preview visuals + push layout flags (compact / Start position) to Launch.
  notifySettings(next);
  return next;
}

/** Persist appearance/layout tweaks without requiring a separate Save click. */
function useAutosaveSettings(settings: LauncherSettings | null) {
  const timer = useRef<number | null>(null);
  const skip = useRef(true);
  useEffect(() => {
    if (!settings) return;
    // Skip the initial load so we don't rewrite the file on mount.
    if (skip.current) {
      skip.current = false;
      return;
    }
    if (timer.current) window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      const threads = Math.min(64, Math.max(4, settings.download_threads || 16));
      const opacity = Math.min(1, Math.max(0.2, settings.ui_panel_opacity ?? 0.92));
      // Persist quietly — UI already got notifySettings from patchSettings.
      // Re-broadcasting here caused Launch to re-render on every keystroke save.
      void api
        .saveSettings({ ...settings, download_threads: threads, ui_panel_opacity: opacity })
        .catch(() => undefined);
    }, 500);
    return () => {
      if (timer.current) window.clearTimeout(timer.current);
    };
  }, [settings]);
}

function SectionNav({
  active,
  onChange,
  labels,
}: {
  active: SettingsSection;
  onChange: (s: SettingsSection) => void;
  labels: Record<SettingsSection, string>;
}) {
  const keys: SettingsSection[] = ["general", "appearance", "java", "backups", "about"];
  return (
    <div className="euml-settings-nav" role="tablist" aria-label="Settings sections">
      {keys.map((key) => (
        <button
          key={key}
          type="button"
          role="tab"
          aria-selected={active === key}
          className={`euml-settings-nav__item${active === key ? " is-active" : ""}`}
          onClick={() => onChange(key)}
        >
          {labels[key]}
        </button>
      ))}
    </div>
  );
}

export function SettingsPage() {
  const { t, locale, setLocale } = useI18n();
  const [settings, setSettings] = useState<LauncherSettings | null>(null);
  const [javas, setJavas] = useState<string[]>([]);
  const [msg, setMsg] = useState<string | null>(null);
  const [section, setSection] = useState<SettingsSection>("general");
  const [bgDragOver, setBgDragOver] = useState(false);

  useEffect(() => {
    api.getSettings().then((s) => {
      setSettings(s);
      notifySettings(s);
    });
    api.detectJavaInstalls().then(setJavas).catch(() => setJavas([]));
  }, []);

  useAutosaveSettings(settings);

  function setBackgroundImage(value: string | null) {
    if (!settings) return;
    setSettings(patchSettings(settings, { background_image: value }));
  }

  async function browseBackgroundImage() {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: "Image", extensions: IMAGE_EXTS }],
      });
      if (typeof path === "string" && path) {
        const imported = await api.importBackgroundImage(path);
        setBackgroundImage(imported);
      }
    } catch {
      /* dialog cancelled / unavailable */
    }
  }

  async function onBackgroundDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setBgDragOver(false);
    const file = e.dataTransfer.files?.[0];
    if (!file || !file.type.startsWith("image/")) return;
    try {
      setBackgroundImage(await fileToBackgroundValue(file));
    } catch {
      /* ignore bad drops */
    }
  }

  async function onSave(e: FormEvent) {
    e.preventDefault();
    if (!settings) return;
    const threads = Math.min(64, Math.max(4, settings.download_threads || 16));
    const opacity = Math.min(1, Math.max(0.2, settings.ui_panel_opacity ?? 0.92));
    const saved = await api.saveSettings({
      ...settings,
      download_threads: threads,
      ui_panel_opacity: opacity,
      locale,
    });
    setSettings(saved);
    notifySettings(saved);
    setMsg(t("saved"));
  }

  async function onResetSettings() {
    if (!window.confirm(t("resetSettingsConfirm"))) return;
    try {
      const saved = await api.resetSettings();
      setSettings(saved);
      notifySettings(saved);
      if (saved.locale) setLocale(saved.locale as Locale);
      setMsg(t("resetSettingsDone"));
    } catch (e) {
      setMsg(String(e));
    }
  }

  if (!settings) return <Text color="secondary">{t("loading")}</Text>;

  const sectionLabels: Record<SettingsSection, string> = {
    general: t("settingsSectionGeneral"),
    appearance: t("appearanceTitle"),
    java: t("settingsSectionJava"),
    backups: t("settingsSectionBackups"),
    about: t("aboutTitle"),
  };

  return (
    <VStack gap={4} style={{ maxWidth: 640 }} className="euml-page">
      <VStack gap={2}>
        <Text type="display-3">{t("settingsTitle")}</Text>
        <Text color="secondary">{t("settingsHint")}</Text>
      </VStack>
      {msg && <Banner status="success" title={msg} />}

      <SectionNav active={section} onChange={setSection} labels={sectionLabels} />

      {section === "general" && (
        <Card padding={4} className="euml-panel">
          <form onSubmit={onSave}>
            <VStack gap={3}>
              <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
                {t("settingsSectionGeneral")}
              </Text>
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
              <Selector
                label={t("reqguardDeepValidation")}
                description={t("reqguardDeepValidationHint")}
                value={settings.reqguard_deep_validation === false ? "off" : "on"}
                onChange={(v) =>
                  setSettings({ ...settings, reqguard_deep_validation: v === "on" })
                }
                options={[
                  { value: "on", label: t("reqguardDeepOn") },
                  { value: "off", label: t("reqguardDeepOff") },
                ]}
              />
              <Selector
                label={t("reqguardLocalScan")}
                description={t("reqguardLocalScanHint")}
                value={settings.reqguard_local_scan === true ? "on" : "off"}
                onChange={(v) =>
                  setSettings({ ...settings, reqguard_local_scan: v === "on" })
                }
                options={[
                  { value: "off", label: t("reqguardLocalOff") },
                  { value: "on", label: t("reqguardLocalOn") },
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
                label={t("cfKey")}
                type="password"
                value={settings.curseforge_api_key ?? ""}
                onChange={(v) => setSettings({ ...settings, curseforge_api_key: v || null })}
              />
              <Button type="submit" label={t("save")} variant="primary" />
              <VStack gap={2} style={{ marginTop: 8 }}>
                <Text weight="semibold">{t("resetSettingsTitle")}</Text>
                <Text color="secondary" type="supporting">
                  {t("resetSettingsHint")}
                </Text>
                <Button
                  type="button"
                  label={t("resetSettings")}
                  variant="destructive"
                  onClick={() => void onResetSettings()}
                />
              </VStack>
            </VStack>
          </form>
        </Card>
      )}

      {section === "appearance" && (
        <Card padding={4} className="euml-panel">
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
              <VStack gap={2}>
                <Text type="supporting" weight="semibold">
                  {t("backgroundImage")}
                </Text>
                <Text color="secondary" type="supporting">
                  {t("backgroundImageHint")}
                </Text>
                <div
                  className={`euml-bg-dropzone${bgDragOver ? " is-dragover" : ""}`}
                  role="button"
                  tabIndex={0}
                  onClick={() => void browseBackgroundImage()}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      void browseBackgroundImage();
                    }
                  }}
                  onDragEnter={(e) => {
                    e.preventDefault();
                    setBgDragOver(true);
                  }}
                  onDragOver={(e) => {
                    e.preventDefault();
                    setBgDragOver(true);
                  }}
                  onDragLeave={() => setBgDragOver(false)}
                  onDrop={(e) => void onBackgroundDrop(e)}
                >
                  <Text type="supporting">{t("backgroundImageDrop")}</Text>
                  {backgroundPreviewSrc(settings.background_image) && (
                    <img
                      className="euml-bg-dropzone__preview"
                      src={backgroundPreviewSrc(settings.background_image) ?? undefined}
                      alt=""
                    />
                  )}
                </div>
                <HStack gap={2} style={{ flexWrap: "wrap" }}>
                  <Button
                    type="button"
                    label={t("backgroundImageBrowse")}
                    variant="secondary"
                    onClick={() => void browseBackgroundImage()}
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
                </HStack>
                <TextInput
                  label={t("backgroundImage")}
                  description={t("backgroundImageHint")}
                  value={
                    settings.background_image?.startsWith("data:")
                      ? "(embedded image)"
                      : (settings.background_image ?? "")
                  }
                  onChange={(v) => {
                    if (!v) {
                      setBackgroundImage(null);
                      return;
                    }
                    // Keep displaying typed text only when it is a safe URL scheme.
                    const safe = sanitizeBackgroundTextInput(v);
                    if (safe) setBackgroundImage(safe);
                  }}
                />
              </VStack>
              <Selector
                label={t("fontFamily")}
                value={settings.font_family ?? "source-han-sans"}
                onChange={(v) => setSettings(patchSettings(settings, { font_family: v }))}
                options={[
                  { value: "source-han-sans", label: t("fontSourceHanSans") },
                  { value: "source-han-serif", label: t("fontSourceHanSerif") },
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
              <Selector
                label={t("launchCompactMode")}
                description={t("launchCompactModeHint")}
                value={settings.launch_only_selected ? "on" : "off"}
                onChange={(v) => {
                  const next = patchSettings(settings, { launch_only_selected: v === "on" });
                  setSettings(next);
                  void api.saveSettings(next).catch(() => undefined);
                }}
                options={[
                  { value: "off", label: t("launchCompactOff") },
                  { value: "on", label: t("launchCompactOn") },
                ]}
              />
              <VStack gap={1}>
                <Text type="supporting" weight="semibold">
                  {t("launchStartPosition")}
                </Text>
                <Text color="secondary" type="supporting">
                  {t("launchStartPositionHint")}
                </Text>
                <HStack gap={2} style={{ flexWrap: "wrap" }}>
                  <Button
                    type="button"
                    label={t("launchStartTop")}
                    variant={
                      (settings.launch_start_position ?? "top") === "top"
                        ? "primary"
                        : "secondary"
                    }
                    onClick={() => {
                      const next = patchSettings(settings, {
                        launch_start_position: "top",
                      });
                      setSettings(next);
                      void api.saveSettings(next).catch(() => undefined);
                    }}
                  />
                  <Button
                    type="button"
                    label={t("launchStartBottom")}
                    variant={
                      settings.launch_start_position === "bottom"
                        ? "primary"
                        : "secondary"
                    }
                    onClick={() => {
                      const next = patchSettings(settings, {
                        launch_start_position: "bottom",
                      });
                      setSettings(next);
                      void api.saveSettings(next).catch(() => undefined);
                    }}
                  />
                </HStack>
              </VStack>
              <label style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                <Text type="supporting" weight="semibold">
                  {t("panelOpacity")} ({Math.round((settings.ui_panel_opacity ?? 0.92) * 100)}%)
                </Text>
                <Text color="secondary" type="supporting">
                  {t("panelOpacityHint")}
                </Text>
                <input
                  type="range"
                  min={20}
                  max={100}
                  step={1}
                  value={Math.round((settings.ui_panel_opacity ?? 0.92) * 100)}
                  onChange={(e) =>
                    setSettings(
                      patchSettings(settings, {
                        ui_panel_opacity: Number(e.target.value) / 100,
                      }),
                    )
                  }
                />
              </label>
              <Button type="submit" label={t("save")} variant="primary" />
            </VStack>
          </form>
        </Card>
      )}

      {section === "java" && (
        <Card padding={4} className="euml-panel">
          <form onSubmit={onSave}>
            <VStack gap={3}>
              <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
                {t("settingsSectionJava")}
              </Text>
              <TextInput
                label={t("javaPath")}
                value={settings.java_path ?? ""}
                onChange={(v) => setSettings({ ...settings, java_path: v || null })}
              />
              <Button type="submit" label={t("save")} variant="primary" />
              {javas.length > 0 ? (
                <VStack gap={1}>
                  <Text weight="semibold" display="block">
                    {t("detectedJava")}
                  </Text>
                  {javas.map((j) => (
                    <Text key={j} color="secondary" type="supporting" display="block">
                      {j}
                    </Text>
                  ))}
                </VStack>
              ) : (
                <Text color="secondary" type="supporting">
                  {t("javaNoneDetected")}
                </Text>
              )}
            </VStack>
          </form>
        </Card>
      )}

      {section === "backups" && (
        <Card padding={4} className="euml-panel">
          <form onSubmit={onSave}>
            <VStack gap={3}>
              <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
                {t("settingsSectionBackups")}
              </Text>
              <Text color="secondary" type="supporting">
                {t("backupsSettingsHint")}
              </Text>
              <Selector
                label={t("autoBackupWorlds")}
                description={t("autoBackupWorldsHint")}
                value={settings.auto_backup_worlds ? "on" : "off"}
                onChange={(v) =>
                  setSettings({ ...settings, auto_backup_worlds: v === "on" })
                }
                options={[
                  { value: "off", label: t("autoBackupOff") },
                  { value: "on", label: t("autoBackupOn") },
                ]}
              />
              <TextInput
                label={t("autoBackupKeep")}
                description={t("autoBackupKeepHint")}
                value={String(settings.auto_backup_keep ?? 5)}
                onChange={(v) =>
                  setSettings({
                    ...settings,
                    auto_backup_keep: Math.min(50, Math.max(1, Number(v) || 5)),
                  })
                }
              />
              <Button type="submit" label={t("save")} variant="primary" />
            </VStack>
          </form>
        </Card>
      )}

      {section === "about" && (
        <VStack gap={4}>
          <Card padding={4} className="euml-panel">
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
            </VStack>
          </Card>

          <Card padding={4} className="euml-panel">
            <VStack gap={3}>
              <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
                {t("licenseSectionTitle")}
              </Text>
              <Banner
                status="error"
                title={t("licenseBannerTitle")}
                description={t("licenseBannerBody")}
              />
              <pre className="euml-license-block">{LICENSE_SUMMARY}</pre>
              <Text color="secondary" type="supporting">
                {t("licenseBindingNote")}
              </Text>
              <Button
                label={t("licenseViewGithub")}
                variant="secondary"
                onClick={() => {
                  window.open(LICENSE_URL, "_blank", "noopener,noreferrer");
                }}
              />
              <Banner
                status="info"
                title={t("terracottaThirdPartyTitle")}
                description={t("terracottaThirdPartyBody")}
              />
            </VStack>
          </Card>

          <Card padding={4} className="euml-panel">
            <VStack gap={3}>
              <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
                {t("launcherChangelog")}
              </Text>
              {LAUNCHER_CHANGELOG.map((entry) => (
                <VStack
                  key={entry.version}
                  gap={2}
                  style={{
                    marginBottom: 12,
                    paddingBottom: 12,
                    borderBottom:
                      "1px solid color-mix(in srgb, var(--color-border-primary, #d6d3d1) 70%, transparent)",
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
                  {entry.sections.map((sec) => (
                    <VStack key={sec.title} gap={1}>
                      <Text weight="semibold" type="supporting" display="block">
                        {sec.title}
                      </Text>
                      {sec.items.map((item) => (
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
      )}
    </VStack>
  );
}
