import { useEffect, useMemo, useState } from "react";
import { createPortal } from "react-dom";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { Spinner } from "@astryxdesign/core/Spinner";
import { api } from "../lib/api";
import { useDownloadStatus } from "../lib/downloadStatus";
import { useI18n } from "../i18n";
import type { ModrinthDependency, ModrinthProjectDetails, ModrinthVersion } from "../lib/types";

function humanBytes(n: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let value = n;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return unit === 0 ? `${Math.round(value)} B` : `${value.toFixed(1)} ${units[unit]}`;
}

export type ModInstallKind = "mod" | "modpack" | "resourcepack" | "shader" | "datapack";

type DepPickerTarget = {
  projectId: string;
  title: string;
  kind: ModInstallKind;
  initialVersionId?: string | null;
};

type Props = {
  projectId: string;
  title: string;
  gameVersion: string;
  loader: string;
  instanceId: string;
  kind: ModInstallKind;
  /** Optional preloaded details — skips an extra fetch when already on the detail page. */
  details?: ModrinthProjectDetails | null;
  /** Prefer this version when the list loads. */
  initialVersionId?: string | null;
  onClose: () => void;
  onInstalled: (info: { label: string; instanceId?: string }) => void;
};

function kindFromProjectType(projectType: string | undefined): ModInstallKind {
  switch (projectType) {
    case "modpack":
    case "resourcepack":
    case "shader":
    case "datapack":
      return projectType;
    default:
      return "mod";
  }
}

function primaryFile(v: ModrinthVersion) {
  return v.files.find((f) => f.primary) ?? v.files[0];
}

function isVersionInstalled(
  v: ModrinthVersion,
  installedIds: Set<string>,
  installedFiles: Set<string>,
): boolean {
  if (installedIds.has(v.id)) return true;
  const file = primaryFile(v);
  return !!(file && installedFiles.has(file.filename));
}

function depLabel(d: ModrinthDependency): string {
  return d.project_title || d.project_slug || d.project_id || d.version_id || "dependency";
}

