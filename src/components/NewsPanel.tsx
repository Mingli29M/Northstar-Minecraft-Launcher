import { useEffect, useState } from "react";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { Spinner } from "@astryxdesign/core/Spinner";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { DismissibleBanner } from "./DismissibleBanner";
import { api } from "../lib/api";
import { useI18n } from "../i18n";
import type { MinecraftNewsItem, MinecraftPatchNote } from "../lib/types";

type Props = {
  /** Compact mode for Launch sidebar (fewer items, no huge titles). */
  compact?: boolean;
};

export function NewsPanel({ compact = false }: Props) {
  const { t } = useI18n();
  const [news, setNews] = useState<MinecraftNewsItem[]>([]);
  const [notes, setNotes] = useState<MinecraftPatchNote[]>([]);
  const [loadingNews, setLoadingNews] = useState(false);
  const [loadingNotes, setLoadingNotes] = useState(false);
  const [errNews, setErrNews] = useState<string | null>(null);
  const [errNotes, setErrNotes] = useState<string | null>(null);
  const [started, setStarted] = useState(false);

  useEffect(() => {
    let cancelled = false;
    let idleHandle: number | undefined;
    let timeoutHandle: number | undefined;

    const load = () => {
      if (cancelled) return;
      setStarted(true);
      setLoadingNews(true);
      setLoadingNotes(true);
      // Fetch independently — a slow/huge patch-notes download must not blank news.
      api
        .fetchMinecraftNews()
        .then((n) => {
          if (!cancelled) setNews(n);
        })
        .catch((e) => {
          if (!cancelled) setErrNews(String(e));
        })
        .finally(() => {
          if (!cancelled) setLoadingNews(false);
        });
      api
        .fetchMinecraftPatchNotes()
        .then((p) => {
          if (!cancelled) setNotes(p);
        })
        .catch((e) => {
          if (!cancelled) setErrNotes(String(e));
        })
        .finally(() => {
          if (!cancelled) setLoadingNotes(false);
        });
    };

    // Defer network + image decode off the cold-start critical path.
    const ric = window.requestIdleCallback?.bind(window);
    if (ric) {
      idleHandle = ric(() => load(), { timeout: 2500 });
    } else {
      timeoutHandle = window.setTimeout(load, 1200);
    }

    return () => {
      cancelled = true;
      if (idleHandle != null && window.cancelIdleCallback) {
        window.cancelIdleCallback(idleHandle);
      }
      if (timeoutHandle != null) window.clearTimeout(timeoutHandle);
    };
  }, []);

  const newsLimit = compact ? 4 : 10;
  const notesLimit = compact ? 4 : 16;

  async function openUrl(url: string) {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(url);
  }

  return (
    <VStack gap={compact ? 3 : 4} className="euml-fade-in">
      {errNews && <DismissibleBanner status="error" title={errNews} onDismiss={() => setErrNews(null)} />}
      {errNotes && <DismissibleBanner status="error" title={errNotes} onDismiss={() => setErrNotes(null)} />}

      <Text type={compact ? "body" : "display-3"} style={compact ? { fontWeight: 600 } : { fontSize: 22 }}>
        {t("newsTitle")}
      </Text>
      {loadingNews && (
        <HStack gap={2} align="center">
          <Spinner size="sm" />
          <Text color="secondary">{t("loading")}</Text>
        </HStack>
      )}
      {!started && !loadingNews && (
        <HStack gap={2} align="center">
          <Spinner size="sm" />
          <Text color="secondary">{t("loading")}</Text>
        </HStack>
      )}
      {started && !loadingNews && news.length === 0 && !errNews && (
        <Text color="secondary">{t("newsEmpty")}</Text>
      )}
      {news.slice(0, newsLimit).map((n, i) => (
        <Card key={`n-${i}`} padding={3}>
          <HStack gap={3} align="start">
            {n.image_url && <img src={n.image_url} alt="" className="euml-avatar euml-avatar--lg" />}
            <VStack gap={1} style={{ flex: 1, minWidth: 0 }}>
              <Text weight="semibold">{n.title}</Text>
              <Text color="secondary" type="supporting">
                {n.tag}
                {n.date ? ` · ${n.date}` : ""}
              </Text>
              {!compact && (
                <Text type="supporting" className="euml-hit-desc">
                  {n.text.replace(/<[^>]+>/g, " ").slice(0, 280)}
                </Text>
              )}
              {n.read_more_url && (
                <Button size="sm" label={t("readMore")} onClick={() => void openUrl(n.read_more_url!)} />
              )}
            </VStack>
          </HStack>
        </Card>
      ))}

      <Text type={compact ? "body" : "display-3"} style={compact ? { fontWeight: 600 } : { fontSize: 22 }}>
        {t("changelogTitle")}
      </Text>
      {loadingNotes && (
        <HStack gap={2} align="center">
          <Spinner size="sm" />
          <Text color="secondary">{t("loading")}</Text>
        </HStack>
      )}
      {!loadingNotes && notes.length === 0 && !errNotes && <Text color="secondary">{t("changelogEmpty")}</Text>}
      {notes.slice(0, notesLimit).map((p, i) => (
        <Card key={`p-${i}`} padding={3}>
          <Text weight="semibold">
            {p.title}
            {p.version ? ` (${p.version})` : ""}
          </Text>
          <Text type="supporting" className="euml-hit-desc" style={{ marginTop: 6 }}>
            {p.body.replace(/<[^>]+>/g, " ").slice(0, compact ? 160 : 400)}
          </Text>
        </Card>
      ))}
    </VStack>
  );
}
