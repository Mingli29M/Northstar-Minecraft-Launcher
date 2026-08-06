import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
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
import { favoriteId } from "../lib/types";
import { useI18n } from "../i18n";
import type { Account, CrashHint, GameExitAnalysis, Instance, InstanceFolder, LauncherSettings, ReqScanResult } from "../lib/types";
import { useDownloadStatus } from "../lib/downloadStatus";

export function LaunchPage() {
  const { t } = useI18n();
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
  const [crashes, setCrashes] = useState<CrashHint[]>([]);
  const [exitAnalysis, setExitAnalysis] = useState<GameExitAnalysis | null>(null);
  const [override, setOverride] = useState(false);

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

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const [list, folderList, acc, st] = await Promise.all([
        api.listInstances(),
        api.listFolders(),
        api.listAccounts(),
        api.getSettings(),
      ]);
      if (cancelled) return;
      setInstances(list);
      setFolders(folderList);
      setAccounts(acc);
      setSettings(st);
      setSelectedId(preferredInstanceId(list, st));
    })().catch((e) => {
      if (!cancelled) setError(String(e));
    });
    return () => {
      cancelled = true;
    };
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
      api
        .analyzeCrash(selected.id)
        .then((c) => {
          if (!cancelled) setCrashes(c);
        })
        .catch(() => {
          if (!cancelled) setCrashes([]);
        });
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
  }, [selected]);

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
    if (!selected || scanBusy) return;
    setScanBusy(true);
    try {
      setScan(await api.reqguardScan(selected.id));
    } finally {
      setScanBusy(false);
    }
  }

  async function onLaunch(instanceId?: string) {
    const id = instanceId ?? selectedId;
    const inst = instances.find((i) => i.id === id) ?? null;
    if (!inst || !settings) return;
    if (id !== selectedId) setSelectedId(id);
    setBusy(true);
    setError(null);
    setStatus(t("preparing"));
    try {
      await api.saveSettings({ ...settings, last_instance_id: inst.id });
      await api.prepareInstance(inst.id);
      setStatus(t("launching"));
      setStatus(await api.launchInstance(inst.id, override));
      setInstances(await api.listInstances());
    } catch (e) {
      const msg = String(e);
      setError(msg);
      setStatus(null);
      appendConsole(msg, "error");
    } finally {
      setBusy(false);
    }
  }

  return (
    <HStack gap={5} align="stretch" style={{ minHeight: "100%" }} className="euml-page">
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
        <Card padding={4} className="euml-launch-hero">
          <VStack gap={4}>
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
            <button
              type="button"
              className="euml-start-btn"
              disabled={!selected || busy}
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
            <HStack gap={4} align="center">
              <CheckboxInput label={t("overrideReq")} value={override} onChange={setOverride} />
              {selected && (
                <Link to={`/versions/${selected.id}`}>
                  <Text color="accent">{t("versionSettings")}</Text>
                </Link>
              )}
              <Link to="/news">
                <Text color="accent">{t("navNews")}</Text>
              </Link>
            </HStack>
          </VStack>
        </Card>

        <Card padding={0} style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column" }}>
          <div style={{ flex: 1, overflow: "auto" }}>
            {instances.length === 0 && (
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
          {selected && scanBusy && (
            <HStack gap={2} align="center">
              <Spinner size="sm" />
              <Text color="secondary" type="supporting">{t("reqguardScanning")}</Text>
            </HStack>
          )}
          {selected && scan && scan.issues.length === 0 && (
            <Text color="accent">{t("reqguardOk", { count: scan.mod_count })}</Text>
          )}
          {selected && scan && scan.issues.some((i) => i.severity === "error") && (
            <Button
              label={t("installAllMissing")}
              size="sm"
              variant="primary"
              style={{ marginBottom: 8 }}
              onClick={async () => setScan(await api.reqguardResolveAll(selected.id))}
            />
          )}
          {scan?.issues.slice(0, 8).map((issue, i) => (
            <VStack key={i} gap={1} style={{ marginBottom: 8 }}>
              <Text type="supporting">
                {issue.source ? `[${issue.source}] ` : ""}
                {issue.message}
              </Text>
              {(issue.project_id || issue.missing_mod_id) && selected && (
                <Button
                  label={`${t("installMissing")}: ${issue.project_id || issue.missing_mod_id}`}
                  size="sm"
                  variant="secondary"
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
    </HStack>
  );
}
