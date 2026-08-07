import { open, save } from "@tauri-apps/plugin-dialog";
import type { FormEvent } from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { CheckboxInput } from "@astryxdesign/core/CheckboxInput";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { Selector } from "@astryxdesign/core/Selector";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { Tab, TabList } from "@astryxdesign/core/TabList";
import { MetricSparkline } from "../components/MetricSparkline";
import { DismissibleBanner } from "../components/DismissibleBanner";
import { FavoriteButton } from "../components/FavoriteButton";
import { api } from "../lib/api";
import { useDownloadStatus } from "../lib/downloadStatus";
import { useI18n } from "../i18n";
import type {
  DedicatedNetworkInfo,
  DedicatedPlayerLists,
  DedicatedProperties,
  DedicatedServer,
  DedicatedStatus,
  HangarProject,
  HangarVersion,
  HostLiveStats,
  HostPluginEntry,
  VersionInfo,
} from "../lib/types";

const HOST_LOADERS = [
  "vanilla",
  "fabric",
  "quilt",
  "forge",
  "neoforge",
  "paper",
  "purpur",
] as const;

const METRIC_HISTORY = 40;

function formatBps(bps: number | null | undefined): string {
  if (bps == null || !Number.isFinite(bps)) return "—";
  if (bps < 1024) return `${bps.toFixed(0)} B/s`;
  if (bps < 1024 * 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
  return `${(bps / (1024 * 1024)).toFixed(2)} MB/s`;
}

function formatMb(mb: number | null | undefined): string {
  if (mb == null || !Number.isFinite(mb)) return "—";
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`;
  return `${mb.toFixed(0)} MB`;
}

function pushMetric(prev: number[], next: number | null | undefined): number[] {
  if (next == null || !Number.isFinite(next)) return prev;
  const out = [...prev, next];
  return out.length > METRIC_HISTORY ? out.slice(-METRIC_HISTORY) : out;
}

function hostIdFromPath(pathname: string): string | undefined {
  const m = pathname.match(/^\/host\/([^/]+)/);
  return m?.[1] ? decodeURIComponent(m[1]) : undefined;
}

function isMissingServerError(e: unknown): boolean {
  const s = String(e);
  return s.includes("Server not found") || s.includes("folder may have been deleted");
}

function isPaperLikeLoader(loader: string): boolean {
  const l = loader.toLowerCase();
  return l === "paper" || l === "purpur";
}

const emptyLists = (): DedicatedPlayerLists => ({
  whitelist: [],
  ops: [],
  bannedPlayers: [],
  bannedIps: [],
});

export function HostPage() {
  const { t } = useI18n();
  const { pathname } = useLocation();
  const navigate = useNavigate();
  const id = hostIdFromPath(pathname);
  const { consoleLines } = useDownloadStatus();

  const [servers, setServers] = useState<DedicatedServer[]>([]);
  const [selected, setSelected] = useState<DedicatedServer | null>(null);
  const [status, setStatus] = useState<DedicatedStatus | null>(null);
  const [props, setProps] = useState<DedicatedProperties | null>(null);
  const [lists, setLists] = useState<DedicatedPlayerLists>(emptyLists());
  const [net, setNet] = useState<DedicatedNetworkInfo | null>(null);
  const [tab, setTab] = useState("console");
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [creating, setCreating] = useState(false);
  const [versions, setVersions] = useState<VersionInfo[]>([]);
  const [versionFilter, setVersionFilter] = useState<"release" | "snapshot" | "all">("release");
  const [form, setForm] = useState({
    name: "",
    gameVersion: "1.21.1",
    loader: "vanilla",
    memoryMb: 2048,
    port: 25565,
  });
  const [command, setCommand] = useState("");
  const [listName, setListName] = useState("");
  const [listUuid, setListUuid] = useState("");
  const [listIp, setListIp] = useState("");
  const [live, setLive] = useState<HostLiveStats | null>(null);
  const [cpuHist, setCpuHist] = useState<number[]>([]);
  const [ramHist, setRamHist] = useState<number[]>([]);
  const [ramSysHist, setRamSysHist] = useState<number[]>([]);
  const [netDownHist, setNetDownHist] = useState<number[]>([]);
  const [netUpHist, setNetUpHist] = useState<number[]>([]);
  const [cpuCount, setCpuCount] = useState(4);
  const [pluginQuery, setPluginQuery] = useState("");
  const [pluginHits, setPluginHits] = useState<HangarProject[]>([]);
  const [pluginVersions, setPluginVersions] = useState<HangarVersion[]>([]);
  const [pluginPick, setPluginPick] = useState<HangarProject | null>(null);
  const [pluginVersion, setPluginVersion] = useState("latest");
  const [installedPlugins, setInstalledPlugins] = useState<HostPluginEntry[]>([]);
  const [pluginSearching, setPluginSearching] = useState(false);
  const consoleRef = useRef<HTMLPreElement>(null);

  const paperLike = selected ? isPaperLikeLoader(String(selected.loader)) : false;

  const versionOptions = useMemo(() => {
    const filtered = versions.filter((v) => {
      if (versionFilter === "all") return true;
      if (versionFilter === "release") return v.type_ === "release";
      return v.type_ !== "release";
    });
    const ids = filtered.map((v) => v.id);
    const uniq = Array.from(new Set([form.gameVersion, ...ids]));
    return uniq.filter(Boolean).slice(0, 250).map((id) => ({ value: id, label: id }));
  }, [versions, versionFilter, form.gameVersion]);

  const hostLines = useMemo(() => {
    if (!id) return [];
    const prefix = `[host:${id}]`;
    return consoleLines.filter((l) => l.text.includes(prefix));
  }, [consoleLines, id]);

  useEffect(() => {
    let cancelled = false;
    api
      .listVersionsDetailed()
      .then((v) => {
        if (!cancelled) setVersions(v);
      })
      .catch(() => {
        api.listVersions().then((ids) => {
          if (!cancelled) {
            setVersions(ids.map((id) => ({ id, type_: "release", release_time: "" })));
          }
        });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    api.dedicatedCpuCount().then(setCpuCount).catch(() => setCpuCount(4));
  }, []);

  useEffect(() => {
    const el = consoleRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [hostLines.length]);

  const refreshInstalledPlugins = useCallback(async (serverId: string) => {
    const list = await api.dedicatedListPlugins(serverId);
    setInstalledPlugins(list);
    return list;
  }, []);

  async function onSearchPlugins(e?: FormEvent) {
    e?.preventDefault();
    if (!selected) return;
    setPluginSearching(true);
    setError(null);
    try {
      const hits = await api.hangarSearchPlugins(pluginQuery, "PAPER", 24);
      setPluginHits(hits);
      setPluginPick(null);
      setPluginVersions([]);
      setPluginVersion("latest");
    } catch (err) {
      setError(String(err));
    } finally {
      setPluginSearching(false);
    }
  }

  async function onPickPlugin(hit: HangarProject) {
    if (!selected) return;
    setPluginPick(hit);
    setPluginVersion("latest");
    try {
      const versions = await api.hangarListPluginVersions(hit.author, hit.slug, "PAPER");
      setPluginVersions(versions);
    } catch (err) {
      setError(String(err));
      setPluginVersions([]);
    }
  }

  async function onInstallPlugin() {
    if (!selected || !pluginPick) return;
    setBusy(true);
    setError(null);
    try {
      await api.hangarInstallPlugin(
        selected.id,
        pluginPick.author,
        pluginPick.slug,
        pluginVersion || "latest",
        "PAPER",
      );
      await refreshInstalledPlugins(selected.id);
      setInfo(t("hostPluginInstalled"));
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  const refreshList = useCallback(async () => {
    const list = await api.listDedicated();
    setServers(list);
    return list;
  }, []);

  const handleMissing = useCallback(async () => {
    setError(t("hostNotFound"));
    setSelected(null);
    setStatus(null);
    setProps(null);
    setLists(emptyLists());
    setNet(null);
    await refreshList().catch(() => undefined);
    navigate("/host");
  }, [navigate, refreshList, t]);

  const loadDetail = useCallback(
    async (serverId: string, listHint?: Awaited<ReturnType<typeof api.listDedicated>>) => {
      try {
        const list = listHint ?? (await refreshList());
        const server = list.find((s) => s.id === serverId);
        if (!server) {
          await handleMissing();
          return;
        }
        setSelected(server);
        // Fast path: status + config first. Network/UPnP/public-IP is deferred so
        // opening Host does not wait on gateway discovery.
        const [st, pr, pl] = await Promise.all([
          api.dedicatedStatus(serverId),
          api.getDedicatedProperties(serverId),
          api.getDedicatedPlayerLists(serverId),
        ]);
        setStatus(st);
        setProps(pr);
        setLists(pl);
        void api
          .dedicatedNetworkInfo(serverId)
          .then(setNet)
          .catch(() => undefined);
      } catch (e) {
        if (isMissingServerError(e)) await handleMissing();
        else setError(String(e));
      }
    },
    [handleMissing, refreshList],
  );

  const onHostRoute = pathname === "/host" || pathname.startsWith("/host/");

  useEffect(() => {
    if (!onHostRoute) return;
    let cancelled = false;
    refreshList()
      .then((list) => {
        if (cancelled || !onHostRoute) return;
        if (id) return loadDetail(id, list);
        setSelected(list[0] ?? null);
        if (list[0]) navigate(`/host/${list[0].id}`, { replace: true });
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [id, onHostRoute]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (!onHostRoute) return;
    setCpuHist([]);
    setRamHist([]);
    setRamSysHist([]);
    setNetDownHist([]);
    setNetUpHist([]);
    setLive(null);
  }, [id, onHostRoute]);

  useEffect(() => {
    if (!onHostRoute || !id || !paperLike || tab !== "plugins") return;
    let cancelled = false;
    api
      .dedicatedListPlugins(id)
      .then((list) => {
        if (!cancelled) setInstalledPlugins(list);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [onHostRoute, id, paperLike, tab]);

  useEffect(() => {
    if (!onHostRoute || !id || !status?.running || tab !== "live") return;
    const tick = () => {
      api
        .dedicatedLiveStats(id)
        .then((s) => {
          setLive(s);
          setCpuHist((h) => pushMetric(h, s.cpuPercent));
          setRamHist((h) => pushMetric(h, s.ramUsedMb));
          setRamSysHist((h) => pushMetric(h, s.ramSystemUsedMb));
          setNetDownHist((h) => pushMetric(h, s.netDownBps));
          setNetUpHist((h) => pushMetric(h, s.netUpBps));
        })
        .catch(() => undefined);
    };
    tick();
    // Keep Host polling light so it doesn't contend with the client launcher
    const timer = setInterval(tick, 5000);
    return () => clearInterval(timer);
  }, [onHostRoute, id, status?.running, tab]);

  // Always poll while a server is selected — do not stop when UI thinks it is
  // Stopped, or a false negative freezes the badge until remount.
  useEffect(() => {
    if (!onHostRoute || !id) return;
    const tick = () => {
      api
        .dedicatedStatus(id)
        .then(setStatus)
        .catch(async (e) => {
          if (isMissingServerError(e)) await handleMissing();
        });
    };
    tick();
    const timer = setInterval(tick, 4000);
    return () => clearInterval(timer);
  }, [onHostRoute, id, handleMissing]);

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const server = await api.createDedicated({
        name: form.name,
        gameVersion: form.gameVersion,
        loader: form.loader,
        memoryMb: form.memoryMb,
        port: form.port,
      });
      setCreating(false);
      setForm({ name: "", gameVersion: "1.21.1", loader: "vanilla", memoryMb: 2048, port: 25565 });
      await refreshList();
      navigate(`/host/${server.id}`);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function runSafe<T>(fn: () => Promise<T>): Promise<T | undefined> {
    setBusy(true);
    setError(null);
    try {
      return await fn();
    } catch (e) {
      if (isMissingServerError(e)) await handleMissing();
      else setError(String(e));
      return undefined;
    } finally {
      setBusy(false);
    }
  }

  async function onInstall() {
    if (!selected) return;
    setInfo(t("hostInstalling"));
    const updated = await runSafe(() => api.installDedicated(selected.id));
    setInfo(null);
    if (updated) {
      setSelected(updated);
      await loadDetail(updated.id);
    }
  }

  async function onAcceptEula() {
    if (!selected) return;
    const updated = await runSafe(() => api.acceptDedicatedEula(selected.id));
    if (updated) setSelected(updated);
  }

  async function onStart() {
    if (!selected) return;
    const st = await runSafe(() => api.startDedicated(selected.id));
    if (st) setStatus(st);
  }

  async function onStop() {
    if (!selected) return;
    const st = await runSafe(() => api.stopDedicated(selected.id));
    if (st) setStatus(st);
  }

  async function onSaveMeta() {
    if (!selected) return;
    const updated = await runSafe(() => api.updateDedicated(selected));
    if (updated) {
      setSelected(updated);
      await refreshList();
    }
  }

  async function onSaveProps() {
    if (!selected || !props) return;
    await runSafe(() => api.setDedicatedProperties(selected.id, props));
    setInfo(t("saved"));
  }

  async function onSaveLists() {
    if (!selected) return;
    await runSafe(() => api.setDedicatedPlayerLists(selected.id, lists));
    setInfo(t("saved"));
  }

  async function onSendCommand(e: FormEvent) {
    e.preventDefault();
    if (!selected || !command.trim()) return;
    await runSafe(() => api.dedicatedSendCommand(selected.id, command));
    setCommand("");
  }

  async function onDelete() {
    if (!selected) return;
    if (!window.confirm(t("hostDeleteConfirm"))) return;
    await runSafe(() => api.deleteDedicated(selected.id));
    setSelected(null);
    await refreshList();
    navigate("/host");
  }

  function addWhitelist() {
    if (!listName.trim()) return;
    setLists({
      ...lists,
      whitelist: [
        ...lists.whitelist,
        { name: listName.trim(), uuid: listUuid.trim() || "00000000-0000-0000-0000-000000000000" },
      ],
    });
    setListName("");
    setListUuid("");
  }

  function addOp() {
    if (!listName.trim()) return;
    setLists({
      ...lists,
      ops: [
        ...lists.ops,
        {
          name: listName.trim(),
          uuid: listUuid.trim() || "00000000-0000-0000-0000-000000000000",
          level: 4,
          bypassesPlayerLimit: false,
        },
      ],
    });
    setListName("");
    setListUuid("");
  }

  function addBanPlayer() {
    if (!listName.trim()) return;
    setLists({
      ...lists,
      bannedPlayers: [
        ...lists.bannedPlayers,
        {
          name: listName.trim(),
          uuid: listUuid.trim() || "00000000-0000-0000-0000-000000000000",
          created: new Date().toISOString(),
          source: "Northstar",
          expires: "forever",
          reason: "Banned via Northstar",
        },
      ],
    });
    setListName("");
    setListUuid("");
  }

  function addBanIp() {
    if (!listIp.trim()) return;
    setLists({
      ...lists,
      bannedIps: [
        ...lists.bannedIps,
        {
          ip: listIp.trim(),
          created: new Date().toISOString(),
          source: "Northstar",
          expires: "forever",
          reason: "Banned via Northstar",
        },
      ],
    });
    setListIp("");
  }

  return (
    <VStack gap={4} className="euml-page">
      <Text type="display-3">{t("hostTitle")}</Text>
      <Text color="secondary">{t("hostHint")}</Text>
      {error && <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />}
      {info && <DismissibleBanner status="info" title={info} onDismiss={() => setInfo(null)} />}

      <HStack gap={2} style={{ flexWrap: "wrap" }}>
        <Button
          size="sm"
          label={t("hostCreate")}
          variant="primary"
          onClick={() => setCreating((v) => !v)}
        />
      </HStack>

      {creating && (
        <Card padding={4} className="euml-fade-in">
          <form onSubmit={onCreate}>
            <VStack gap={3}>
              <TextInput
                label={t("name")}
                value={form.name}
                onChange={(v) => setForm({ ...form, name: v })}
              />
              <HStack gap={2} style={{ flexWrap: "wrap" }}>
                <Selector
                  label={t("gameVersion")}
                  value={form.gameVersion}
                  onChange={(v) => setForm({ ...form, gameVersion: v })}
                  options={
                    versionOptions.length
                      ? versionOptions
                      : [{ value: form.gameVersion, label: form.gameVersion }]
                  }
                />
                <Selector
                  label={t("releases")}
                  value={versionFilter}
                  onChange={(v) => setVersionFilter(v as "release" | "snapshot" | "all")}
                  options={[
                    { value: "release", label: t("releases") },
                    { value: "snapshot", label: t("snapshots") },
                    { value: "all", label: t("all") },
                  ]}
                />
              </HStack>
              <Selector
                label={t("loader")}
                value={form.loader}
                onChange={(v) => setForm({ ...form, loader: v })}
                options={HOST_LOADERS.map((l) => ({ value: l, label: l }))}
              />
              <HStack gap={2}>
                <TextInput
                  label={t("hostMemory")}
                  value={String(form.memoryMb)}
                  onChange={(v) => setForm({ ...form, memoryMb: Number(v) || 2048 })}
                />
                <TextInput
                  label={t("hostPort")}
                  value={String(form.port)}
                  onChange={(v) => setForm({ ...form, port: Number(v) || 25565 })}
                />
              </HStack>
              <HStack gap={2}>
                <Button type="submit" label={t("create")} variant="primary" isDisabled={busy} />
                <Button label={t("cancel")} variant="secondary" onClick={() => setCreating(false)} />
              </HStack>
            </VStack>
          </form>
        </Card>
      )}

      <HStack gap={4} align="start" style={{ flexWrap: "wrap" }}>
        <Card padding={0} style={{ minWidth: 220, flex: "0 0 240px" }} className="euml-fade-in">
          {servers.length === 0 ? (
            <Text color="secondary" style={{ padding: 16 }}>
              {t("hostEmpty")}
            </Text>
          ) : (
            servers.map((s) => (
              <HStack
                key={s.id}
                justify="between"
                align="center"
                gap={2}
                style={{
                  padding: "10px 12px",
                  cursor: "pointer",
                  background: selected?.id === s.id ? "var(--astryx-color-bg-secondary, #f0f4fa)" : undefined,
                }}
                onClick={() => navigate(`/host/${s.id}`)}
              >
                <VStack gap={0} style={{ minWidth: 0, flex: 1 }}>
                  <HStack gap={1} align="center" style={{ minWidth: 0 }}>
                    <Text weight="semibold" style={{ overflow: "hidden", textOverflow: "ellipsis" }}>
                      {s.name}
                    </Text>
                    {selected?.id === s.id && status?.running && (
                      <span className="euml-host-status is-running" style={{ padding: "2px 8px", fontSize: 10 }}>
                        <span className="euml-host-status__dot" aria-hidden />
                        {t("hostRunning")}
                      </span>
                    )}
                  </HStack>
                  <Text color="secondary" type="supporting">
                    {s.gameVersion} · {s.loader} · :{s.port}
                  </Text>
                </VStack>
                <FavoriteButton
                  kind="dedicated"
                  itemKey={s.id}
                  label={s.name}
                  subtitle={`${s.gameVersion} · ${s.loader}`}
                />
              </HStack>
            ))
          )}
        </Card>

        {selected && (
          <VStack gap={3} style={{ flex: 1, minWidth: 280 }}>
            <Card padding={4} className="euml-fade-in">
              <VStack gap={3}>
                <HStack justify="between" align="center" style={{ flexWrap: "wrap" }} gap={2}>
                  <VStack gap={1}>
                    <HStack gap={2} align="center" style={{ flexWrap: "wrap" }}>
                      <Text type="display-3">{selected.name}</Text>
                      <span
                        className={`euml-host-status ${status?.running ? "is-running" : "is-stopped"}`}
                        title={status?.pid ? `PID ${status.pid}` : undefined}
                      >
                        <span className="euml-host-status__dot" aria-hidden />
                        {status?.running ? t("hostRunning") : t("hostStopped")}
                        {status?.pid ? ` · PID ${status.pid}` : ""}
                      </span>
                    </HStack>
                    <Text color="secondary">
                      {selected.gameVersion} · {selected.loader} · :{selected.port}
                      {" · "}
                      {selected.installed ? t("hostInstalled") : t("hostNotInstalled")}
                    </Text>
                  </VStack>
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <Button
                      label={t("hostOpenMods")}
                      variant="secondary"
                      onClick={() => void runSafe(() => api.openDedicatedFolder(selected.id))}
                    />
                    <Button label={t("delete")} variant="secondary" onClick={() => void onDelete()} />
                  </HStack>
                </HStack>

                {!selected.eulaAccepted && (
                  <VStack gap={2}>
                    <Text color="secondary">{t("hostEulaHint")}</Text>
                    <Button label={t("hostAcceptEula")} onClick={() => void onAcceptEula()} isDisabled={busy} />
                  </VStack>
                )}
                {selected.eulaAccepted && (
                  <Text color="secondary" type="supporting">
                    {t("hostEulaAccepted")}
                  </Text>
                )}

                <HStack gap={2} style={{ flexWrap: "wrap" }}>
                  <TextInput
                    label={t("name")}
                    value={selected.name}
                    onChange={(v) => setSelected({ ...selected, name: v })}
                  />
                  <TextInput
                    label={t("hostMemory")}
                    value={String(selected.memoryMb)}
                    onChange={(v) => setSelected({ ...selected, memoryMb: Number(v) || 2048 })}
                  />
                  <TextInput
                    label={t("hostPort")}
                    value={String(selected.port)}
                    onChange={(v) => setSelected({ ...selected, port: Number(v) || 25565 })}
                  />
                  <Button label={t("save")} onClick={() => void onSaveMeta()} isDisabled={busy} />
                </HStack>
              </VStack>
            </Card>

            <TabList value={tab} onChange={setTab}>
              <Tab value="console" label={t("hostConsole")} />
              <Tab value="live" label={t("hostLive")} />
              <Tab value="properties" label={t("hostProperties")} />
              <Tab value="lists" label={t("hostPlayerLists")} />
              {paperLike && <Tab value="plugins" label={t("hostPlugins")} />}
              <Tab value="files" label={t("hostFiles")} />
              <Tab value="network" label={t("hostNetwork")} />
              <Tab value="advanced" label={t("hostAdvanced")} />
            </TabList>

            {tab === "live" && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <Text>
                      {t("hostPlayersOnline")}: {live?.playersOnline ?? 0}
                      {live?.playersMax != null ? ` / ${live.playersMax}` : ""}
                    </Text>
                    <Text>
                      {t("hostTps")}: {live?.tps != null ? live.tps.toFixed(1) : "—"}
                    </Text>
                    <Text>
                      {t("hostMspt")}: {live?.mspt != null ? live.mspt.toFixed(1) : "—"}
                    </Text>
                    <Text>
                      {t("hostEntities")}: {live?.entityCount ?? "—"}
                    </Text>
                    <Text>
                      {t("hostMobs")}: {live?.mobCount ?? "—"}
                    </Text>
                  </HStack>
                  {live?.playerNames?.length ? (
                    <Text color="secondary">{live.playerNames.join(", ")}</Text>
                  ) : null}

                  <Text color="secondary" type="supporting">
                    {t("hostMetricsHint")}
                  </Text>
                  <HStack gap={3} style={{ flexWrap: "wrap", alignItems: "stretch" }}>
                    <MetricSparkline
                      label={t("hostCpuUsage")}
                      value={
                        live?.cpuPercent != null ? `${live.cpuPercent.toFixed(1)}%` : "—"
                      }
                      values={cpuHist}
                      color="#1370f0"
                    />
                    <MetricSparkline
                      label={t("hostRamUsage")}
                      value={formatMb(live?.ramUsedMb)}
                      values={ramHist}
                      color="#0f9d58"
                    />
                    <MetricSparkline
                      label={t("hostRamSystem")}
                      value={
                        live?.ramSystemUsedMb != null && live?.ramTotalMb != null
                          ? `${formatMb(live.ramSystemUsedMb)} / ${formatMb(live.ramTotalMb)}`
                          : formatMb(live?.ramSystemUsedMb)
                      }
                      values={ramSysHist}
                      color="#f4b400"
                    />
                    <MetricSparkline
                      label={t("hostNetDown")}
                      value={formatBps(live?.netDownBps)}
                      values={netDownHist}
                      color="#db4437"
                    />
                    <MetricSparkline
                      label={t("hostNetUp")}
                      value={formatBps(live?.netUpBps)}
                      values={netUpHist}
                      color="#ab47bc"
                    />
                  </HStack>

                  {live?.note ? <Text color="secondary">{live.note}</Text> : null}
                  <Button
                    label={t("hostRefreshStats")}
                    isDisabled={busy || !status?.running}
                    onClick={() =>
                      void runSafe(async () => {
                        const s = await api.dedicatedLiveStats(selected.id);
                        setLive(s);
                        setCpuHist((h) => pushMetric(h, s.cpuPercent));
                        setRamHist((h) => pushMetric(h, s.ramUsedMb));
                        setRamSysHist((h) => pushMetric(h, s.ramSystemUsedMb));
                        setNetDownHist((h) => pushMetric(h, s.netDownBps));
                        setNetUpHist((h) => pushMetric(h, s.netUpBps));
                        return s;
                      })
                    }
                  />
                </VStack>
              </Card>
            )}

            {tab === "console" && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <pre
                    ref={consoleRef}
                    className="euml-console-pre"
                    style={{
                      maxHeight: 320,
                      overflow: "auto",
                      margin: 0,
                      padding: 12,
                      fontSize: 12,
                      background: "var(--astryx-color-bg-secondary, #0f1419)",
                      color: "#d8e0ea",
                      borderRadius: 8,
                    }}
                  >
                    {hostLines.length === 0
                      ? t("consoleEmpty")
                      : hostLines.map((l) => `${l.ts} ${l.text}`).join("\n")}
                  </pre>
                  <form onSubmit={onSendCommand}>
                    <HStack gap={2} align="end">
                      <TextInput
                        label={t("hostCommand")}
                        value={command}
                        onChange={setCommand}
                        isDisabled={!status?.running}
                      />
                      <Button
                        type="submit"
                        label={t("hostSendCommand")}
                        isDisabled={!status?.running || busy}
                      />
                    </HStack>
                  </form>
                </VStack>
              </Card>
            )}

            {tab === "properties" && props && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <TextInput
                    label={t("hostMotd")}
                    value={props.motd}
                    onChange={(v) => setProps({ ...props, motd: v })}
                  />
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <TextInput
                      label={t("hostMaxPlayers")}
                      value={String(props.maxPlayers)}
                      onChange={(v) => setProps({ ...props, maxPlayers: Number(v) || 20 })}
                    />
                    <TextInput
                      label={t("difficulty")}
                      value={props.difficulty}
                      onChange={(v) => setProps({ ...props, difficulty: v })}
                    />
                    <TextInput
                      label={t("gameMode")}
                      value={props.gamemode}
                      onChange={(v) => setProps({ ...props, gamemode: v })}
                    />
                    <TextInput
                      label={t("hostViewDistance")}
                      value={String(props.viewDistance)}
                      onChange={(v) => setProps({ ...props, viewDistance: Number(v) || 10 })}
                    />
                    <TextInput
                      label={t("hostLevelName")}
                      value={props.levelName}
                      onChange={(v) => setProps({ ...props, levelName: v })}
                    />
                    <TextInput
                      label={t("hostPort")}
                      value={String(props.serverPort)}
                      onChange={(v) => setProps({ ...props, serverPort: Number(v) || 25565 })}
                    />
                  </HStack>
                  <CheckboxInput
                    label={t("hostOnlineMode")}
                    value={props.onlineMode}
                    onChange={(v) => setProps({ ...props, onlineMode: v })}
                  />
                  <CheckboxInput
                    label={t("hostWhiteList")}
                    value={props.whiteList}
                    onChange={(v) => setProps({ ...props, whiteList: v })}
                  />
                  <CheckboxInput
                    label={t("hostSpawnMonsters")}
                    value={props.spawnMonsters}
                    onChange={(v) => setProps({ ...props, spawnMonsters: v })}
                  />
                  <Button
                    label={t("hostSaveProperties")}
                    variant="primary"
                    isDisabled={busy}
                    onClick={() => void onSaveProps()}
                  />
                </VStack>
              </Card>
            )}

            {tab === "lists" && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <TextInput label={t("hostAddName")} value={listName} onChange={setListName} />
                    <TextInput label={t("hostAddUuid")} value={listUuid} onChange={setListUuid} />
                    <TextInput label={t("hostAddIp")} value={listIp} onChange={setListIp} />
                  </HStack>
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <Button size="sm" label={`${t("hostAdd")} ${t("hostWhitelist")}`} onClick={addWhitelist} />
                    <Button size="sm" label={`${t("hostAdd")} ${t("hostOps")}`} onClick={addOp} />
                    <Button size="sm" label={`${t("hostAdd")} ${t("hostBannedPlayers")}`} onClick={addBanPlayer} />
                    <Button size="sm" label={`${t("hostAdd")} ${t("hostBannedIps")}`} onClick={addBanIp} />
                  </HStack>

                  <Text weight="semibold">{t("hostWhitelist")}</Text>
                  {lists.whitelist.map((p, i) => (
                    <HStack key={`w-${i}`} justify="between">
                      <Text>
                        {p.name} ({p.uuid})
                      </Text>
                      <Button
                        size="sm"
                        label={t("delete")}
                        onClick={() =>
                          setLists({ ...lists, whitelist: lists.whitelist.filter((_, j) => j !== i) })
                        }
                      />
                    </HStack>
                  ))}

                  <Text weight="semibold">{t("hostOps")}</Text>
                  {lists.ops.map((p, i) => (
                    <HStack key={`o-${i}`} justify="between">
                      <Text>
                        {p.name} (lvl {p.level})
                      </Text>
                      <Button
                        size="sm"
                        label={t("delete")}
                        onClick={() => setLists({ ...lists, ops: lists.ops.filter((_, j) => j !== i) })}
                      />
                    </HStack>
                  ))}

                  <Text weight="semibold">{t("hostBannedPlayers")}</Text>
                  {lists.bannedPlayers.map((p, i) => (
                    <HStack key={`b-${i}`} justify="between">
                      <Text>{p.name}</Text>
                      <Button
                        size="sm"
                        label={t("delete")}
                        onClick={() =>
                          setLists({
                            ...lists,
                            bannedPlayers: lists.bannedPlayers.filter((_, j) => j !== i),
                          })
                        }
                      />
                    </HStack>
                  ))}

                  <Text weight="semibold">{t("hostBannedIps")}</Text>
                  {lists.bannedIps.map((p, i) => (
                    <HStack key={`i-${i}`} justify="between">
                      <Text>{p.ip}</Text>
                      <Button
                        size="sm"
                        label={t("delete")}
                        onClick={() =>
                          setLists({ ...lists, bannedIps: lists.bannedIps.filter((_, j) => j !== i) })
                        }
                      />
                    </HStack>
                  ))}

                  <Button
                    label={t("hostSaveLists")}
                    variant="primary"
                    isDisabled={busy}
                    onClick={() => void onSaveLists()}
                  />
                </VStack>
              </Card>
            )}

            {tab === "plugins" && paperLike && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <Text color="secondary">{t("hostPluginsHint")}</Text>
                  <form onSubmit={onSearchPlugins}>
                    <HStack gap={2} align="end" style={{ flexWrap: "wrap" }}>
                      <TextInput
                        label={t("hostHangarSearch")}
                        value={pluginQuery}
                        onChange={setPluginQuery}
                      />
                      <Button
                        type="submit"
                        label={t("search")}
                        variant="primary"
                        isDisabled={pluginSearching || busy}
                      />
                    </HStack>
                  </form>

                  {pluginHits.length > 0 && (
                    <VStack gap={2} style={{ alignItems: "stretch" }}>
                      <Text weight="semibold">{t("hostHangarResults")}</Text>
                      {pluginHits.map((hit) => (
                        <HStack
                          key={`${hit.author}/${hit.slug}`}
                          justify="between"
                          align="start"
                          gap={2}
                          style={{
                            padding: "8px 0",
                            borderBottom: "1px solid var(--astryx-color-border, #d0d7e2)",
                            cursor: "pointer",
                            background:
                              pluginPick?.slug === hit.slug && pluginPick?.author === hit.author
                                ? "var(--astryx-color-bg-secondary, #f0f4fa)"
                                : undefined,
                          }}
                          onClick={() => void onPickPlugin(hit)}
                        >
                          <VStack gap={0} style={{ minWidth: 0, flex: 1 }}>
                            <Text weight="semibold">{hit.name}</Text>
                            <Text color="secondary" type="supporting">
                              {hit.author}/{hit.slug}
                              {hit.downloads != null ? ` · ${hit.downloads.toLocaleString()} DL` : ""}
                            </Text>
                            <Text color="secondary" type="supporting">
                              {hit.description.slice(0, 160)}
                              {hit.description.length > 160 ? "…" : ""}
                            </Text>
                          </VStack>
                        </HStack>
                      ))}
                    </VStack>
                  )}

                  {pluginPick && (
                    <VStack gap={2} style={{ alignItems: "stretch" }}>
                      <Text weight="semibold">
                        {t("hostPluginInstall")}: {pluginPick.name}
                      </Text>
                      <Selector
                        label={t("hostPluginVersion")}
                        value={pluginVersion}
                        onChange={setPluginVersion}
                        options={[
                          { value: "latest", label: t("hostPluginLatest") },
                          ...pluginVersions.map((v) => ({ value: v.name, label: v.name })),
                        ]}
                      />
                      <Button
                        label={t("hostPluginInstall")}
                        variant="primary"
                        isDisabled={busy}
                        onClick={() => void onInstallPlugin()}
                      />
                    </VStack>
                  )}

                  <Text weight="semibold">{t("hostPluginsInstalled")}</Text>
                  {installedPlugins.length === 0 ? (
                    <Text color="secondary">{t("hostPluginsEmpty")}</Text>
                  ) : (
                    installedPlugins.map((p) => (
                      <HStack key={p.name} justify="between" align="center" gap={2}>
                        <Text>
                          {p.name}
                          {!p.enabled ? ` (${t("hostPluginDisabled")})` : ""}
                        </Text>
                        <HStack gap={2}>
                          <Button
                            size="sm"
                            label={p.enabled ? t("hostPluginDisable") : t("hostPluginEnable")}
                            onClick={() =>
                              void runSafe(async () => {
                                if (!selected) return;
                                const list = await api.dedicatedSetPluginEnabled(
                                  selected.id,
                                  p.name,
                                  !p.enabled,
                                );
                                setInstalledPlugins(list);
                                return list;
                              })
                            }
                          />
                          <Button
                            size="sm"
                            label={t("delete")}
                            onClick={() =>
                              void runSafe(async () => {
                                if (!selected) return;
                                if (!window.confirm(t("hostPluginDeleteConfirm"))) return;
                                const list = await api.dedicatedDeletePlugin(selected.id, p.name);
                                setInstalledPlugins(list);
                                return list;
                              })
                            }
                          />
                        </HStack>
                      </HStack>
                    ))
                  )}
                </VStack>
              </Card>
            )}

            {tab === "files" && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <Button
                      label={t("hostImportMrpack")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const path = await open({
                            multiple: false,
                            filters: [{ name: "mrpack", extensions: ["mrpack"] }],
                          });
                          if (!path || Array.isArray(path)) return;
                          const updated = await api.importDedicatedMrpack(selected.id, path);
                          setSelected(updated);
                          setInfo(t("saved"));
                          await loadDetail(updated.id);
                          return updated;
                        })
                      }
                    />
                    <Button
                      label={t("hostUploadWorld")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const path = await open({ directory: true, multiple: false });
                          if (!path || Array.isArray(path)) return;
                          const msg = await api.dedicatedUploadWorld(selected.id, path);
                          setInfo(msg);
                          return msg;
                        })
                      }
                    />
                    <Button
                      label={t("hostUploadMods")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const path = await open({
                            multiple: false,
                            directory: true,
                          });
                          if (!path || Array.isArray(path)) return;
                          const msg = await api.dedicatedUploadMods(selected.id, path);
                          setInfo(msg);
                          return msg;
                        })
                      }
                    />
                    <Button
                      label={t("hostDownloadWorld")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const path = await save({
                            defaultPath: `${selected.name}-world.zip`,
                            filters: [{ name: "zip", extensions: ["zip"] }],
                          });
                          if (!path) return;
                          const msg = await api.dedicatedDownloadWorld(selected.id, path);
                          setInfo(msg);
                          return msg;
                        })
                      }
                    />
                    <Button
                      label={t("hostDownloadMods")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const path = await save({
                            defaultPath: `${selected.name}-mods.zip`,
                            filters: [{ name: "zip", extensions: ["zip"] }],
                          });
                          if (!path) return;
                          const msg = await api.dedicatedDownloadMods(selected.id, path);
                          setInfo(msg);
                          return msg;
                        })
                      }
                    />
                  </HStack>
                </VStack>
              </Card>
            )}

            {tab === "network" && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <Text>
                    {t("hostJoinAddress")} (LAN): {net?.joinAddress ?? net?.lanIp ?? "—"}
                    {net?.joinAddress ? "" : net?.lanIp ? `:${net.port ?? selected.port}` : ""}
                  </Text>
                  <Text>
                    {t("hostWanJoin")}: {net?.wanJoinAddress ?? "—"}
                  </Text>
                  <Text>
                    {t("hostPublicIp")}: {net?.publicIp ?? "—"}
                  </Text>
                  <Text>
                    {t("hostLanIp")}: {net?.lanIp ?? "—"}
                  </Text>
                  <Text>
                    {t("hostPort")}: {net?.port ?? selected.port}
                  </Text>
                  <Text weight="semibold">{t("hostAdapters")}</Text>
                  {(net?.adapters ?? []).length === 0 ? (
                    <Text color="secondary">—</Text>
                  ) : (
                    (net?.adapters ?? []).map((a) => (
                      <Text key={`${a.name}-${a.ipv4}`}>
                        {a.name}: {a.ipv4}
                      </Text>
                    ))
                  )}
                  <Text>
                    {t("hostMapMethod")}:{" "}
                    {net?.mapMethod
                      ? net.mapMethod.toUpperCase()
                      : net?.upnpStatus === "mapped"
                        ? "—"
                        : net?.upnpStatus ?? "—"}
                  </Text>
                  <Text color="secondary">{net?.upnpMessage ?? ""}</Text>
                  <Text color="secondary">{net?.internetHint ?? ""}</Text>
                  {(net?.mapAttempts?.length ?? 0) > 0 && (
                    <VStack gap={1} style={{ alignItems: "stretch" }}>
                      <Text weight="semibold">{t("hostMapAttempts")}</Text>
                      {net!.mapAttempts!.map((a) => (
                        <Text key={a.method} color={a.ok ? undefined : "secondary"}>
                          {a.ok ? "✓" : "✗"} {a.method.toUpperCase()}: {a.message}
                        </Text>
                      ))}
                    </VStack>
                  )}
                  {net?.needsManual && (
                    <VStack gap={2} style={{ alignItems: "stretch" }}>
                      <Text weight="semibold">{t("hostManualForward")}</Text>
                      <Text color="secondary">{net.manualHint || t("hostManualForward")}</Text>
                      <Button label={t("hostUseRelay")} isDisabled />
                      <Text color="secondary">{net.relayHint || t("hostRelayUnavailable")}</Text>
                    </VStack>
                  )}
                  <Text color="secondary">{net?.firewallHint ?? ""}</Text>
                  <Text color="secondary">{net?.wlanHint ?? t("hostWlanHint")}</Text>
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <Button
                      label={t("hostUpnpMap")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const n = await api.dedicatedUpnpMap(selected.id);
                          setNet(n);
                          return n;
                        })
                      }
                    />
                    <Button
                      label={t("hostUpnpUnmap")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const n = await api.dedicatedUpnpUnmap(selected.id);
                          setNet(n);
                          return n;
                        })
                      }
                    />
                    <Button
                      label={t("hostFirewall")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const msg = await api.dedicatedFirewallRule(selected.id);
                          setInfo(msg);
                          return msg;
                        })
                      }
                    />
                    <Button
                      label={t("hostRefreshNetwork")}
                      isDisabled={busy}
                      onClick={() =>
                        void runSafe(async () => {
                          const n = await api.dedicatedNetworkInfo(selected.id);
                          setNet(n);
                          return n;
                        })
                      }
                    />
                  </HStack>
                </VStack>
              </Card>
            )}

            {tab === "advanced" && (
              <Card padding={4} className="euml-fade-in">
                <VStack gap={3}>
                  <Text weight="semibold">{t("hostCpuAffinity")}</Text>
                  <HStack gap={2} style={{ flexWrap: "wrap" }}>
                    <Button
                      size="sm"
                      label={t("hostCpuAll")}
                      onClick={() => setSelected({ ...selected, cpuAffinityMask: null })}
                    />
                    {Array.from({ length: Math.min(cpuCount, 32) }, (_, i) => {
                      const bit = 1n << BigInt(i);
                      const mask = BigInt(selected.cpuAffinityMask ?? 0);
                      const on = (mask & bit) !== 0n;
                      return (
                        <Button
                          key={i}
                          size="sm"
                          variant={on ? "primary" : "secondary"}
                          label={t("hostCpuCore", { n: i })}
                          onClick={() => {
                            const next = on ? mask & ~bit : mask | bit;
                            setSelected({
                              ...selected,
                              cpuAffinityMask: next === 0n ? null : Number(next),
                            });
                          }}
                        />
                      );
                    })}
                  </HStack>
                  <Button label={t("save")} onClick={() => void onSaveMeta()} isDisabled={busy} />
                </VStack>
              </Card>
            )}

            {/* Sticky bottom action bar */}
            <div
              style={{
                position: "sticky",
                bottom: 0,
                zIndex: 5,
                marginTop: 8,
                padding: "12px 14px",
                borderTop: "1px solid var(--astryx-color-border, #d0d7e2)",
                background: "var(--astryx-color-bg-primary, #fff)",
                boxShadow: "0 -6px 16px rgba(0,0,0,0.06)",
              }}
            >
              <HStack gap={2} align="center" style={{ flexWrap: "wrap" }}>
                <span
                  className={`euml-host-status ${status?.running ? "is-running" : "is-stopped"}`}
                  style={{ marginRight: 8 }}
                >
                  <span className="euml-host-status__dot" aria-hidden />
                  {status?.running
                    ? t("hostRunning")
                    : selected.installed
                      ? t("hostStopped")
                      : t("hostNotInstalled")}
                  {status?.pid ? ` · PID ${status.pid}` : ""}
                </span>
                {!selected.installed ? (
                  <Button
                    label={busy ? t("hostInstalling") : t("hostInstall")}
                    variant="primary"
                    isDisabled={busy}
                    onClick={() => void onInstall()}
                  />
                ) : (
                  <Button
                    label={busy ? t("hostInstalling") : t("hostReinstall")}
                    variant="secondary"
                    isDisabled={busy || !!status?.running}
                    onClick={() => void onInstall()}
                  />
                )}
                {!selected.eulaAccepted && (
                  <Button
                    label={t("hostAcceptEula")}
                    isDisabled={busy}
                    onClick={() => void onAcceptEula()}
                  />
                )}
                {status?.running ? (
                  <Button
                    label={t("hostStop")}
                    variant="secondary"
                    isDisabled={busy}
                    onClick={() => void onStop()}
                  />
                ) : (
                  <Button
                    label={t("hostStart")}
                    variant="primary"
                    isDisabled={busy || !selected.installed || !selected.eulaAccepted}
                    onClick={() => void onStart()}
                  />
                )}
              </HStack>
            </div>
          </VStack>
        )}
      </HStack>
    </VStack>
  );
}
