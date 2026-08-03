import { useEffect, useMemo, useState } from "react";
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
import { api } from "../lib/api";
import { useI18n } from "../i18n";
import type { ModrinthProjectDetails } from "../lib/types";

/** KeepAlive has no `<Routes>`, so `useParams()` is always empty — parse from pathname. */
function projectIdFromPath(pathname: string): string | null {
  const m = pathname.match(/^\/download\/mod\/([^/?#]+)/);
  return m?.[1] ? decodeURIComponent(m[1]) : null;
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
  const [installing, setInstalling] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

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

  async function install() {
    if (!details?.versions[0] || !instanceId) {
      setError(t("pickInstance"));
      return;
    }
    setInstalling(true);
    setError(null);
    try {
      if (details.project_type === "mod") {
        await api.installMod(instanceId, details.project_id, details.versions[0].id);
      } else {
        await api.installContentFromModrinth(instanceId, details.versions[0].id, details.project_type, null);
      }
      setStatus(`${t("install")}: ${details.title}`);
    } catch (e) {
      setError(String(e));
    } finally {
      setInstalling(false);
    }
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
    <VStack gap={4} className="euml-page" style={{ maxWidth: 860 }}>
      <Button size="sm" label={t("back")} onClick={() => navigate(-1)} />
      {error && <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />}
      {status && <DismissibleBanner status="success" title={status} onDismiss={() => setStatus(null)} />}
      {details && (
        <Card padding={4}>
          <HStack gap={4} align="start">
            {details.icon_url && <img src={details.icon_url} alt="" className="euml-avatar euml-avatar--lg" />}
            <VStack gap={2} style={{ flex: 1 }}>
              <Text type="display-3" style={{ fontSize: 28 }}>
                {details.title}
              </Text>
              <Text color="secondary">{details.description}</Text>
              <Text type="supporting" color="secondary">
                {details.downloads.toLocaleString()} downloads · {details.followers.toLocaleString()} followers ·{" "}
                {details.categories.slice(0, 6).join(" · ")}
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
                <Button label={t("install")} variant="primary" isLoading={installing} onClick={install} />
                <Button size="sm" label="Modrinth" onClick={() => openUrl(details.modrinth_url)} />
                <Button size="sm" label="MCMod.cn" onClick={() => openUrl(details.mcmod_url)} />
                <Button size="sm" label="CurseForge" onClick={() => openUrl(details.curseforge_url)} />
                {details.source_url && (
                  <Button size="sm" label="Source" onClick={() => openUrl(details.source_url!)} />
                )}
                {details.wiki_url && <Button size="sm" label="Wiki" onClick={() => openUrl(details.wiki_url!)} />}
              </HStack>
            </VStack>
          </HStack>
          {details.body && (
            <div
              className="euml-mod-body"
              style={{ marginTop: 20, fontSize: 14, lineHeight: 1.55, whiteSpace: "pre-wrap" }}
            >
              {details.body.replace(/#{1,6}\s/g, "").slice(0, 6000)}
            </div>
          )}
          {details.versions.length > 0 && (
            <VStack gap={1} style={{ marginTop: 16 }}>
              <Text weight="semibold">{t("compatibleVersions")}</Text>
              {details.versions.map((v) => (
                <Text key={v.id} type="supporting" color="secondary">
                  {v.version_number}
                </Text>
              ))}
            </VStack>
          )}
        </Card>
      )}
    </VStack>
  );
}
