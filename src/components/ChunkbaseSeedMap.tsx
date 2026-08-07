import { useEffect, useMemo, useState } from "react";
import { Button } from "@astryxdesign/core/Button";
import { Text } from "@astryxdesign/core/Text";
import { Selector } from "@astryxdesign/core/Selector";
import { HStack } from "@astryxdesign/core/HStack";
import { VStack } from "@astryxdesign/core/VStack";
import { useI18n } from "../i18n";
import {
  buildChunkbaseSeedMapUrl,
  type ChunkbaseDimension,
} from "../lib/chunkbase";

type Props = {
  seed: string;
  gameVersion: string;
  /** Kept for callers; Chunkbase forbids iframes and in-app windows deadlock on Windows. */
  defaultExpanded?: boolean;
};

export function ChunkbaseSeedMap({ seed, gameVersion }: Props) {
  const { t } = useI18n();
  const [dimension, setDimension] = useState<ChunkbaseDimension>("overworld");
  const [debouncedSeed, setDebouncedSeed] = useState(seed);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const id = window.setTimeout(() => setDebouncedSeed(seed), 400);
    return () => window.clearTimeout(id);
  }, [seed]);

  const url = useMemo(
    () => buildChunkbaseSeedMapUrl(debouncedSeed, gameVersion, dimension),
    [debouncedSeed, gameVersion, dimension],
  );

  async function openInBrowser() {
    setBusy(true);
    setError(null);
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
    } catch (e) {
      try {
        window.open(url, "_blank", "noopener,noreferrer");
      } catch {
        setError(String(e));
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <VStack gap={2} className="euml-chunkbase">
      <VStack gap={0}>
        <Text weight="semibold">{t("chunkbaseSeedMap")}</Text>
        <Text color="secondary" type="supporting">
          {t("chunkbaseSeedMapHint")}
        </Text>
      </VStack>

      <Selector
        label={t("chunkbaseDimension")}
        value={dimension}
        onChange={(v) => setDimension(v as ChunkbaseDimension)}
        options={[
          { value: "overworld", label: t("chunkbaseOverworld") },
          { value: "nether", label: t("chunkbaseNether") },
          { value: "end", label: t("chunkbaseEnd") },
        ]}
      />

      <div className="euml-chunkbase-panel">
        <Text weight="semibold">{t("chunkbaseOpenPrompt")}</Text>
        <Text color="secondary" type="supporting">
          {t("chunkbaseOpenPromptHint")}
        </Text>
        <HStack gap={2} style={{ marginTop: 10, flexWrap: "wrap" }}>
          <Button
            size="sm"
            label={t("openChunkbase")}
            variant="primary"
            isDisabled={busy}
            onClick={() => void openInBrowser()}
          />
        </HStack>
        {error && (
          <Text color="secondary" type="supporting" style={{ marginTop: 8 }}>
            {error}
          </Text>
        )}
      </div>

      <Text color="secondary" type="supporting">
        {t("chunkbaseAttribution")}
      </Text>
    </VStack>
  );
}
