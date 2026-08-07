import type { FormEvent } from "react";
import { useCallback, useDeferredValue, useEffect, useMemo, useRef, useState, useTransition } from "react";
import { useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { Tab, TabList } from "@astryxdesign/core/TabList";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { Selector } from "@astryxdesign/core/Selector";
import { Spinner } from "@astryxdesign/core/Spinner";
import { DismissibleBanner } from "../components/DismissibleBanner";
import { NewsPanel } from "../components/NewsPanel";
import { FavoriteButton } from "../components/FavoriteButton";
import { ModInstallPicker, type ModInstallKind } from "../components/ModInstallPicker";
import { api } from "../lib/api";
import { normalizeMcVersion } from "../lib/mcVersion";
import { effectiveLoader } from "../lib/loaderDetect";
import { useFavorites } from "../lib/favorites";
import { loadPreferredInstanceId, rememberPreferredInstance } from "../lib/preferredInstance";
import { favoriteId } from "../lib/types";
import { useI18n } from "../i18n";
import type { Instance, JavaStatus, LoaderKind, ModrinthHit, ModrinthProjectType, VersionInfo } from "../lib/types";

const LOADERS: LoaderKind[] = ["vanilla", "fabric", "quilt", "forge", "neoforge"];

const MOD_TAGS = [
  "adventure",
  "cursed",
  "decoration",
  "economy",
  "equipment",
  "food",
  "game-mechanics",
  "library",
  "magic",
  "management",
  "minigame",
  "mobs",
  "optimization",
  "social",
  "storage",
  "technology",
  "transportation",
  "utility",
  "worldgen",
];

type ContentTab = "mods" | "modpack" | "resourcepack" | "shader" | "datapack";

export function DownloadPage() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { isFavorite } = useFavorites();
  const [tab, setTab] = useState("game");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [versions, setVersions] = useState<VersionInfo[]>([]);
  const [filter, setFilter] = useState("release");
  const [q, setQ] = useState("");
  const deferredQ = useDeferredValue(q);
  const deferredFilter = useDeferredValue(filter);
  const [name, setName] = useState("");
  const [gameVersion, setGameVersion] = useState("1.21.1");
  const [loader, setLoader] = useState<LoaderKind>("fabric");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [modQuery, setModQuery] = useState("");
  const deferredModQuery = useDeferredValue(modQuery);
  const [activeTag, setActiveTag] = useState<string | null>(null);
  const [modHits, setModHits] = useState<ModrinthHit[]>([]);
  const [targetInstance, setTargetInstance] = useState("");
  const [searchVersion, setSearchVersion] = useState("1.21.1");
  const [searchLoader, setSearchLoader] = useState<LoaderKind>("fabric");
  const [targetWorld, setTargetWorld] = useState("");
  const [worlds, setWorlds] = useState<{ name: string }[]>([]);
  const [instances, setInstances] = useState<Instance[]>([]);
  const [searching, setSearching] = useState(false);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [installPicker, setInstallPicker] = useState<{
    hit: ModrinthHit;
    kind: ModInstallKind;
  } | null>(null);
  const [javaStatus, setJavaStatus] = useState<JavaStatus | null>(null);
  const [javaBusy, setJavaBusy] = useState(false);
  const [, startTransition] = useTransition();
  const searchGen = useRef(0);

  const releaseVersions = useMemo(
    () => versions.filter((v) => v.type_ === "release").map((v) => v.id),
    [versions],
  );
  const versionOptions = useMemo(() => {
    const ids =
      filter === "all"
        ? versions.map((v) => v.id)
        : versions.filter((v) => v.type_ === filter).map((v) => v.id);
    const uniq = Array.from(new Set([searchVersion, gameVersion, ...ids]));
    return uniq.filter(Boolean).slice(0, 200).map((id) => ({ value: id, label: id }));
  }, [versions, filter, searchVersion, gameVersion]);

  useEffect(() => {
    let cancelled = false;
    let instanceReady = false;
    void loadPreferredInstanceId().then(({ instances: list, instanceId }) => {
      if (cancelled) return;
      instanceReady = true;
      setInstances(list);
      if (instanceId) {
        setTargetInstance(instanceId);
        const inst = list.find((i) => i.id === instanceId) ?? list[0];
        if (inst) {
          setSearchVersion(normalizeMcVersion(inst.game_version));
          setSearchLoader(effectiveLoader(inst));
        }
      }
    });
    api
      .listVersionsDetailed()
      .then((v) => {
        if (cancelled) return;
        setVersions(v);
        const first = v.find((x) => x.type_ === "release");
        if (first) {
          setGameVersion(first.id);
          if (!instanceReady) setSearchVersion(first.id);
        }
      })
      .catch(async () => {
        try {
          const ids = await api.listVersions();
          if (cancelled) return;
          setVersions(ids.map((id) => ({ id, type_: "release", release_time: "" })));
        } catch (e) {
          if (!cancelled) setError(String(e));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const inst = instances.find((i) => i.id === targetInstance);
    if (!inst) return;
    setSearchVersion(normalizeMcVersion(inst.game_version));
    setSearchLoader(effectiveLoader(inst));
    void rememberPreferredInstance(targetInstance);
  }, [targetInstance, instances]);

  useEffect(() => {
    let cancelled = false;
    void api.javaStatus(normalizeMcVersion(gameVersion)).then((s) => {
      if (!cancelled) setJavaStatus(s);
    }).catch(() => {
      if (!cancelled) setJavaStatus(null);
    });
    return () => {
      cancelled = true;
    };
  }, [gameVersion]);

  useEffect(() => {
    if (!targetInstance || tab !== "datapack") {
      setWorlds([]);
      return;
    }
    api
      .listWorlds(targetInstance)
      .then((w) => {
        setWorlds(w);
        if (w[0]) setTargetWorld(w[0].name);
      })
      .catch(() => setWorlds([]));
  }, [targetInstance, tab]);

  const contentTab: ContentTab | null =
    tab === "mods" ||
    tab === "packs" ||
    tab === "resourcepack" ||
    tab === "shader" ||
    tab === "datapack"
      ? tab === "packs"
        ? "modpack"
        : tab
      : null;

  const runModrinthSearch = useCallback(
    (queryOverride?: string) => {
      if (!contentTab) return;
      const gen = ++searchGen.current;
      const projectType: ModrinthProjectType =
        contentTab === "mods"
          ? "mod"
          : contentTab === "modpack"
            ? "modpack"
            : contentTab === "resourcepack"
              ? "resourcepack"
              : contentTab === "shader"
                ? "shader"
                : "datapack";
      const cats = activeTag && contentTab === "mods" ? [activeTag] : undefined;
      setSearching(true);
      setError(null);
      const gv = normalizeMcVersion(searchVersion);
      const q = (queryOverride ?? modQuery).trim();
      const run =
        projectType === "mod"
          ? api.searchMods(q, gv, searchLoader, cats)
          : api.searchContent(q, gv, searchLoader, projectType, cats);
      void run
        .then((hits) => {
          if (searchGen.current !== gen) return;
          setModHits(hits);
          setStatus(hits.length === 0 ? t("noResults") : null);
        })
        .catch((e) => {
          if (searchGen.current !== gen) return;
          setError(String(e));
          setModHits([]);
        })
        .finally(() => {
          if (searchGen.current === gen) setSearching(false);
        });
    },
    [contentTab, activeTag, searchVersion, searchLoader, modQuery, t],
  );

  // Auto-search while typing / changing filters (does not require a target instance).
  useEffect(() => {
    if (!contentTab) return;
    const timer = window.setTimeout(() => {
      runModrinthSearch(deferredModQuery);
    }, 320);
    return () => {
      clearTimeout(timer);
    };
  }, [contentTab, deferredModQuery, activeTag, searchVersion, searchLoader, runModrinthSearch]);

  const shown = useMemo(() => {
    const out: VersionInfo[] = [];
    const fav: VersionInfo[] = [];
    const rest: VersionInfo[] = [];
    for (const v of versions) {
      if (deferredFilter !== "all" && v.type_ !== deferredFilter) continue;
      if (deferredQ && !v.id.includes(deferredQ)) continue;
      if (isFavorite(favoriteId("mcversion", v.id))) fav.push(v);
      else rest.push(v);
    }
    for (const v of [...fav, ...rest]) {
      out.push(v);
      if (out.length >= 80) break;
    }
    return out;
  }, [versions, deferredFilter, deferredQ, isFavorite]);

  async function installGame(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setStatus(t("preparing"));
    try {
      const gv = normalizeMcVersion(gameVersion);
      if (!gv) {
        setError(
          `Invalid Minecraft version "${gameVersion}". Pick a real version like 1.21.1, not a pack name.`,
        );
        setStatus(null);
        return;
      }
      const inst = await api.createInstance({
        name: name || `${gv}-${loader}`,
        gameVersion: gv,
        loader,
      });
      if (loader !== "vanilla") await api.installLoader(inst.id);
      await api.prepareInstance(inst.id);
      setStatus(`${inst.name}`);
      navigate(`/versions/${inst.id}`);
    } catch (err) {
      setError(String(err));
      setStatus(null);
    }
  }

  function openInstallPicker(hit: ModrinthHit, kind: ContentTab) {
    if (!hit.versions[0]) {
      setError(t("noCompatibleVersion"));
      return;
    }
    if (kind !== "modpack" && !targetInstance) {
      setError(t("pickInstance"));
      return;
    }
    if (kind === "datapack" && !targetWorld) {
      setError(t("pickWorld"));
      return;
    }
    setError(null);
    const installKind: ModInstallKind =
      kind === "mods" ? "mod" : (kind as ModInstallKind);
    setInstallPicker({ hit, kind: installKind });
  }

  async function installDatapackDirect(hit: ModrinthHit) {
    // Datapacks still need a world target; picker installs content without world.
    if (!hit.versions[0] || !targetInstance) return;
    if (!targetWorld) {
      setError(t("pickWorld"));
      return;
    }
    setInstallingId(hit.project_id);
    setError(null);
    try {
      await api.installContentFromModrinth(
        targetInstance,
        hit.versions[0].id,
        "datapack",
        targetWorld,
      );
      setStatus(`${t("install")}: ${hit.title}`);
    } catch (e) {
      setError(String(e));
    } finally {
      setInstallingId(null);
    }
  }

  function openDetails(hit: ModrinthHit) {
    const params = new URLSearchParams({
      v: normalizeMcVersion(searchVersion),
      loader: searchLoader,
      instance: targetInstance,
    });
    navigate(`/download/mod/${hit.project_id}?${params.toString()}`);
  }

  function contentPanel(kind: ContentTab) {
    return (
      <Card padding={4} className="euml-fade-in">
        <VStack gap={3}>
          <div className="euml-toolbar">
            {kind !== "modpack" && (
              <Selector
                label={t("targetVersion")}
                value={targetInstance}
                onChange={setTargetInstance}
                options={[
                  { value: "", label: "—" },
                  ...instances.map((i) => ({
                    value: i.id,
                    label: `${i.name} · ${normalizeMcVersion(i.game_version)} · ${i.loader}`,
                  })),
                ]}
              />
            )}
            <Selector
              label={t("gameVersion")}
              value={searchVersion}
              onChange={setSearchVersion}
              options={
                versionOptions.length
                  ? versionOptions
                  : releaseVersions.map((id) => ({ value: id, label: id }))
              }
            />
            {(kind === "mods" || kind === "modpack") && (
              <Selector
                label={t("loader")}
                value={searchLoader}
                onChange={(v) => setSearchLoader(v as LoaderKind)}
                options={LOADERS.map((l) => ({ value: l, label: l }))}
              />
            )}
            {kind === "datapack" && (
              <Selector
                label={t("targetWorld")}
                value={targetWorld}
                onChange={setTargetWorld}
                options={[
                  { value: "", label: "—" },
                  ...worlds.map((w) => ({ value: w.name, label: w.name })),
                ]}
              />
            )}
            <TextInput
              label={kind === "modpack" ? t("searchModpacks") : t("searchMods")}
              value={modQuery}
              onChange={setModQuery}
              onEnter={() => runModrinthSearch(modQuery)}
            />
            <Button
              size="sm"
              label={searching ? t("searching") : t("search")}
              isLoading={searching}
              onClick={() => runModrinthSearch(modQuery)}
            />
            {searching && <Spinner size="sm" />}
          </div>

          {kind === "modpack" && (
            <Text color="secondary" type="supporting">
              {t("modpackInstallHint")}
            </Text>
          )}

          {(kind === "mods" ||
            kind === "modpack" ||
            kind === "resourcepack" ||
            kind === "shader" ||
            kind === "datapack") && (
            <div className="euml-tags">
              <button
                type="button"
                className={`euml-tag${favoritesOnly ? " is-active" : ""}`}
                onClick={() => setFavoritesOnly((v) => !v)}
              >
                ★ {t("showFavoritesOnly")}
              </button>
              {kind === "mods" &&
                MOD_TAGS.map((tag) => (
                  <button
                    key={tag}
                    type="button"
                    className={`euml-tag${activeTag === tag ? " is-active" : ""}`}
                    onClick={() => setActiveTag((prev) => (prev === tag ? null : tag))}
                  >
                    {tag}
                  </button>
                ))}
            </div>
          )}

          {(favoritesOnly
            ? modHits.filter((h) => isFavorite(favoriteId("modrinth", h.project_id)))
            : modHits
          ).map((h) => {
            const ok = Boolean(h.versions[0]);
            const cats = (h.categories ?? []).filter((c) => !LOADERS.includes(c as LoaderKind)).slice(0, 4);
            return (
              <HStack key={h.project_id} justify="between" align="center" className="euml-fade-in" gap={3}>
                {h.icon_url ? (
                  <img src={h.icon_url} alt="" className="euml-avatar" loading="lazy" />
                ) : (
                  <div className="euml-loader-badge" data-loader="vanilla">
                    ?
                  </div>
                )}
                <VStack gap={0.5} style={{ minWidth: 0, flex: 1 }}>
                  <button
                    type="button"
                    onClick={() => openDetails(h)}
                    style={{
                      background: "none",
                      border: "none",
                      padding: 0,
                      textAlign: "left",
                      cursor: "pointer",
                      color: "inherit",
                      font: "inherit",
                    }}
                  >
                    <Text weight="semibold">{h.title}</Text>
                  </button>
                  <Text color="secondary" type="supporting" className="euml-hit-desc">
                    {h.description}
                  </Text>
                  {cats.length > 0 && (
                    <Text color="secondary" type="supporting">
                      {cats.join(" · ")}
                    </Text>
                  )}
                  {!ok && (
                    <Text color="secondary" type="supporting">
                      {t("noCompatibleVersion")}
                    </Text>
                  )}
                </VStack>
                <HStack gap={2} align="center">
                  <FavoriteButton
                    kind="modrinth"
                    itemKey={h.project_id}
                    label={h.title}
                    subtitle={h.slug}
                    iconUrl={h.icon_url}
                  />
                  <Button size="sm" label={t("details")} onClick={() => openDetails(h)} />
                  <Button
                    label={
                      installingId === h.project_id
                        ? kind === "modpack"
                          ? t("installingModpack")
                          : t("installing")
                        : kind === "modpack"
                          ? t("installModpack")
                          : t("install")
                    }
                    size="sm"
                    isDisabled={
                      (!ok || installingId != null) || (kind !== "modpack" && !targetInstance)
                    }
                    isLoading={installingId === h.project_id}
                    onClick={() =>
                      kind === "datapack" ? void installDatapackDirect(h) : openInstallPicker(h, kind)
                    }
                  />
                </HStack>
              </HStack>
            );
          })}
          {!searching &&
            (favoritesOnly
              ? modHits.filter((h) => isFavorite(favoriteId("modrinth", h.project_id))).length === 0
              : modHits.length === 0) && (
              <Text color="secondary">{favoritesOnly ? t("favoritesEmpty") : t("noResults")}</Text>
            )}
        </VStack>
      </Card>
    );
  }

  return (
    <VStack gap={4} className="euml-page">
      {installPicker && (
        <ModInstallPicker
          projectId={installPicker.hit.project_id}
          title={installPicker.hit.title}
          gameVersion={normalizeMcVersion(searchVersion)}
          loader={searchLoader}
          instanceId={targetInstance}
          kind={installPicker.kind}
          onClose={() => setInstallPicker(null)}
          onInstalled={({ label, instanceId }) => {
            setStatus(`${t("install")}: ${label}`);
            setInstallPicker(null);
            if (instanceId) navigate(`/versions/${instanceId}`);
          }}
        />
      )}
      <Text type="display-3">{t("downloadTitle")}</Text>
      <TabList
        value={tab}
        onChange={(v) => {
          setTab(v);
          setModHits([]);
          setActiveTag(null);
          setStatus(null);
          setError(null);
        }}
      >
        <Tab value="game" label={t("tabGame")} />
        <Tab value="mods" label={t("tabMods")} />
        <Tab value="resourcepack" label={t("tabResourcepacks")} />
        <Tab value="shader" label={t("tabShaders")} />
        <Tab value="datapack" label={t("tabDatapacks")} />
        <Tab value="packs" label={t("tabPacks")} />
        <Tab value="news" label={t("tabNews")} />
      </TabList>
      {status && <DismissibleBanner status="info" title={status} onDismiss={() => setStatus(null)} />}
      {error && <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />}

      {tab === "news" && <NewsPanel />}

      {tab === "game" && (
        <HStack gap={4} align="start" className="euml-fade-in">
          <Card
            padding={0}
            style={{ flex: 1, maxHeight: "70vh", overflow: "hidden", display: "flex", flexDirection: "column" }}
          >
            <HStack gap={2} style={{ padding: 12, borderBottom: "1px solid var(--color-border)" }}>
              <TextInput label={t("searchVersion")} value={q} onChange={setQ} />
              <Selector
                label={t("all")}
                value={filter}
                onChange={setFilter}
                options={[
                  { value: "release", label: t("releases") },
                  { value: "snapshot", label: t("snapshots") },
                  { value: "all", label: t("all") },
                ]}
              />
            </HStack>
            <div style={{ overflow: "auto", flex: 1, contentVisibility: "auto" }}>
              {shown.map((v) => (
                <div
                  key={v.id}
                  className={`euml-list-row${gameVersion === v.id ? " is-selected" : ""}`}
                  style={{ cursor: "pointer" }}
                  onClick={() => {
                    startTransition(() => {
                      setGameVersion(v.id);
                      setName(`${v.id}-${loader}`);
                    });
                  }}
                >
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <span className="euml-list-row__title">{v.id}</span>
                    <span className="euml-list-row__meta">{v.type_}</span>
                  </div>
                  <FavoriteButton kind="mcversion" itemKey={v.id} label={v.id} subtitle={v.type_} />
                </div>
              ))}
            </div>
          </Card>
          <Card padding={4} style={{ width: 320 }}>
            <form onSubmit={installGame}>
              <VStack gap={3}>
                <Text weight="semibold">{t("oneClickInstall")}</Text>
                {javaStatus && (
                  <VStack gap={1}>
                    <Text weight="semibold" type="supporting">
                      {t("javaPrereqTitle")}
                    </Text>
                    <Text color="secondary" type="supporting">
                      {t("javaPrereqNeed", {
                        version: gameVersion,
                        major: javaStatus.required_major,
                      })}
                    </Text>
                    <Text color={javaStatus.satisfied ? "accent" : "secondary"} type="supporting">
                      {javaStatus.satisfied
                        ? t("javaPrereqOk", { major: javaStatus.required_major })
                        : t("javaPrereqMissing")}
                    </Text>
                    {!javaStatus.satisfied && (
                      <Button
                        type="button"
                        size="sm"
                        variant="secondary"
                        label={
                          javaBusy
                            ? t("javaDownloading")
                            : t("javaDownloadTemurin", { major: javaStatus.required_major })
                        }
                        isDisabled={javaBusy}
                        onClick={async () => {
                          setJavaBusy(true);
                          setError(null);
                          try {
                            const path = await api.downloadTemurin(javaStatus.required_major);
                            setStatus(path);
                            setJavaStatus(await api.javaStatus(normalizeMcVersion(gameVersion)));
                          } catch (e) {
                            setError(String(e));
                          } finally {
                            setJavaBusy(false);
                          }
                        }}
                      />
                    )}
                  </VStack>
                )}
                <TextInput label={t("name")} value={name} onChange={setName} />
                <Selector
                  label={t("gameVersion")}
                  value={gameVersion}
                  onChange={setGameVersion}
                  options={versionOptions.length ? versionOptions : [{ value: gameVersion, label: gameVersion }]}
                />
                <Selector
                  label={t("loader")}
                  value={loader}
                  onChange={(v) => setLoader(v as LoaderKind)}
                  options={LOADERS.map((l) => ({ value: l, label: l }))}
                />
                <Button type="submit" label={t("downloadInstall")} variant="primary" width="100%" />
              </VStack>
            </form>
          </Card>
        </HStack>
      )}

      {tab === "mods" && contentPanel("mods")}
      {tab === "resourcepack" && contentPanel("resourcepack")}
      {tab === "shader" && contentPanel("shader")}
      {tab === "datapack" && contentPanel("datapack")}

      {tab === "packs" && (
        <VStack gap={4} className="euml-fade-in">
          {contentPanel("modpack")}
          <Card padding={4}>
            <VStack gap={3}>
              <Text weight="semibold">{t("importLocalPack")}</Text>
              <Text color="secondary" type="supporting">
                {t("importFolderHint")}
              </Text>
              <HStack gap={2} style={{ flexWrap: "wrap" }}>
                <Button
                  label={t("importMrpack")}
                  onClick={async () => {
                    const path = await open({ filters: [{ name: "mrpack", extensions: ["mrpack"] }] });
                    if (!path || Array.isArray(path)) return;
                    navigate(`/versions/${(await api.importMrpack(path)).id}`);
                  }}
                />
                <Button
                  label={t("importPrism")}
                  onClick={async () => {
                    const path = await open({ directory: true });
                    if (!path || Array.isArray(path)) return;
                    navigate(`/versions/${(await api.importForeignInstance(path)).id}`);
                  }}
                />
                <Button
                  label={t("importFolder")}
                  variant="primary"
                  onClick={async () => {
                    const path = await open({ directory: true });
                    if (!path || Array.isArray(path)) return;
                    setError(null);
                    try {
                      const list = await api.importInstanceFolder(path);
                      setStatus(t("importedCount", { count: list.length }));
                      if (list[0]) navigate(`/versions/${list[0].id}`);
                    } catch (e) {
                      setError(String(e));
                    }
                  }}
                />
              </HStack>
            </VStack>
          </Card>
        </VStack>
      )}
    </VStack>
  );
}
