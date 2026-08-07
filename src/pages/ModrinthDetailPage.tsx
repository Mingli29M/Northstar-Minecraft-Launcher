import { useEffect, useMemo, useState } from "react";
import { createPortal } from "react-dom";
import { useLocation, useNavigate, useSearchParams } from "react-router-dom";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { Spinner } from "@astryxdesign/core/Spinner";
import { DismissibleBanner } from "../components/DismissibleBanner";
import { FavoriteButton } from "../components/FavoriteButton";
import { MarkdownBody } from "../components/MarkdownBody";
import { ModInstallPicker, type ModInstallKind } from "../components/ModInstallPicker";
import { api } from "../lib/api";
import { useI18n } from "../i18n";
import type { ModrinthProjectDetails, ModrinthVersion } from "../lib/types";

/** KeepAlive has no `<Routes>`, so `useParams()` is always empty — parse from pathname. */
function projectIdFromPath(pathname: string): string | null {
  const m = pathname.match(/^\/download\/mod\/([^/?#]+)/);
  return m?.[1] ? decodeURIComponent(m[1]) : null;
}

function primaryFile(v: ModrinthVersion) {
  return v.files.find((f) => f.primary) ?? v.files[0];
}

export function ModrinthDetailPage() {
  const { t } = useI18n();
  const { pathname } = useLocation();
  const id = useMemo(() => projectIdFromPath(pathname), [pathname]);
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const gameVersion = params.get("v") || "1.21.1";
  const loader = params.get("loader") || "fabric";
  const instanceId = params.get("instance") || "";
  const [details, setDetails] = useState<ModrinthProjectDetails | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [pickerVersionId, setPickerVersionId] = useState<string | null>(null);
  const [depPicker, setDepPicker] = useState<{
    projectId: string;
    title: string;
    initialVersionId?: string | null;
  } | null>(null);
  const [lightbox, setLightbox] = useState<string | null>(null);
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set());
  const [installedFiles, setInstalledFiles] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (!id) {
      setError(t("detailsMissingId"));
      setDetails(null);
      return;
    }
    let cancelled = false;
    setDetails(null);
    setError(null);
    setLoading(true);
    api
      .getModrinthProject(id, gameVersion, loader)
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
  }, [id, gameVersion, loader, t]);

  useEffect(() => {
    if (!instanceId || details?.project_type === "modpack") {
      setInstalledIds(new Set());
      setInstalledFiles(new Set());
      return;
    }
    let cancelled = false;
    api
      .installedModrinthMarkers(instanceId)
      .then((m) => {
        if (cancelled) return;
        setInstalledIds(new Set(m.version_ids));
        setInstalledFiles(new Set(m.filenames));
      })
      .catch(() => {
        if (!cancelled) {
          setInstalledIds(new Set());
          setInstalledFiles(new Set());
        }
      });
    return () => {
      cancelled = true;
    };
  }, [instanceId, details?.project_type]);

  const installKind: ModInstallKind = useMemo(() => {
    const pt = details?.project_type;
    if (pt === "modpack" || pt === "resourcepack" || pt === "shader" || pt === "datapack") {
      return pt;
    }
    return "mod";
  }, [details?.project_type]);

  function versionInstalled(v: ModrinthVersion): boolean {
    if (installedIds.has(v.id)) return true;
    const file = primaryFile(v);
    return !!(file && installedFiles.has(file.filename));
  }

  if (loading && !details && !error) {
    return (
      <VStack gap={3} className="euml-page" style={{ padding: 24 }}>
        <Spinner />
        <Text color="secondary">{t("loading")}</Text>
      </VStack>
    );
  }

  return (
    <VStack gap={4} className="euml-page" style={{ maxWidth: 920 }}>
      <Button size="sm" label={t("back")} onClick={() => navigate(-1)} />
      {error && <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />}
      {status && <DismissibleBanner status="success" title={status} onDismiss={() => setStatus(null)} />}

      {pickerOpen && details && (
        <ModInstallPicker
          projectId={details.project_id}
          title={details.title}
          gameVersion={gameVersion}
          loader={loader}
          instanceId={instanceId}
          kind={installKind}
          details={details}
          initialVersionId={pickerVersionId}
          onClose={() => {
            setPickerOpen(false);
            setPickerVersionId(null);
          }}
          onInstalled={({ label, instanceId: newId }) => {
            setStatus(`${t("install")}: ${label}`);
            setPickerOpen(false);
            setPickerVersionId(null);
            if (newId) navigate(`/versions/${newId}`);
            else if (instanceId) {
              void api.installedModrinthMarkers(instanceId).then((m) => {
                setInstalledIds(new Set(m.version_ids));
                setInstalledFiles(new Set(m.filenames));
              });
            }
          }}
        />
      )}

      {depPicker && (
        <ModInstallPicker
          projectId={depPicker.projectId}
          title={depPicker.title}
          gameVersion={gameVersion}
          loader={loader}
          instanceId={instanceId}
          kind="mod"
          initialVersionId={depPicker.initialVersionId}
          onClose={() => setDepPicker(null)}
          onInstalled={({ label }) => {
            setStatus(`${t("install")}: ${label}`);
            setDepPicker(null);
            if (instanceId) {
              void api.installedModrinthMarkers(instanceId).then((m) => {
                setInstalledIds(new Set(m.version_ids));
                setInstalledFiles(new Set(m.filenames));
              });
            }
          }}
        />
      )}

      {lightbox &&
        createPortal(
          <div
            className="euml-gallery-lightbox"
            role="presentation"
            onClick={() => setLightbox(null)}
          >
            <img src={lightbox} alt="" />
          </div>,
          document.body,
        )}

      {details && (
        <Card padding={4}>
          <HStack gap={4} align="start">
            {details.icon_url && (
              <img src={details.icon_url} alt="" className="euml-avatar euml-avatar--lg" />
            )}
            <VStack gap={2} style={{ flex: 1 }}>
              <Text type="display-3" style={{ fontSize: 28 }}>
                {details.title}
              </Text>
              <Text color="secondary">{details.description}</Text>
              <Text type="supporting" color="secondary">
                {details.downloads.toLocaleString()} downloads · {details.followers.toLocaleString()}{" "}
                followers · {details.categories.slice(0, 6).join(" · ")}
              </Text>
              <HStack gap={2} style={{ flexWrap: "wrap" }} align="center">
                <FavoriteButton
                  kind="modrinth"
                  itemKey={details.project_id}
                  label={details.title}
                  subtitle={details.slug}
                  iconUrl={details.icon_url}
                  size={20}
                />
                <Button
                  label={
                    details.project_type === "modpack" ? t("installModpack") : t("install")
                  }
                  variant="primary"
                  onClick={() => {
                    if (details.project_type !== "modpack" && !instanceId) {
                      setError(t("pickInstance"));
                      return;
                    }
                    if (!details.versions[0]) {
                      setError(t("noCompatibleVersion"));
                      return;
                    }
                    setPickerVersionId(details.versions[0]?.id ?? null);
                    setPickerOpen(true);
                  }}
                />
                <Button size="sm" label="Modrinth" onClick={() => void openUrl(details.modrinth_url)} />
                <Button size="sm" label="MCMod.cn" onClick={() => void openUrl(details.mcmod_url)} />
                <Button
                  size="sm"
                  label="CurseForge"
                  onClick={() => void openUrl(details.curseforge_url)}
                />
                {details.source_url && (
                  <Button size="sm" label="Source" onClick={() => void openUrl(details.source_url!)} />
                )}
                {details.wiki_url && (
                  <Button size="sm" label="Wiki" onClick={() => void openUrl(details.wiki_url!)} />
                )}
              </HStack>
            </VStack>
          </HStack>

          {details.gallery?.length > 0 && (
            <VStack gap={2} style={{ marginTop: 20 }}>
              <Text weight="semibold">{t("gallery")}</Text>
              <div className="euml-gallery">
                {details.gallery.map((img) => (
                  <button
                    key={img.url}
                    type="button"
                    className="euml-gallery-item"
                    title={img.title ?? undefined}
                    onClick={() => setLightbox(img.url)}
                  >
                    <img src={img.url} alt={img.title ?? details.title} loading="lazy" />
                  </button>
                ))}
              </div>
            </VStack>
          )}

          {details.body && (
            <VStack gap={2} style={{ marginTop: 20 }}>
              <Text weight="semibold">{t("descriptionMd")}</Text>
              <MarkdownBody content={details.body} />
            </VStack>
          )}

          {details.versions.length > 0 && (
            <VStack gap={2} style={{ marginTop: 20 }}>
              <Text weight="semibold">{t("versionList")}</Text>
              <div className="euml-version-list" style={{ maxHeight: 360 }}>
                {details.versions.map((v) => {
                  const installed = versionInstalled(v);
                  return (
                    <div key={v.id} className="euml-version-row" style={{ cursor: "default" }}>
                      <VStack gap={0} style={{ flex: 1, minWidth: 0 }}>
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
                            (v.loaders ?? []).slice(0, 4).join(", "),
                            v.date_published
                              ? new Date(v.date_published).toLocaleDateString()
                              : null,
                          ]
                            .filter(Boolean)
                            .join(" · ")}
                        </Text>
                        {(v.dependencies ?? []).filter((d) => d.dependency_type === "required")
                          .length > 0 && (
                          <div className="euml-version-deps">
                            <Text color="secondary" type="supporting">
                              {t("prerequisitesRequired")}:{" "}
                            </Text>
                            {(v.dependencies ?? [])
                              .filter((d) => d.dependency_type === "required")
                              .map((d, i, arr) => {
                                const id = d.project_id || d.project_slug;
                                const label =
                                  d.project_title || d.project_slug || d.project_id || "dependency";
                                return (
                                  <span key={`${v.id}-dep-${i}`}>
                                    {id ? (
                                      <button
                                        type="button"
                                        className="euml-prereq-link euml-prereq-link--inline"
                                        onClick={() => {
                                          if (!instanceId && details.project_type !== "modpack") {
                                            setError(t("pickInstance"));
                                            return;
                                          }
                                          setDepPicker({
                                            projectId: id,
                                            title: label,
                                            initialVersionId: d.version_id ?? null,
                                          });
                                        }}
                                      >
                                        {label}
                                      </button>
                                    ) : (
                                      label
                                    )}
                                    {i < arr.length - 1 ? ", " : ""}
                                  </span>
                                );
                              })}
                          </div>
                        )}
                      </VStack>
                      <Button
                        size="sm"
                        label={t("installThisVersion")}
                        variant="secondary"
                        onClick={() => {
                          if (details.project_type !== "modpack" && !instanceId) {
                            setError(t("pickInstance"));
                            return;
                          }
                          setPickerVersionId(v.id);
                          setPickerOpen(true);
                        }}
                      />
                    </div>
                  );
                })}
              </div>
            </VStack>
          )}
        </Card>
      )}
    </VStack>
  );
}