export function ModInstallPicker({
  projectId,
  title,
  gameVersion,
  loader,
  instanceId,
  kind,
  details: preloaded,
  initialVersionId,
  onClose,
  onInstalled,
}: Props) {
  const { t } = useI18n();
  const { progress } = useDownloadStatus();
  const [details, setDetails] = useState<ModrinthProjectDetails | null>(preloaded ?? null);
  const [loading, setLoading] = useState(!preloaded);
  const [error, setError] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string>("");
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set());
  const [installedFiles, setInstalledFiles] = useState<Set<string>>(new Set());
  const [installing, setInstalling] = useState(false);
  const [depPicker, setDepPicker] = useState<DepPickerTarget | null>(null);
  const [statusNote, setStatusNote] = useState<string | null>(null);

  const bytesTotal = progress?.bytesTotal ?? null;
  const bytesDone = progress?.bytesDone ?? null;
  const byteMode = bytesTotal != null && bytesTotal > 0 && bytesDone != null;
  const progressPct = byteMode
    ? Math.min(100, Math.round((bytesDone! / bytesTotal!) * 100))
    : progress && progress.total > 0
      ? Math.min(100, Math.round((progress.done / progress.total) * 100))
      : installing
        ? undefined
        : 0;
  const progressLabel = installing
    ? progress?.message || t("installing")
    : null;
  const progressCounter = installing
    ? byteMode
      ? `${humanBytes(bytesDone!)} / ${humanBytes(bytesTotal!)}`
      : progress && progress.total > 0
        ? `${progress.done}/${progress.total}`
        : ""
    : "";

  useEffect(() => {
    let cancelled = false;
    if (preloaded) {
      setDetails(preloaded);
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    api
      .getModrinthProject(projectId, gameVersion, loader)
      .then((d) => {
        if (!cancelled) setDetails(d);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [projectId, gameVersion, loader, preloaded]);

  function refreshInstalledMarkers() {
    if (!instanceId || kind === "modpack") {
      setInstalledIds(new Set());
      setInstalledFiles(new Set());
      return;
    }
    api
      .installedModrinthMarkers(instanceId)
      .then((m) => {
        setInstalledIds(new Set(m.version_ids));
        setInstalledFiles(new Set(m.filenames));
      })
      .catch(() => {
        setInstalledIds(new Set());
        setInstalledFiles(new Set());
      });
  }

  useEffect(() => {
    refreshInstalledMarkers();
    // eslint-disable-next-line react-hooks/exhaustive-deps -- refresh on instance/kind only
  }, [instanceId, kind]);

  const versions = details?.versions ?? [];

  useEffect(() => {
    if (!versions.length) {
      setSelectedId("");
      return;
    }
    setSelectedId((prev) => {
      if (prev && versions.some((v) => v.id === prev)) return prev;
      if (initialVersionId && versions.some((v) => v.id === initialVersionId)) {
        return initialVersionId;
      }
      return versions[0].id;
    });
  }, [versions, initialVersionId]);

  const selected = useMemo(
    () => versions.find((v) => v.id === selectedId) ?? null,
    [versions, selectedId],
  );

  const prereqs = useMemo(() => {
    const deps = selected?.dependencies ?? [];
    const required = deps.filter((d) => d.dependency_type.toLowerCase() === "required");
    const optional = deps.filter((d) => d.dependency_type.toLowerCase() === "optional");
    const incompatible = deps.filter((d) => d.dependency_type.toLowerCase() === "incompatible");
    return { required, optional, incompatible };
  }, [selected]);

  useEffect(() => {
    // Nested dep pickers manage their own lock; only the topmost visible dialog locks scroll.
    if (depPicker) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = prev;
    };
  }, [depPicker]);

  function openDependency(dep: ModrinthDependency) {
    const id = dep.project_id || dep.project_slug;
    if (!id || installing) return;
    setStatusNote(null);
    setDepPicker({
      projectId: id,
      title: depLabel(dep),
      kind: "mod",
      initialVersionId: dep.version_id ?? null,
    });
  }

  function renderDepLink(dep: ModrinthDependency, key: string) {
    const clickable = Boolean(dep.project_id || dep.project_slug);
    if (!clickable) {
      return (
        <Text key={key} type="supporting">
          · {depLabel(dep)}
        </Text>
      );
    }
    return (
      <button
        key={key}
        type="button"
        className="euml-prereq-link"
        disabled={installing}
        onClick={() => openDependency(dep)}
      >
        · {depLabel(dep)}
      </button>
    );
  }

  const installKind = details ? kindFromProjectType(details.project_type) : kind;
  const displayTitle = details?.title || title;

  async function confirmInstall() {
    if (!selected) {
      setError(t("noCompatibleVersion"));
      return;
    }
    if (installKind !== "modpack" && !instanceId) {
      setError(t("pickInstance"));
      return;
    }
    setInstalling(true);
    setError(null);
    try {
      if (installKind === "modpack") {
        const inst = await api.installModpackFromModrinth(selected.id);
        onInstalled({ label: inst.name, instanceId: inst.id });
      } else if (installKind === "mod") {
        await api.installMod(instanceId, details?.project_id || projectId, selected.id);
        onInstalled({ label: displayTitle });
      } else {
        await api.installContentFromModrinth(instanceId, selected.id, installKind, null);
        onInstalled({ label: displayTitle });
      }
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setInstalling(false);
    }
  }

  if (depPicker) {
    return (
      <ModInstallPicker
        projectId={depPicker.projectId}
        title={depPicker.title}
        gameVersion={gameVersion}
        loader={loader}
        instanceId={instanceId}
        kind={depPicker.kind}
        initialVersionId={depPicker.initialVersionId}
        onClose={() => setDepPicker(null)}
        onInstalled={({ label }) => {
          setStatusNote(`${t("install")}: ${label}`);
          setDepPicker(null);
          refreshInstalledMarkers();
        }}
      />
    );
  }

  const modal = (
    <div className="euml-modal-backdrop" role="presentation" onClick={onClose}>
      <div
        className="euml-modal euml-install-picker"
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
      >
        <Card padding={4}>
          <VStack gap={3}>
            <HStack justify="between" align="start" gap={3}>
              <VStack gap={0} style={{ flex: 1, minWidth: 0 }}>
                <Text weight="semibold">{t("chooseVersion")}</Text>
                <Text color="secondary" type="supporting">
                  {displayTitle}
                </Text>
              </VStack>
              <Button size="sm" label={t("cancel")} variant="secondary" onClick={onClose} />
            </HStack>

            {statusNote && (
              <Text color="secondary" type="supporting">
                {statusNote}
              </Text>
            )}

            {error && (
              <Text color="secondary" type="supporting">
                {error}
              </Text>
            )}

            {loading && (
              <HStack gap={2} align="center">
                <Spinner size="sm" />
                <Text color="secondary">{t("loading")}</Text>
              </HStack>
            )}

            {!loading && versions.length === 0 && (
              <Text color="secondary">{t("noCompatibleVersion")}</Text>
            )}

            {!loading && versions.length > 0 && (
              <div className="euml-version-list">
                {versions.map((v) => {
                  const installed = isVersionInstalled(v, installedIds, installedFiles);
                  const active = v.id === selectedId;
                  return (
                    <button
                      key={v.id}
                      type="button"
                      className={`euml-version-row${active ? " euml-version-row--active" : ""}`}
                      onClick={() => setSelectedId(v.id)}
                    >
                      <VStack gap={0} style={{ flex: 1, minWidth: 0, textAlign: "left" }}>
                        <HStack gap={2} align="center" style={{ flexWrap: "wrap" }}>
                          <Text weight="semibold">{v.version_number}</Text>
                          {v.version_type && v.version_type !== "release" && (
                            <span className="euml-version-badge">{v.version_type}</span>
                          )}
                          {installed && (
                            <span className="euml-version-badge euml-version-badge--installed">
                              {t("alreadyInstalled")}
                            </span>
                          )}
                        </HStack>
                        <Text color="secondary" type="supporting">
                          {[
                            v.name && v.name !== v.version_number ? v.name : null,
                            (v.loaders ?? []).slice(0, 3).join(", "),
                            v.date_published
                              ? new Date(v.date_published).toLocaleDateString()
                              : null,
                          ]
                            .filter(Boolean)
                            .join(" · ")}
                        </Text>
                      </VStack>
                    </button>
                  );
                })}
              </div>
            )}

            {selected && (
              <div className="euml-prereq-panel">
                <Text weight="semibold">{t("prerequisites")}</Text>
                {prereqs.required.length === 0 &&
                  prereqs.optional.length === 0 &&
                  prereqs.incompatible.length === 0 && (
                    <Text color="secondary" type="supporting">
                      {t("prerequisitesNone")}
                    </Text>
                  )}
                {prereqs.required.length > 0 && (
                  <VStack gap={1}>
                    <Text type="supporting" color="secondary">
                      {t("prerequisitesRequired")}
                    </Text>
                    {prereqs.required.map((d, i) => renderDepLink(d, `r-${i}`))}
                  </VStack>
                )}
                {prereqs.optional.length > 0 && (
                  <VStack gap={1}>
                    <Text type="supporting" color="secondary">
                      {t("prerequisitesOptional")}
                    </Text>
                    {prereqs.optional.map((d, i) => renderDepLink(d, `o-${i}`))}
                  </VStack>
                )}
                {prereqs.incompatible.length > 0 && (
                  <VStack gap={1}>
                    <Text type="supporting" color="secondary">
                      {t("prerequisitesIncompatible")}
                    </Text>
                    {prereqs.incompatible.map((d, i) => renderDepLink(d, `i-${i}`))}
                  </VStack>
                )}
                {isVersionInstalled(selected, installedIds, installedFiles) && (
                  <Text color="secondary" type="supporting">
                    {t("alreadyInstalledHint")}
                  </Text>
                )}
              </div>
            )}

            {installing && (
              <div className="euml-install-progress">
                <HStack justify="between" align="center" gap={2}>
                  <Text type="supporting" style={{ flex: 1, minWidth: 0 }}>
                    {progressLabel}
                  </Text>
                  {progressCounter && <Text type="supporting">{progressCounter}</Text>}
                </HStack>
                <div className="euml-progress-track">
                  <div
                    className="euml-progress-fill"
                    style={{
                      width: progressPct == null ? "35%" : `${progressPct}%`,
                      opacity: progressPct == null ? 0.55 : 1,
                    }}
                  />
                </div>
              </div>
            )}

            <HStack gap={2} justify="end" style={{ flexWrap: "wrap" }}>
              <Button
                size="sm"
                label={t("cancel")}
                variant="secondary"
                isDisabled={installing}
                onClick={onClose}
              />
              <Button
                size="sm"
              label={
                installing
                  ? installKind === "modpack"
                    ? t("installingModpack")
                    : t("installing")
                  : installKind === "modpack"
                    ? t("installModpack")
                    : t("install")
              }
              variant="primary"
              isLoading={installing}
              isDisabled={!selected || installing || (installKind !== "modpack" && !instanceId)}
              onClick={() => void confirmInstall()}
            />
            </HStack>
          </VStack>
        </Card>
      </div>
    </div>
  );

  return createPortal(modal, document.body);
}
