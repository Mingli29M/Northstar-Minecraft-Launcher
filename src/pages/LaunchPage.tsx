import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import { Link, useLocation } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { DismissibleBanner } from "../components/DismissibleBanner";
import { NewsPanel } from "../components/NewsPanel";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { CheckboxInput } from "@astryxdesign/core/CheckboxInput";
import { Text } from "@astryxdesign/core/Text";
import { Spinner } from "@astryxdesign/core/Spinner";
import { Selector } from "@astryxdesign/core/Selector";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { api } from "../lib/api";
import { AccountAvatar } from "../components/AccountAvatar";
import { loaderIconSrc } from "../lib/avatars";
import { effectiveLoader } from "../lib/loaderDetect";
import { useFavorites } from "../lib/favorites";
import { FavoriteButton } from "../components/FavoriteButton";
import { preferredInstanceId, rememberPreferredInstance } from "../lib/preferredInstance";
import { SETTINGS_EVENT } from "../lib/appearance";
import { getSettingsSnapshot, subscribeSettings } from "../lib/settingsStore";
import { favoriteId } from "../lib/types";
import { useI18n } from "../i18n";
import type {
  Account,
  CrashHint,
  GameExitAnalysis,
  Instance,
  InstanceFolder,
  LauncherSettings,
  ReqIssue,
  ReqScanResult,
} from "../lib/types";
import { useDownloadStatus } from "../lib/downloadStatus";

/** Mirrors backend `issue_is_installable` — hide Install for breaks / version mismatches. */
function isInstallableReqIssue(issue: ReqIssue): boolean {
  if (issue.severity !== "error") return false;
  const msg = issue.message.toLowerCase();
  if (
    msg.includes("breaks ") ||
    msg.includes(", found ") ||
    msg.includes("incompatible") ||
    msg.includes("requires minecraft")
  ) {
    return false;
  }
  if (issue.project_id?.trim()) return true;
  const missing = issue.missing_mod_id?.trim();
  if (!missing) return false;
  return (
    msg.includes("missing") ||
    msg.includes("depends on") ||
    msg.includes("not installed") ||
    msg.includes(" — install")
  );
}

