import type { FormEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { Tab, TabList } from "@astryxdesign/core/TabList";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { Selector } from "@astryxdesign/core/Selector";
import { api } from "../lib/api";
import { loaderIconSrc } from "../lib/avatars";
import { useI18n } from "../i18n";
import type {
  ContentItem,
  Instance,
  InstanceFolder,
  LogLine,
  LitematicaInfo,
  ModEntry,
  ParsedConfig,
  ReqScanResult,
  WorldBackup,
  WorldInfo,
  WorldSettings,
} from "../lib/types";
import { normalizeMcVersion } from "../lib/mcVersion";
import { DismissibleBanner } from "../components/DismissibleBanner";
import { FavoriteButton } from "../components/FavoriteButton";
import { useFavorites } from "../lib/favorites";
import { favoriteId } from "../lib/types";

function versionIdFromPath(pathname: string): string | undefined {
  const m = pathname.match(/^\/versions\/([^/]+)/);
  return m?.[1];
}

/** Filter key: "all" | "root" | "favorites" | folder uuid */
type FolderFilter = "all" | "root" | "favorites" | string;

export function VersionsPage() {
  const { t } = useI18n();
  const { isFavorite } = useFavorites();
  const { pathname } = useLocation();
  const id = versionIdFromPath(pathname);
  const navigate = useNavigate();
  const [instances, setInstances] = useState<Instance[]>([]);
  const [folders, setFolders] = useState<InstanceFolder[]>([]);
  const [folderFilter, setFolderFilter] = useState<FolderFilter>("all");
  const [selected, setSelected] = useState<Instance | null>(null);
  const [tab, setTab] = useState("mods");
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [mods, setMods] = useState<ModEntry[]>([]);
  const [content, setContent] = useState<ContentItem[]>([]);
  const [worldsDetailed, setWorldsDetailed] = useState<WorldInfo[]>([]);
  const [expandedWorld, setExpandedWorld] = useState<string | null>(null);
  const [worldBackups, setWorldBackups] = useState<WorldBackup[]>([]);
  const [litematicaInfo, setLitematicaInfo] = useState<LitematicaInfo | null>(null);
  const [datapacks, setDatapacks] = useState<ContentItem[]>([]);
  const [worldForDp, setWorldForDp] = useState("");
  const [scan, setScan] = useState<ReqScanResult | null>(null);
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [creating, setCreating] = useState(false);
  const [creatingFolder, setCreatingFolder] = useState(false);
  const [folderName, setFolderName] = useState("");
  const [form, setForm] = useState({ name: "", gameVersion: "1.21.1", loader: "fabric", memoryMb: 4096 });
  const [draft, setDraft] = useState<Instance | null>(null);
  const [worldSettings, setWorldSettings] = useState<WorldSettings | null>(null);
  const [editingWorld, setEditingWorld] = useState<string | null>(null);
  const [configFiles, setConfigFiles] = useState<string[]>([]);
  const [configPath, setConfigPath] = useState("");
  const [configText, setConfigText] = useState("");
  const [configDirty, setConfigDirty] = useState(false);
  const [parsedConfig, setParsedConfig] = useState<ParsedConfig | null>(null);
  const [showRawConfig, setShowRawConfig] = useState(false);
  const [contentQuery, setContentQuery] = useState("");

  const refresh = useCallback(async () => {
    const [list, folderList] = await Promise.all([api.listInstances(), api.listFolders()]);
    setInstances(list);
    setFolders(folderList);
    const sel = id ? list.find((i) => i.id === id) ?? null : list[0] ?? null;
    setSelected(sel);
    setDraft(sel);
  }, [id]);

  useEffect(() => {
    let cancelled = false;
    refresh().catch((e) => {
      if (!cancelled) setError(String(e));
    });
    return () => {
      cancelled = true;
    };
  }, [refresh]);

  useEffect(() => {
    if (!selected) return;
    let cancelled = false;
    const run = async () => {
      try {
        if (tab === "mods") {
          const m = await api.listInstanceMods(selected.id);
          if (!cancelled) setMods(m);
        } else if (tab === "worlds") {
          const [worlds, litematica] = await Promise.all([
            api.listWorldsDetailed(selected.id),
            api.detectLitematica(selected.id),
          ]);
          if (!cancelled) {
            setWorldsDetailed(worlds);
            setLitematicaInfo(litematica);
            setContent(
              worlds.map((w) => ({
                name: w.name,
                path: w.path,
                kind: "saves",
                icon_path: w.icon_path,
              })),
            );
          }
        } else if (tab === "resourcepacks") {
          const c = await api.listContent(selected.id, "resourcepacks");
          if (!cancelled) setContent(c);
        } else if (tab === "shaders") {
          const c = await api.listContent(selected.id, "shaderpacks");
          if (!cancelled) setContent(c);
        } else if (tab === "datapacks") {
          const worlds = await api.listWorlds(selected.id);
          if (cancelled) return;
          setContent(worlds);
          const world = worldForDp || worlds[0]?.name || "";
          if (world && world !== worldForDp) setWorldForDp(world);
          if (world) {
            const d = await api.listDatapacks(selected.id, world);
            if (!cancelled) setDatapacks(d);
          } else {
            setDatapacks([]);
          }
        } else if (tab === "configs") {
          const files = await api.listInstanceConfigs(selected.id);
          if (!cancelled) {
            setConfigFiles(files);
            if (files[0] && !configPath) {
              setConfigPath(files[0]);
            }
          }
        } else if (tab === "reqguard") {
          const s = await api.reqguardScan(selected.id);
          if (!cancelled) setScan(s);
        } else if (tab === "logs") {
          const l = await api.readLogs(selected.id);
          if (!cancelled) setLogs(l);
        }
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    };
    const timer = window.setTimeout(run, 0);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [selected, tab, worldForDp]);

  useEffect(() => {
    if (!selected || !expandedWorld) {
      setWorldBackups([]);
      return;
    }
    let cancelled = false;
    api
      .listWorldBackups(selected.id, expandedWorld)
      .then((backups) => {
        if (!cancelled) setWorldBackups(backups);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [selected, expandedWorld, worldsDetailed]);

  useEffect(() => {
    if (!selected || tab !== "configs" || !configPath) return;
    let cancelled = false;
    api
      .readInstanceTextFile(selected.id, configPath)
      .then(async (text) => {
        if (cancelled) return;
        setConfigText(text);
        setConfigDirty(false);
        const parsed = await api.parseConfigFile(configPath, text);
        if (!cancelled) setParsedConfig(parsed);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [selected, tab, configPath]);

  useEffect(() => {
    if (!selected || !editingWorld) {
      setWorldSettings(null);
      return;
    }
    let cancelled = false;
    api
      .getWorldSettings(selected.id, editingWorld)
      .then((s) => {
        if (!cancelled) setWorldSettings(s);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [selected, editingWorld]);

  const filteredInstances = useMemo(() => {
    if (folderFilter === "all") return instances;
    if (folderFilter === "favorites") {
      return instances.filter((i) => isFavorite(favoriteId("instance", i.id)));
    }
    if (folderFilter === "root") return instances.filter((i) => !i.folder);
    return instances.filter((i) => i.folder === folderFilter);
  }, [instances, folderFilter, isFavorite]);

  const q = contentQuery.trim().toLowerCase();
  const filteredMods = useMemo(
    () => (q ? mods.filter((m) => m.file_name.toLowerCase().includes(q)) : mods),
    [mods, q],
  );
  const filteredContent = useMemo(
    () => (q ? content.filter((c) => c.name.toLowerCase().includes(q)) : content),
    [content, q],
  );
  const filteredWorldsDetailed = useMemo(
    () => (q ? worldsDetailed.filter((w) => w.name.toLowerCase().includes(q)) : worldsDetailed),
    [worldsDetailed, q],
  );
  const filteredDatapacks = useMemo(
    () => (q ? datapacks.filter((d) => d.name.toLowerCase().includes(q)) : datapacks),
    [datapacks, q],
  );
  const filteredConfigFiles = useMemo(
    () => (q ? configFiles.filter((f) => f.toLowerCase().includes(q)) : configFiles),
    [configFiles, q],
  );

  const instanceRows = useMemo(
    () =>
      filteredInstances.map((inst) => (
        <div
          key={inst.id}
          className={`euml-list-row${selected?.id === inst.id ? " is-selected" : ""}`}
        >
          <Link
            to={`/versions/${inst.id}`}
            style={{
              textDecoration: "none",
              color: "inherit",
              display: "flex",
              alignItems: "center",
              gap: 12,
              flex: 1,
              minWidth: 0,
            }}
          >
            {inst.icon_path ? (
              <img src={inst.icon_path} alt="" className="euml-avatar" />
            ) : (
              <img src={loaderIconSrc(inst.loader)} alt={inst.loader} className="euml-avatar" />
            )}
            <div style={{ flex: 1, minWidth: 0 }}>
              <span className="euml-list-row__title">{inst.name}</span>
              <span className="euml-list-row__meta">
                {inst.game_version} · {inst.loader}
              </span>
            </div>
          </Link>
          <FavoriteButton
            kind="instance"
            itemKey={inst.id}
            label={inst.name}
            subtitle={`${inst.game_version} · ${inst.loader}`}
            iconUrl={inst.icon_path}
          />
        </div>
      )),
    [filteredInstances, selected?.id],
  );

  const activeFolder =
    folderFilter !== "all" && folderFilter !== "root" && folderFilter !== "favorites"
      ? folderFilter
      : null;

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    const inst = await api.createInstance({
      name: form.name || "New",
      gameVersion: form.gameVersion,
      loader: form.loader,
      memoryMb: form.memoryMb,
      folder: activeFolder,
    });
    if (form.loader !== "vanilla") await api.installLoader(inst.id);
    setCreating(false);
    navigate(`/versions/${inst.id}`);
    await refresh();
  }

  async function onCreateFolder(e: FormEvent) {
    e.preventDefault();
    try {
      const folder = await api.createFolder(folderName);
      setFolders(await api.listFolders());
      setFolderFilter(folder.id);
      setFolderName("");
      setCreatingFolder(false);
      setStatus(t("folderCreated"));
    } catch (err) {
      setError(String(err));
    }
  }

  async function onImportFolder() {
    const path = await open({ directory: true, title: t("importFolder") });
    if (!path || Array.isArray(path)) return;
    setError(null);
    setStatus(t("loading"));
    try {
      const list = await api.importInstanceFolder(path, activeFolder);
      setStatus(t("importedCount", { count: list.length }));
      await refresh();
      if (list[0]) navigate(`/versions/${list[0].id}`);
    } catch (err) {
      setStatus(null);
      setError(String(err));
    }
  }

  async function pickAndInstall(kind: string, directory = false) {
    if (!selected) return;
    const path = await open(
      directory
        ? { directory: true }
        : { filters: [{ name: "zip", extensions: ["zip"] }], multiple: false },
    );
    if (!path || Array.isArray(path)) return;
    setContent(await api.installContentZip(selected.id, kind, path));
  }

  function contentList(kind: string) {
    return (
      <Card padding={0} className="euml-fade-in">
        <HStack gap={2} style={{ padding: 12, borderBottom: "1px solid var(--color-border)" }} align="end">
          <div style={{ flex: 1, minWidth: 140 }}>
            <TextInput label={t("searchInstalled")} value={contentQuery} onChange={setContentQuery} />
          </div>
          {kind === "saves" ? (
            <Button
              size="sm"
              label={t("importSave")}
              onClick={async () => {
                if (!selected) return;
                const path = await open({ directory: true });
                if (!path || Array.isArray(path)) return;
                setContent(await api.importSave(selected.id, path));
              }}
            />
          ) : (
            <>
              <Button size="sm" label={t("importZip")} onClick={() => pickAndInstall(kind)} />
              <Button size="sm" label={t("importFile")} variant="secondary" onClick={() => pickAndInstall(kind, true)} />
            </>
          )}
        </HStack>
        {filteredContent.map((item) => (
          <HStack key={item.path} justify="between" align="center" className="euml-list-row" gap={3}>
            {item.icon_path ? (
              <img src={item.icon_path} alt="" className="euml-avatar" />
            ) : (
              <div className="euml-avatar" style={{ display: "grid", placeItems: "center", fontSize: 11 }}>
                {item.name.slice(0, 2).toUpperCase()}
              </div>
            )}
            <Text style={{ flex: 1, minWidth: 0 }}>{item.name}</Text>
            <HStack gap={2}>
              {kind === "saves" && (
                <Button size="sm" label={t("worldSettings")} onClick={() => setEditingWorld(item.name)} />
              )}
              <Button size="sm" label={t("openItem")} variant="secondary" onClick={() => api.openContentItem(item.path)} />
              <Button
                size="sm"
                label={t("delete")}
                variant="destructive"
                onClick={async () => {
                  if (!selected) return;
                  setContent(await api.deleteContent(selected.id, kind, item.name));
                }}
              />
            </HStack>
          </HStack>
        ))}
        {filteredContent.length === 0 && (
          <div style={{ padding: 16 }}>
            <Text color="secondary">{t("none")}</Text>
          </div>
        )}
      </Card>
    );
  }

  return (
    <HStack gap={5} align="stretch" style={{ minHeight: "100%" }} className="euml-page">
      <Card padding={0} style={{ width: 300, overflow: "hidden", display: "flex", flexDirection: "column" }}>
        <div className="euml-sidebar-actions">
          <Text weight="semibold">{t("versionsTitle")}</Text>
          <div className="euml-toolbar">
            <Button label={t("newFolder")} size="sm" variant="secondary" onClick={() => setCreatingFolder(true)} />
            <Button label={t("newVersion")} size="sm" onClick={() => setCreating(true)} />
            <Button size="sm" label={t("importFolder")} variant="secondary" onClick={onImportFolder} />
          </div>
          <Text color="secondary" type="supporting" className="euml-hint">
            {t("importFolderHint")}
          </Text>
        </div>

        <div style={{ borderBottom: "1px solid var(--color-border)", maxHeight: 220, overflow: "auto" }}>
          <div className="euml-folder-label">{t("folders")}</div>
          <button
            type="button"
            className={`euml-list-row${folderFilter === "all" ? " is-selected" : ""}`}
            onClick={() => setFolderFilter("all")}
          >
            <div style={{ flex: 1 }}>
              <span className="euml-list-row__title">{t("allFolders")}</span>
            </div>
            <span className="euml-list-row__meta">{instances.length}</span>
          </button>
          <button
            type="button"
            className={`euml-list-row${folderFilter === "favorites" ? " is-selected" : ""}`}
            onClick={() => setFolderFilter("favorites")}
          >
            <div style={{ flex: 1 }}>
              <span className="euml-list-row__title">{t("favorites")}</span>
            </div>
            <span className="euml-list-row__meta">
              {instances.filter((i) => isFavorite(favoriteId("instance", i.id))).length}
            </span>
          </button>
          <button
            type="button"
            className={`euml-list-row${folderFilter === "root" ? " is-selected" : ""}`}
            onClick={() => setFolderFilter("root")}
          >
            <div style={{ flex: 1 }}>
              <span className="euml-list-row__title">{t("uncategorized")}</span>
            </div>
            <span className="euml-list-row__meta">{instances.filter((i) => !i.folder).length}</span>
          </button>
          {folders.map((f) => (
            <button
              key={f.id}
              type="button"
              className={`euml-list-row${folderFilter === f.id ? " is-selected" : ""}`}
              onClick={() => setFolderFilter(f.id)}
            >
              <div style={{ flex: 1, minWidth: 0 }}>
                <span className="euml-list-row__title">{f.name}</span>
                <span className="euml-list-row__meta" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {f.path}
                </span>
              </div>
              <span className="euml-list-row__meta">{instances.filter((i) => i.folder === f.id).length}</span>
            </button>
          ))}
          {activeFolder && (
            <div className="euml-toolbar" style={{ padding: "12px 16px 16px" }}>
              <Button size="sm" label={t("openDiskFolder")} variant="secondary" onClick={() => api.openDiskFolder(activeFolder)} />
              <Button
                size="sm"
                label={t("deleteFolder")}
                variant="destructive"
                onClick={async () => {
                  setFolders(await api.deleteFolder(activeFolder));
                  setFolderFilter("all");
                  await refresh();
                }}
              />
            </div>
          )}
        </div>

        <div style={{ flex: 1, overflow: "auto" }}>{instanceRows}</div>
      </Card>

      <VStack gap={4} style={{ flex: 1, minWidth: 0 }}>
        {error && <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />}
        {status && <DismissibleBanner status="info" title={status} onDismiss={() => setStatus(null)} />}
        {creatingFolder && (
          <Card padding={5} className="euml-fade-in">
            <form onSubmit={onCreateFolder}>
              <VStack gap={4}>
                <TextInput label={t("folderName")} value={folderName} onChange={setFolderName} />
                <div className="euml-toolbar">
                  <Button type="submit" label={t("createFolder")} variant="primary" />
                  <Button label={t("cancel")} variant="secondary" onClick={() => setCreatingFolder(false)} />
                </div>
              </VStack>
            </form>
          </Card>
        )}
        {creating && (
          <Card padding={5} className="euml-fade-in">
            <form onSubmit={onCreate}>
              <VStack gap={4}>
                <TextInput label={t("name")} value={form.name} onChange={(v) => setForm({ ...form, name: v })} />
                <TextInput
                  label={t("gameVersion")}
                  value={form.gameVersion}
                  onChange={(v) => setForm({ ...form, gameVersion: v })}
                />
                <Selector
                  label={t("loader")}
                  value={form.loader}
                  onChange={(v) => setForm({ ...form, loader: v })}
                  options={["vanilla", "fabric", "quilt", "forge", "neoforge"].map((l) => ({
                    value: l,
                    label: l,
                  }))}
                />
                <Selector
                  label={t("folders")}
                  value={activeFolder ?? "root"}
                  onChange={(v) => setFolderFilter(v === "root" ? "root" : v)}
                  options={[
                    { value: "root", label: t("uncategorized") },
                    ...folders.map((f) => ({ value: f.id, label: f.name })),
                  ]}
                />
                <div className="euml-toolbar">
                  <Button type="submit" label={t("create")} variant="primary" />
                  <Button label={t("cancel")} variant="secondary" onClick={() => setCreating(false)} />
                </div>
              </VStack>
            </form>
          </Card>
        )}

        {!selected ? (
          <Text color="secondary">{t("none")}</Text>
        ) : (
          <>
            <div className="euml-section euml-fade-in">
              <HStack justify="between" align="start" gap={4}>
                <VStack gap={1}>
                  <Text type="display-3">{selected.name}</Text>
                  <Text color="secondary">
                    {selected.game_version} · {selected.loader}
                    {selected.folder ? ` · ${selected.folder}` : ""}
                  </Text>
                </VStack>
              </HStack>
              <div className="euml-toolbar">
                <Selector
                  label={t("moveToFolder")}
                  value={selected.folder ?? "root"}
                  onChange={async (v) => {
                    const moved = await api.moveInstance(selected.id, v === "root" ? null : v);
                    setSelected(moved);
                    setDraft(moved);
                    setInstances(await api.listInstances());
                    setFolders(await api.listFolders());
                  }}
                  options={[
                    { value: "root", label: t("uncategorized") },
                    ...folders.map((f) => ({ value: f.id, label: f.name })),
                  ]}
                />
                <Button label={t("openFolder")} variant="secondary" onClick={() => api.openInstanceFolder(selected.id)} />
                <Button
                  label={t("delete")}
                  variant="destructive"
                  onClick={async () => {
                    await api.deleteInstance(selected.id);
                    navigate("/versions");
                    await refresh();
                  }}
                />
              </div>
            </div>

            <TabList
              value={tab}
              onChange={(v) => {
                setTab(v);
                setContentQuery("");
              }}
            >
              <Tab value="mods" label={t("mods")} />
              <Tab value="configs" label={t("configs")} />
              <Tab value="worlds" label={t("worlds")} />
              <Tab value="resourcepacks" label={t("resourcepacks")} />
              <Tab value="shaders" label={t("shaders")} />
              <Tab value="datapacks" label={t("datapacks")} />
              <Tab value="reqguard" label="ReqGuard" />
              <Tab value="logs" label={t("logs")} />
              <Tab value="advanced" label={t("advanced")} />
            </TabList>

            {tab === "mods" && (
              <Card padding={0} className="euml-fade-in">
                <HStack gap={2} style={{ padding: 12, borderBottom: "1px solid var(--color-border)" }} align="end">
                  <div style={{ flex: 1, minWidth: 140 }}>
                    <TextInput label={t("searchInstalled")} value={contentQuery} onChange={setContentQuery} />
                  </div>
                  <Button
                    size="sm"
                    label={t("updateMods")}
                    onClick={async () => {
                      setStatus(t("updatingMods"));
                      try {
                        const results = await api.updateInstanceMods(selected.id);
                        const n = results.filter((r) => r.updated).length;
                        setStatus(t("modsUpdated", { count: n }));
                        setMods(await api.listInstanceMods(selected.id));
                      } catch (e) {
                        setError(String(e));
                      }
                    }}
                  />
                </HStack>
                {filteredMods.map((m) => (
                  <HStack
                    key={m.file_name}
                    justify="between"
                    align="center"
                    gap={3}
                    style={{ padding: "10px 14px", borderBottom: "1px solid var(--color-border)" }}
                  >
                    {m.icon_path ? (
                      <img src={m.icon_path} alt="" className="euml-avatar" />
                    ) : (
                      <div className="euml-avatar" style={{ display: "grid", placeItems: "center", fontSize: 11 }}>
                        {m.file_name.slice(0, 2).toUpperCase()}
                      </div>
                    )}
                    <Text style={{ flex: 1, textDecoration: m.enabled ? undefined : "line-through" }}>{m.file_name}</Text>
                    <HStack gap={2}>
                      <Button
                        size="sm"
                        label={t("modSettings")}
                        onClick={async () => {
                          const files = await api.configsForMod(selected.id, m.file_name);
                          setTab("configs");
                          const all = await api.listInstanceConfigs(selected.id);
                          setConfigFiles(all);
                          if (files[0]) setConfigPath(files[0]);
                          else if (all[0]) setConfigPath(all[0]);
                          else setStatus(t("noConfigFiles"));
                        }}
                      />
                      <Button
                        size="sm"
                        label={m.enabled ? t("disable") : t("enable")}
                        onClick={async () => setMods(await api.setModEnabled(selected.id, m.file_name, !m.enabled))}
                      />
                      <Button
                        size="sm"
                        variant="destructive"
                        label={t("delete")}
                        onClick={async () => setMods(await api.uninstallMod(selected.id, m.file_name))}
                      />
                    </HStack>
                  </HStack>
                ))}
                {filteredMods.length === 0 && (
                  <div style={{ padding: 16 }}>
                    <Text color="secondary">{t("none")}</Text>
                  </div>
                )}
              </Card>
            )}

            {tab === "configs" && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <TextInput
                    label={t("searchInstalled")}
                    value={contentQuery}
                    onChange={setContentQuery}
                  />
                  <Selector
                    label={t("configFile")}
                    value={configPath}
                    onChange={(v) => {
                      setConfigPath(v);
                      setConfigDirty(false);
                      setParsedConfig(null);
                    }}
                    options={[
                      { value: "", label: "—" },
                      ...filteredConfigFiles.map((f) => ({ value: f, label: f })),
                    ]}
                  />
                  {configPath && parsedConfig && parsedConfig.fields.length > 0 && !showRawConfig && (
                    <VStack gap={3} className="euml-settings-form">
                      <Text type="supporting" color="secondary">
                        {t("configFormat")}: {parsedConfig.format}
                      </Text>
                      {(() => {
                        const groups = new Map<string, { field: (typeof parsedConfig.fields)[0]; idx: number }[]>();
                        parsedConfig.fields.forEach((f, idx) => {
                          const section = f.section || t("configSectionGeneral");
                          const list = groups.get(section) ?? [];
                          list.push({ field: f, idx });
                          groups.set(section, list);
                        });
                        return [...groups.entries()].map(([section, items]) => (
                          <VStack key={section} gap={2} className="euml-config-section">
                            <Text weight="semibold" className="euml-config-section__title">
                              {section}
                            </Text>
                            {items.map(({ field: f, idx }) => (
                              <HStack key={f.key} gap={2} align="center">
                                <VStack gap={0} style={{ width: 200, flexShrink: 0 }}>
                                  <Text type="supporting" weight="semibold">
                                    {f.label || f.key}
                                  </Text>
                                  {f.label && f.label !== f.key && (
                                    <Text color="secondary" type="supporting" style={{ fontSize: 11 }}>
                                      {f.key}
                                    </Text>
                                  )}
                                </VStack>
                                {f.value_type === "bool" ? (
                                  <label>
                                    <input
                                      type="checkbox"
                                      checked={f.value === "true"}
                                      onChange={(e) => {
                                        const fields = [...parsedConfig.fields];
                                        fields[idx] = { ...f, value: e.target.checked ? "true" : "false" };
                                        setParsedConfig({ ...parsedConfig, fields });
                                        setConfigDirty(true);
                                      }}
                                    />
                                  </label>
                                ) : (
                                  <TextInput
                                    label=""
                                    value={f.value}
                                    onChange={(v) => {
                                      const fields = [...parsedConfig.fields];
                                      fields[idx] = { ...f, value: v };
                                      setParsedConfig({ ...parsedConfig, fields });
                                      setConfigDirty(true);
                                    }}
                                  />
                                )}
                              </HStack>
                            ))}
                          </VStack>
                        ));
                      })()}
                    </VStack>
                  )}
                  {(showRawConfig || (parsedConfig && parsedConfig.fields.length === 0)) && configPath && (
                    <textarea
                      className="euml-config-editor"
                      value={configText}
                      onChange={(e) => {
                        setConfigText(e.target.value);
                        setConfigDirty(true);
                      }}
                      rows={18}
                    />
                  )}
                  {configPath && (
                    <HStack gap={2}>
                      {parsedConfig && parsedConfig.fields.length > 0 && (
                        <Button
                          size="sm"
                          label={showRawConfig ? t("formView") : t("rawView")}
                          onClick={() => setShowRawConfig(!showRawConfig)}
                        />
                      )}
                      <Button
                        label={t("save")}
                        variant="primary"
                        isDisabled={!configDirty}
                        onClick={async () => {
                          let contents = configText;
                          if (parsedConfig && !showRawConfig && parsedConfig.fields.length > 0) {
                            contents = await api.applyConfigFields(configPath, configText, parsedConfig.fields);
                          }
                          await api.writeInstanceTextFile(selected.id, configPath, contents);
                          setConfigText(contents);
                          setParsedConfig(await api.parseConfigFile(configPath, contents));
                          setConfigDirty(false);
                          setStatus(t("configSaved"));
                        }}
                      />
                    </HStack>
                  )}
                  {filteredConfigFiles.length === 0 && (
                    <Text color="secondary">{configFiles.length === 0 ? t("noConfigFiles") : t("none")}</Text>
                  )}
                </VStack>
              </Card>
            )}

            {tab === "worlds" && (
              <VStack gap={3}>
                {litematicaInfo?.present && (
                  <DismissibleBanner
                    status="info"
                    title={`${t("litematicaDetected")} — ${t("litematicaSchematicsPath", { path: litematicaInfo.schematics_path })}`}
                    onDismiss={() => setLitematicaInfo({ ...litematicaInfo, present: false })}
                  />
                )}
                <Card padding={0} className="euml-fade-in">
                  <HStack gap={2} style={{ padding: 12, borderBottom: "1px solid var(--color-border)" }} align="end">
                    <div style={{ flex: 1, minWidth: 140 }}>
                      <TextInput label={t("searchInstalled")} value={contentQuery} onChange={setContentQuery} />
                    </div>
                    <Button
                      size="sm"
                      label={t("importSave")}
                      onClick={async () => {
                        if (!selected) return;
                        const path = await open({ directory: true });
                        if (!path || Array.isArray(path)) return;
                        const imported = await api.importSave(selected.id, path);
                        setContent(imported);
                        setWorldsDetailed(await api.listWorldsDetailed(selected.id));
                      }}
                    />
                  </HStack>
                  {filteredWorldsDetailed.map((world) => (
                    <div key={world.path}>
                      <HStack justify="between" align="center" className="euml-list-row" gap={3}>
                        {world.icon_path ? (
                          <img src={world.icon_path} alt="" className="euml-avatar" />
                        ) : (
                          <div className="euml-avatar" style={{ display: "grid", placeItems: "center", fontSize: 11 }}>
                            {world.name.slice(0, 2).toUpperCase()}
                          </div>
                        )}
                        <VStack gap={0} style={{ flex: 1, minWidth: 0 }}>
                          <Text>{world.name}</Text>
                          <Text color="secondary" type="supporting">
                            {world.has_backups
                              ? t("backupCount", { count: world.backup_count })
                              : t("noBackups")}
                          </Text>
                        </VStack>
                        <HStack gap={2}>
                          <Button
                            size="sm"
                            label={expandedWorld === world.name ? t("cancel") : t("worldBackups")}
                            onClick={() =>
                              setExpandedWorld(expandedWorld === world.name ? null : world.name)
                            }
                          />
                          <Button
                            size="sm"
                            label={t("worldSettings")}
                            onClick={() => setEditingWorld(world.name)}
                          />
                          <Button
                            size="sm"
                            label={t("openItem")}
                            variant="secondary"
                            onClick={() => api.openContentItem(world.path)}
                          />
                          <Button
                            size="sm"
                            label={t("delete")}
                            variant="destructive"
                            onClick={async () => {
                              if (!selected) return;
                              setContent(await api.deleteContent(selected.id, "saves", world.name));
                              setWorldsDetailed(await api.listWorldsDetailed(selected.id));
                              if (expandedWorld === world.name) setExpandedWorld(null);
                            }}
                          />
                        </HStack>
                      </HStack>
                      {expandedWorld === world.name && (
                        <VStack gap={2} style={{ padding: "8px 16px 16px", background: "var(--color-surface-secondary, rgba(0,0,0,0.03))" }}>
                          <HStack justify="between" align="center">
                            <Text weight="semibold">{t("worldBackups")}</Text>
                            <Button
                              size="sm"
                              label={t("createBackup")}
                              onClick={async () => {
                                if (!selected) return;
                                await api.createWorldBackup(selected.id, world.name);
                                setWorldsDetailed(await api.listWorldsDetailed(selected.id));
                                setWorldBackups(await api.listWorldBackups(selected.id, world.name));
                                setStatus(t("backupCreated"));
                              }}
                            />
                          </HStack>
                          {worldBackups.map((backup) => (
                            <HStack key={backup.path} justify="between" align="center" gap={2}>
                              <VStack gap={0} style={{ flex: 1, minWidth: 0 }}>
                                <Text type="supporting">{backup.name}</Text>
                                <Text color="secondary" type="supporting" style={{ fontSize: 11 }}>
                                  {backup.created_at}
                                </Text>
                              </VStack>
                              <HStack gap={2}>
                                <Button
                                  size="sm"
                                  label={t("restoreBackup")}
                                  onClick={async () => {
                                    if (!selected) return;
                                    if (!window.confirm(t("restoreBackupConfirm"))) return;
                                    await api.restoreWorldBackup(selected.id, world.name, backup.name);
                                    setWorldsDetailed(await api.listWorldsDetailed(selected.id));
                                    setWorldBackups(await api.listWorldBackups(selected.id, world.name));
                                    setStatus(t("backupRestored"));
                                  }}
                                />
                                <Button
                                  size="sm"
                                  label={t("deleteBackup")}
                                  variant="destructive"
                                  onClick={async () => {
                                    if (!selected) return;
                                    await api.deleteWorldBackup(selected.id, world.name, backup.name);
                                    setWorldsDetailed(await api.listWorldsDetailed(selected.id));
                                    setWorldBackups(await api.listWorldBackups(selected.id, world.name));
                                    setStatus(t("backupDeleted"));
                                  }}
                                />
                              </HStack>
                            </HStack>
                          ))}
                          {worldBackups.length === 0 && (
                            <Text color="secondary" type="supporting">
                              {t("noBackups")}
                            </Text>
                          )}
                        </VStack>
                      )}
                    </div>
                  ))}
                  {filteredWorldsDetailed.length === 0 && (
                    <div style={{ padding: 16 }}>
                      <Text color="secondary">{t("none")}</Text>
                    </div>
                  )}
                </Card>
                {editingWorld && worldSettings && (
                  <Card padding={4} className="euml-fade-in">
                    <VStack gap={3}>
                      <HStack justify="between" align="center">
                        <Text weight="semibold">
                          {t("worldSettings")}: {editingWorld}
                        </Text>
                        <Button size="sm" label={t("cancel")} onClick={() => setEditingWorld(null)} />
                      </HStack>
                      <TextInput
                        label={t("worldSeed")}
                        value={worldSettings.seed}
                        onChange={(v) => setWorldSettings({ ...worldSettings, seed: v })}
                        description={t("worldSeedWarn")}
                      />
                      <HStack gap={2}>
                        <Button
                          size="sm"
                          label={t("copySeed")}
                          onClick={() => navigator.clipboard.writeText(worldSettings.seed)}
                        />
                        <Button
                          size="sm"
                          label={t("openChunkbase")}
                          variant="primary"
                          onClick={() => {
                            const ver = selected.game_version.replace(/\./g, "_");
                            const url = `https://www.chunkbase.com/apps/seed-map#seed=${encodeURIComponent(worldSettings.seed)}&platform=java_${ver}`;
                            void import("@tauri-apps/plugin-opener").then(({ openUrl }) => openUrl(url));
                          }}
                        />
                      </HStack>
                      <Selector
                        label={t("difficulty")}
                        value={String(worldSettings.difficulty)}
                        onChange={(v) => setWorldSettings({ ...worldSettings, difficulty: Number(v) })}
                        options={[
                          { value: "0", label: "Peaceful" },
                          { value: "1", label: "Easy" },
                          { value: "2", label: "Normal" },
                          { value: "3", label: "Hard" },
                        ]}
                      />
                      <Selector
                        label={t("gameMode")}
                        value={String(worldSettings.game_type)}
                        onChange={(v) => setWorldSettings({ ...worldSettings, game_type: Number(v) })}
                        options={[
                          { value: "0", label: "Survival" },
                          { value: "1", label: "Creative" },
                          { value: "2", label: "Adventure" },
                          { value: "3", label: "Spectator" },
                        ]}
                      />
                      <HStack gap={3}>
                        <label>
                          <input
                            type="checkbox"
                            checked={worldSettings.hardcore}
                            onChange={(e) => setWorldSettings({ ...worldSettings, hardcore: e.target.checked })}
                          />{" "}
                          {t("hardcore")}
                        </label>
                        <label>
                          <input
                            type="checkbox"
                            checked={worldSettings.allow_commands}
                            onChange={(e) =>
                              setWorldSettings({ ...worldSettings, allow_commands: e.target.checked })
                            }
                          />{" "}
                          {t("allowCommands")}
                        </label>
                        <label>
                          <input
                            type="checkbox"
                            checked={worldSettings.keep_inventory}
                            onChange={(e) =>
                              setWorldSettings({ ...worldSettings, keep_inventory: e.target.checked })
                            }
                          />{" "}
                          keepInventory
                        </label>
                        <label>
                          <input
                            type="checkbox"
                            checked={worldSettings.do_daylight_cycle}
                            onChange={(e) =>
                              setWorldSettings({ ...worldSettings, do_daylight_cycle: e.target.checked })
                            }
                          />{" "}
                          doDaylightCycle
                        </label>
                      </HStack>
                      <Button
                        label={t("save")}
                        variant="primary"
                        onClick={async () => {
                          setWorldSettings(await api.saveWorldSettings(selected.id, worldSettings));
                          setStatus(t("worldSettingsSaved"));
                        }}
                      />
                    </VStack>
                  </Card>
                )}
              </VStack>
            )}
            {tab === "resourcepacks" && contentList("resourcepacks")}
            {tab === "shaders" && contentList("shaderpacks")}

            {tab === "datapacks" && (
              <VStack gap={3} className="euml-fade-in">
                <TextInput
                  label={t("searchInstalled")}
                  value={contentQuery}
                  onChange={setContentQuery}
                />
                <Selector
                  label={t("targetWorld")}
                  value={worldForDp}
                  onChange={setWorldForDp}
                  options={[
                    { value: "", label: "—" },
                    ...content.map((w) => ({ value: w.name, label: w.name })),
                  ]}
                />
                <HStack gap={2}>
                  <Button
                    size="sm"
                    label={t("importZip")}
                    isDisabled={!worldForDp}
                    onClick={async () => {
                      const path = await open({ filters: [{ name: "zip", extensions: ["zip"] }] });
                      if (!path || Array.isArray(path) || !worldForDp) return;
                      setDatapacks(await api.installDatapack(selected.id, worldForDp, path));
                    }}
                  />
                  <Button
                    size="sm"
                    label={t("importFile")}
                    variant="secondary"
                    isDisabled={!worldForDp}
                    onClick={async () => {
                      const path = await open({ directory: true });
                      if (!path || Array.isArray(path) || !worldForDp) return;
                      setDatapacks(await api.installDatapack(selected.id, worldForDp, path));
                    }}
                  />
                </HStack>
                <Card padding={0}>
                  {filteredDatapacks.map((d) => (
                    <HStack key={d.path} justify="between" align="center" className="euml-list-row">
                      <Text>{d.name}</Text>
                      <Button
                        size="sm"
                        label={t("delete")}
                        variant="destructive"
                        onClick={async () =>
                          setDatapacks(await api.deleteDatapack(selected.id, worldForDp, d.name))
                        }
                      />
                    </HStack>
                  ))}
                  {filteredDatapacks.length === 0 && (
                    <div style={{ padding: 16 }}>
                      <Text color="secondary">{worldForDp ? t("none") : t("pickWorld")}</Text>
                    </div>
                  )}
                </Card>
              </VStack>
            )}

            {tab === "reqguard" && (
              <Card padding={4} className="euml-fade-in">
                <HStack gap={2} style={{ flexWrap: "wrap" }}>
                  <Button label={t("rerunReqguard")} onClick={async () => setScan(await api.reqguardScan(selected.id))} />
                  {scan && scan.issues.some((i) => i.severity === "error") && (
                    <Button
                      label={t("installAllMissing")}
                      variant="primary"
                      onClick={async () => setScan(await api.reqguardResolveAll(selected.id))}
                    />
                  )}
                </HStack>
                <VStack gap={2} style={{ marginTop: 12 }}>
                  {scan?.issues.map((issue, i) => (
                    <VStack key={i} gap={1}>
                      <Text type="supporting">
                        {issue.source ? `[${issue.source}] ` : ""}
                        {issue.message}
                      </Text>
                      {(issue.project_id || issue.missing_mod_id) && (
                        <Button
                          size="sm"
                          variant="secondary"
                          label={`${t("installMissing")}: ${issue.project_id || issue.missing_mod_id}`}
                          onClick={async () =>
                            setScan(
                              await api.reqguardResolve(
                                selected.id,
                                issue.project_id || issue.missing_mod_id!,
                              ),
                            )
                          }
                        />
                      )}
                    </VStack>
                  ))}
                  {scan && scan.issues.length === 0 && (
                    <Text color="accent">{t("reqguardOk", { count: scan.mod_count })}</Text>
                  )}
                </VStack>
              </Card>
            )}

            {tab === "logs" && (
              <Card padding={3} className="euml-fade-in">
                <pre style={{ maxHeight: "50vh", overflow: "auto", fontSize: 12, margin: 0 }}>
                  {logs.map((l, i) => (
                    <div key={i}>{l.text}</div>
                  ))}
                </pre>
              </Card>
            )}

            {tab === "advanced" && draft && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <Text weight="semibold">{t("instanceSettings")}</Text>
                  <TextInput
                    label={t("name")}
                    value={draft.name}
                    onChange={(v) => setDraft({ ...draft, name: v })}
                  />
                  <TextInput
                    label={t("gameVersion")}
                    value={draft.game_version}
                    onChange={(v) => setDraft({ ...draft, game_version: normalizeMcVersion(v) })}
                    description={t("upgradeVersionHint")}
                  />
                  <Selector
                    label={t("loader")}
                    value={draft.loader}
                    onChange={(v) => setDraft({ ...draft, loader: v as Instance["loader"] })}
                    options={[
                      { value: "vanilla", label: "vanilla" },
                      { value: "fabric", label: "fabric" },
                      { value: "quilt", label: "quilt" },
                      { value: "forge", label: "forge" },
                      { value: "neoforge", label: "neoforge" },
                    ]}
                  />
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <Button
                      label={t("save")}
                      variant="primary"
                      onClick={async () => {
                        const updated = await api.changeInstanceVersion(
                          draft.id,
                          normalizeMcVersion(draft.game_version),
                          draft.loader,
                          draft.loader_version,
                        );
                        const named = await api.updateInstance({ ...updated, name: draft.name, memory_mb: draft.memory_mb, jvm_args: draft.jvm_args });
                        setSelected(named);
                        setDraft(named);
                        await refresh();
                        setStatus(t("saved"));
                      }}
                    />
                    <Button
                      label={t("reinstallLoader")}
                      onClick={async () => {
                        setStatus(t("preparing"));
                        try {
                          const inst = await api.reinstallLoader(selected.id);
                          setSelected(inst);
                          setDraft(inst);
                          setStatus(t("loaderReinstalled"));
                        } catch (e) {
                          setError(String(e));
                        }
                      }}
                    />
                    <Button
                      label={t("setIcon")}
                      onClick={async () => {
                        const path = await open({
                          filters: [{ name: "image", extensions: ["png", "jpg", "jpeg", "webp"] }],
                        });
                        if (!path || Array.isArray(path)) return;
                        const inst = await api.setInstanceIcon(selected.id, path);
                        setSelected(inst);
                        setDraft(inst);
                        await refresh();
                      }}
                    />
                    <Button
                      label={t("redownloadAssets")}
                      onClick={async () => {
                        setStatus(t("preparing"));
                        try {
                          setStatus(await api.prepareInstance(selected.id));
                        } catch (e) {
                          setError(String(e));
                        }
                      }}
                    />
                  </HStack>
                  <TextInput label="JVM" value={draft.jvm_args} onChange={(v) => setDraft({ ...draft, jvm_args: v })} />
                  <TextInput
                    label="Memory MB"
                    value={String(draft.memory_mb)}
                    onChange={(v) => setDraft({ ...draft, memory_mb: Number(v) || 4096 })}
                  />
                </VStack>
              </Card>
            )}
          </>
        )}
      </VStack>
    </HStack>
  );
}