export function LaunchPage() {
  const { t } = useI18n();
  const { pathname } = useLocation();
  const { progress, appendConsole } = useDownloadStatus();
  const { isFavorite, favoritesOf } = useFavorites();
  const [instances, setInstances] = useState<Instance[]>([]);
  const [folders, setFolders] = useState<InstanceFolder[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [settings, setSettings] = useState<LauncherSettings | null>(null);
  const [selectedId, setSelectedId] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [scan, setScan] = useState<ReqScanResult | null>(null);
  const [scanBusy, setScanBusy] = useState(false);
  const [fixBusy, setFixBusy] = useState(false);
  const [fixError, setFixError] = useState<string | null>(null);
  const [crashes, setCrashes] = useState<CrashHint[]>([]);
  const [exitAnalysis, setExitAnalysis] = useState<GameExitAnalysis | null>(null);
  const [override, setOverride] = useState(false);
  const [gameRunning, setGameRunning] = useState(false);
  const [stopBusy, setStopBusy] = useState(false);
  // Live snapshot beats local/disk races (Settings autosave vs Launch reload).
  const liveSettings = useSyncExternalStore(
    subscribeSettings,
    getSettingsSnapshot,
    getSettingsSnapshot,
  );
  const layout = liveSettings ?? settings;
  const onlySelected = Boolean(layout?.launch_only_selected);
  // Honour Settings → Start position in both full and compact Launch layouts.
  const startAtBottom =
    String(layout?.launch_start_position ?? "top").trim().toLowerCase() ===
    "bottom";

  const busyLabel = useMemo(() => {
    if (!busy) return t("startGame");
    if (status === t("launching")) return t("launching");
    if (progress?.active && progress.message) {
      const phase = progress.phase;
      if (phase === "assets" && progress.total > 0) {
        return `${t("preparingAssets")} ${progress.done}/${progress.total}`;
      }
      if (phase === "libraries" || phase === "libs") {
        return progress.total > 0
          ? `${t("preparingLibraries")} ${progress.done}/${progress.total}`
          : t("preparingLibraries");
      }
      return progress.message;
    }
    return status ?? t("preparing");
  }, [busy, status, progress, t]);

  const selected = useMemo(() => instances.find((i) => i.id === selectedId) ?? null, [instances, selectedId]);
  const activeAccount = accounts.find((a) => a.active) ?? accounts[0] ?? null;

  const versionOptions = useMemo(() => {
    const favIds = new Set(favoritesOf("instance").map((f) => f.id));
    const sorted = [...instances].sort((a, b) => {
      const af = favIds.has(favoriteId("instance", a.id)) ? 0 : 1;
      const bf = favIds.has(favoriteId("instance", b.id)) ? 0 : 1;
      return af - bf;
    });
    return sorted.map((i) => {
      const loader = effectiveLoader(i);
      const star = isFavorite(favoriteId("instance", i.id)) ? "★ " : "";
      return {
        value: i.id,
        label: `${star}${i.name}  ·  ${i.game_version} · ${loader}`,
      };
    });
  }, [instances, isFavorite, favoritesOf]);

  const grouped = useMemo(() => {
    const sections: { key: string; title: string; items: Instance[] }[] = [];
    const favItems = instances.filter((i) => isFavorite(favoriteId("instance", i.id)));
    if (favItems.length) {
      sections.push({ key: "favorites", title: t("favorites"), items: favItems });
    }
    const root = instances.filter((i) => !i.folder);
    if (root.length) sections.push({ key: "root", title: t("uncategorized"), items: root });
    for (const f of folders) {
      const items = instances.filter((i) => i.folder === f.id);
      if (items.length) sections.push({ key: f.id, title: f.name, items });
    }
    const known = new Set(folders.map((f) => f.id));
    const orphan = instances.filter((i) => i.folder && !known.has(i.folder));
    if (orphan.length) sections.push({ key: "orphan", title: t("uncategorized"), items: orphan });
    return sections;
  }, [instances, folders, t, isFavorite]);

  const loadGen = useRef(0);
  const reload = useCallback(async () => {
    const gen = ++loadGen.current;
    const [list, folderList, acc, st] = await Promise.all([
      api.listInstances(),
      api.listFolders(),
      api.listAccounts(),
      api.getSettings(),
    ]);
    if (gen !== loadGen.current) return;
    setInstances(list);
    setFolders(folderList);
    setAccounts(acc);
    // Prefer live snapshot / in-memory layout flags when disk is still catching
    // up to the Settings autosave debounce — otherwise Start position snaps back.
    const snap = getSettingsSnapshot();
    setSettings((prev) => ({
      ...st,
      launch_start_position:
        snap?.launch_start_position ??
        prev?.launch_start_position ??
        st.launch_start_position,
      launch_only_selected:
        snap?.launch_only_selected ??
        prev?.launch_only_selected ??
        st.launch_only_selected,
    }));
    setSelectedId((prev) =>
      prev && list.some((i) => i.id === prev) ? prev : preferredInstanceId(list, st),
    );
  }, []);

  // This pane is kept mounted while other pages are visited, so an instance
  // created elsewhere (a download, an import) is only picked up if we reload
  // whenever the page comes back into view.
  const visible = pathname === "/";
  useEffect(() => {
    if (!visible) return;
    let cancelled = false;
    void reload().catch((e) => {
      if (!cancelled) setError(String(e));
    });
    return () => {
      cancelled = true;
    };
  }, [visible, reload]);

  // Settings → Appearance (compact / Start position) must update this keep-alive
  // pane immediately, not only on the next remount.
  useEffect(() => {
    const onSettings = (e: Event) => {
      const detail = (e as CustomEvent<LauncherSettings>).detail;
      if (!detail) return;
      setSettings((prev) => {
        if (
          prev &&
          prev.launch_only_selected === detail.launch_only_selected &&
          prev.launch_start_position === detail.launch_start_position &&
          prev.reqguard_local_scan === detail.reqguard_local_scan &&
          prev.reqguard_deep_validation === detail.reqguard_deep_validation
        ) {
          return prev;
        }
        return detail;
      });
    };
    window.addEventListener(SETTINGS_EVENT, onSettings);
    return () => window.removeEventListener(SETTINGS_EVENT, onSettings);
  }, []);

  useEffect(() => {
    if (!selectedId) return;
    void rememberPreferredInstance(selectedId).then(() => {
      setSettings((prev) =>
        prev && prev.last_instance_id !== selectedId
          ? { ...prev, last_instance_id: selectedId }
          : prev,
      );
    });
  }, [selectedId]);

  useEffect(() => {
    if (!selected) {
      setScan(null);
      setCrashes([]);
      setExitAnalysis(null);
      return;
    }
    setExitAnalysis(null);
    let cancelled = false;
    const timer = window.setTimeout(() => {
      // Compact mode hides the ReqGuard and crash panels. When both scan modes
      // are off in Settings, skip the work entirely — Launch is still gated by
      // the backend check when local scan is enabled.
      const localOn = settings?.reqguard_local_scan === true;
      const deepOn = settings?.reqguard_deep_validation !== false;
      if (!onlySelected && (localOn || deepOn)) {
        setScanBusy(true);
        api
          .reqguardScan(selected.id)
          .then((s) => {
            if (!cancelled) setScan(s);
          })
          .catch(() => {
            if (!cancelled) setScan(null);
          })
          .finally(() => {
            if (!cancelled) setScanBusy(false);
          });
      } else {
        setScan(null);
        setScanBusy(false);
      }
      if (!onlySelected) {
        api
          .analyzeCrash(selected.id)
          .then((c) => {
            if (!cancelled) setCrashes(c);
          })
          .catch(() => {
            if (!cancelled) setCrashes([]);
          });
      }
      api
        .lastGameExitAnalysis(selected.id)
        .then((analysis) => {
          if (!cancelled) setExitAnalysis(analysis);
        })
        .catch(() => {
          if (!cancelled) setExitAnalysis(null);
        });
    }, 50);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [selected, onlySelected, settings?.reqguard_local_scan, settings?.reqguard_deep_validation]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<GameExitAnalysis>("euml:game-exit-analysis", (event) => {
      const analysis = event.payload;
      if (analysis.instance_id !== selected?.id) return;
      setExitAnalysis(analysis);
      setCrashes(analysis.hints);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [selected?.id]);

  async function rerunReqguard() {
    if (!selected || scanBusy || fixBusy) return;
    setScanBusy(true);
    setFixError(null);
    try {
      setScan(await api.reqguardScan(selected.id));
    } finally {
      setScanBusy(false);
    }
  }

  async function fixMissing(missingModId: string, projectId?: string | null) {
    if (!selected || fixBusy) return;
    setFixBusy(true);
    setFixError(null);
    try {
      setScan(await api.reqguardResolve(selected.id, missingModId, projectId));
    } catch (e) {
      const msg = String(e);
      setFixError(msg);
      appendConsole(`${t("reqguardFixFailed")}: ${msg}`, "error");
    } finally {
      setFixBusy(false);
    }
  }

  async function fixAllMissing() {
    if (!selected || fixBusy) return;
    setFixBusy(true);
    setFixError(null);
    try {
      setScan(await api.reqguardResolveAll(selected.id));
    } catch (e) {
      const msg = String(e);
      setFixError(msg);
      appendConsole(`${t("reqguardFixFailed")}: ${msg}`, "error");
    } finally {
      setFixBusy(false);
    }
  }

  async function onLaunch(instanceId?: string) {
    const id = instanceId ?? selectedId;
    const inst = instances.find((i) => i.id === id) ?? null;
    const baseSettings = getSettingsSnapshot() ?? settings;
    if (!inst || !baseSettings || gameRunning) return;
    if (id !== selectedId) setSelectedId(id);
    setBusy(true);
    setError(null);
    setStatus(t("preparing"));
    try {
      await api.saveSettings({ ...baseSettings, last_instance_id: inst.id });
      await api.prepareInstance(inst.id);
      setStatus(t("launching"));
      setStatus(await api.launchInstance(inst.id, override));
      const run = await api.gameRunState(inst.id);
      setGameRunning(Boolean(run.running));
      setInstances(await api.listInstances());
    } catch (e) {
      const msg = String(e);
      setError(msg);
      setStatus(null);
      appendConsole(msg, "error");
      try {
        const run = await api.gameRunState(inst.id);
        setGameRunning(Boolean(run.running));
      } catch {
        setGameRunning(false);
      }
    } finally {
      setBusy(false);
    }
  }

  async function onStop() {
    if (!selectedId || stopBusy || !gameRunning) return;
    setStopBusy(true);
    setError(null);
    try {
      setStatus(await api.stopInstance(selectedId));
      setGameRunning(false);
    } catch (e) {
      const msg = String(e);
      setError(msg);
      appendConsole(msg, "error");
    } finally {
      setStopBusy(false);
    }
  }

  useEffect(() => {
    if (!selectedId) {
      setGameRunning(false);
      return;
    }
    let cancelled = false;
    void api.gameRunState(selectedId).then((s) => {
      if (!cancelled) setGameRunning(Boolean(s.running));
    });
    return () => {
      cancelled = true;
    };
  }, [selectedId]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<{ instanceId: string; running: boolean }>("euml:game-state", (ev) => {
      if (ev.payload.instanceId === selectedId) {
        setGameRunning(Boolean(ev.payload.running));
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [selectedId]);

  function renderLaunchActions() {
    return (
      <div className="euml-launch-actions">
        <button
          type="button"
          className="euml-start-btn"
          disabled={!selected || busy || gameRunning}
          onClick={() => void onLaunch()}
        >
          {busy ? (
            <HStack gap={2} align="center" justify="center">
              <Spinner size="sm" />
              <span>{busyLabel}</span>
            </HStack>
          ) : (
            t("startGame")
          )}
        </button>
        <button
          type="button"
          className="euml-stop-btn"
          disabled={!selected || !gameRunning || stopBusy}
          onClick={() => void onStop()}
        >
          {stopBusy ? t("stoppingGame") : t("stopGame")}
        </button>
      </div>
    );
  }

  return (
    <HStack gap={5} align="stretch" className="euml-page" style={{ minHeight: "100%" }}>
      <VStack gap={4} style={{ flex: 1, minWidth: 0 }}>
        <HStack justify="between" align="center" gap={4}>
          <Text type="display-3">{t("launchTitle")}</Text>
          <HStack gap={2} align="center">
            {activeAccount && <AccountAvatar account={activeAccount} sizeHint="sm" />}
            <Text color="secondary" type="body">
              {activeAccount ? (
                <>
                  {t("currentAccount")}: <Text weight="semibold">{activeAccount.username}</Text>{" "}
                  <Link to="/accounts">{t("switchAccount")}</Link>
                </>
              ) : (
                <Link to="/accounts">{t("addAccount")}</Link>
              )}
            </Text>
          </HStack>
        </HStack>

        {/* PCL / HMCL style: version select + big Start */}
        <Card
          padding={4}
          className="euml-launch-hero"
          style={
            onlySelected
              ? { flex: 1, display: "flex", flexDirection: "column" }
              : undefined
          }
        >
          <VStack gap={4} style={onlySelected ? { flex: 1, minHeight: 0 } : undefined}>
            <HStack gap={4} align="end">
              <div style={{ flex: 1, minWidth: 220 }}>
                <Selector
                  label={t("selectVersion")}
                  value={selectedId}
                  onChange={setSelectedId}
                  options={
                    versionOptions.length
                      ? versionOptions
                      : [{ value: "", label: t("noVersions") }]
                  }
                />
              </div>
              {selected && (
                <HStack gap={2} align="center" style={{ paddingBottom: 4 }}>
                  {selected.icon_path ? (
                    <img src={selected.icon_path} alt="" className="euml-avatar" />
                  ) : (
                    <img
                      src={loaderIconSrc(effectiveLoader(selected))}
                      alt={effectiveLoader(selected)}
                      className="euml-avatar"
                    />
                  )}
                  <VStack gap={0}>
                    <Text weight="semibold">{selected.name}</Text>
                    <Text color="secondary" type="supporting">
                      {selected.game_version} · {effectiveLoader(selected)}
                    </Text>
                  </VStack>
                  <FavoriteButton
                    kind="instance"
                    itemKey={selected.id}
                    label={selected.name}
                    subtitle={`${selected.game_version} · ${effectiveLoader(selected)}`}
                    iconUrl={selected.icon_path}
                  />
                </HStack>
              )}
            </HStack>
            {!startAtBottom && renderLaunchActions()}
            <HStack gap={4} align="center">
              <CheckboxInput label={t("overrideReq")} value={override} onChange={setOverride} />
              {!onlySelected && selected && (
                <Link to={`/versions/${selected.id}`}>
                  <Text color="accent">{t("versionSettings")}</Text>
                </Link>
              )}
              {!onlySelected && (
                <Link to="/news">
                  <Text color="accent">{t("navNews")}</Text>
                </Link>
              )}
            </HStack>
            {startAtBottom && onlySelected && (
              <div style={{ marginTop: "auto", width: "100%" }}>{renderLaunchActions()}</div>
            )}
          </VStack>
        </Card>

        {!onlySelected && (
        <Card
          padding={0}
          style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column", minHeight: 0 }}
        >
          <div style={{ flex: 1, overflow: "auto" }}>
            {grouped.length === 0 && (
              <div style={{ padding: 16 }}>
                <Text color="secondary">{t("noVersions")}</Text>
              </div>
            )}
            {grouped.map((section) => (
              <div key={section.key}>
                <div className="euml-folder-label">{section.title}</div>
                {section.items.map((inst) => {
                  const loader = effectiveLoader(inst);
                  return (
                    <div
                      key={`${section.key}-${inst.id}`}
                      className={`euml-list-row${selectedId === inst.id ? " is-selected" : ""}`}
                      style={{ cursor: "pointer" }}
                      onClick={() => setSelectedId(inst.id)}
                      onDoubleClick={() => void onLaunch(inst.id)}
                    >
                      {inst.icon_path ? (
                        <img src={inst.icon_path} alt="" className="euml-avatar" />
                      ) : (
                        <img src={loaderIconSrc(loader)} alt={loader} className="euml-avatar" />
                      )}
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <span className="euml-list-row__title">{inst.name}</span>
                        <span className="euml-list-row__meta">
                          {inst.game_version} · {loader}
                        </span>
                      </div>
                      <span className="euml-list-row__meta">
                        {inst.last_played ? t("played") : t("neverPlayed")}
                      </span>
                      <FavoriteButton
                        kind="instance"
                        itemKey={inst.id}
                        label={inst.name}
                        subtitle={`${inst.game_version} · ${loader}`}
                        iconUrl={inst.icon_path}
                      />
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        </Card>
        )}

        {/* Bottom Start: always after the version list (full) / hero (compact). */}
        {startAtBottom && !onlySelected && (
          <div className="euml-launch-actions-dock">{renderLaunchActions()}</div>
        )}

        {status && <DismissibleBanner status="info" title={status} onDismiss={() => setStatus(null)} />}
        {error && <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />}
        {exitAnalysis && (
          <DismissibleBanner
            status="error"
            title={`${t("gameExitAnalysis")}: ${exitAnalysis.summary}`}
            onDismiss={() => setExitAnalysis(null)}
          />
        )}
      </VStack>

      {!onlySelected && (
      <VStack gap={3} style={{ width: 320, flexShrink: 0 }}>
        <Card padding={3}>
          <HStack justify="between" align="center" style={{ marginBottom: 8 }}>
            <Text weight="semibold" display="block">
              {t("reqguard")}
            </Text>
            {selected && (
              <Button
                label={t("rerunReqguard")}
                size="sm"
                variant="secondary"
                isDisabled={scanBusy}
                onClick={() => void rerunReqguard()}
              />
            )}
          </HStack>
          {!selected && <Text color="secondary">{t("reqguardPick")}</Text>}
          {selected && (scanBusy || fixBusy) && (
            <HStack gap={2} align="center">
              <Spinner size="sm" />
              <Text color="secondary" type="supporting">
                {fixBusy ? t("installingMissing") : t("reqguardScanning")}
              </Text>
            </HStack>
          )}
          {fixError && (
            <DismissibleBanner
              status="error"
              title={fixError}
              onDismiss={() => setFixError(null)}
            />
          )}
          {selected &&
            scan &&
            !scanBusy &&
            !fixBusy &&
            !scan.local_scan &&
            !scan.deep_scan &&
            scan.issues.length === 0 && (
              <Text color="secondary" type="supporting">
                {t("reqguardModesIdle")}
              </Text>
            )}
          {selected && scan && scan.issues.length === 0 && (scan.local_scan || scan.deep_scan) && (
            <Text color="accent">{t("reqguardOk", { count: scan.mod_count })}</Text>
          )}
          {selected && scan && scan.issues.some(isInstallableReqIssue) && (
            <Button
              label={t("installAllMissing")}
              size="sm"
              variant="primary"
              style={{ marginBottom: 8 }}
              isDisabled={fixBusy || scanBusy}
              onClick={() => void fixAllMissing()}
            />
          )}
          {scan?.issues.slice(0, 8).map((issue, i) => (
            <VStack key={i} gap={1} style={{ marginBottom: 8 }}>
              <Text type="supporting">
                {issue.source ? `[${issue.source}] ` : ""}
                {issue.message}
              </Text>
              {selected && isInstallableReqIssue(issue) && (
                <Button
                  label={`${t("installMissing")}: ${issue.missing_mod_id || issue.project_id}`}
                  size="sm"
                  variant="secondary"
                  isDisabled={fixBusy || scanBusy}
                  onClick={() =>
                    void fixMissing(
                      issue.missing_mod_id || issue.project_id!,
                      issue.project_id,
                    )
                  }
                />
              )}
            </VStack>
          ))}
        </Card>
        <Card padding={3}>
          <Text weight="semibold" display="block" style={{ marginBottom: 8 }}>
            {t("crashAnalysis")}
          </Text>
          {crashes.length === 0 && <Text color="secondary">{t("crashNone")}</Text>}
          {crashes.map((c, i) => (
            <VStack key={i} gap={0.5} style={{ marginBottom: 8 }}>
              <Text weight="semibold">{c.title}</Text>
              <Text color="secondary" type="supporting">
                {c.detail}
              </Text>
            </VStack>
          ))}
        </Card>
        <Card padding={3} style={{ maxHeight: 360, overflow: "auto" }}>
          <NewsPanel compact />
        </Card>
      </VStack>
      )}
    </HStack>
  );
}
