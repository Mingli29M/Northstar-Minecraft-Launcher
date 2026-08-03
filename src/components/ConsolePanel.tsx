import { useEffect, useRef } from "react";
import { Button } from "@astryxdesign/core/Button";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { HStack } from "@astryxdesign/core/HStack";
import { VStack } from "@astryxdesign/core/VStack";
import { useI18n } from "../i18n";
import {
  useDownloadStatus,
  useFilteredConsoleLines,
  type ConsoleFilter,
  isConsoleWindow,
} from "../lib/downloadStatus";

const FILTERS: ConsoleFilter[] = ["all", "info", "progress", "warn", "error", "game", "server"];

type Props = {
  /** Full-window console (detached) vs dock panel */
  variant?: "dock" | "window";
};

export function ConsolePanel({ variant = "dock" }: Props) {
  const { t } = useI18n();
  const {
    clearConsole,
    consoleFilter,
    setConsoleFilter,
    consoleQuery,
    setConsoleQuery,
    setConsoleOpen,
    consoleDetached,
    openConsoleWindow,
    dockConsole,
  } = useDownloadStatus();
  const lines = useFilteredConsoleLines();
  const preRef = useRef<HTMLPreElement>(null);
  const detached = isConsoleWindow();

  useEffect(() => {
    const el = preRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [lines.length]);

  const filterLabel = (f: ConsoleFilter) => {
    switch (f) {
      case "all":
        return t("consoleFilterAll");
      case "info":
        return t("consoleFilterInfo");
      case "progress":
        return t("consoleFilterProgress");
      case "warn":
        return t("consoleFilterWarn");
      case "error":
        return t("consoleFilterError");
      case "game":
        return t("consoleFilterGame");
      case "server":
        return t("consoleFilterServer");
    }
  };

  return (
    <div className={`euml-console euml-console--${variant}`}>
      <HStack justify="between" align="center" gap={2} style={{ marginBottom: 8, flexWrap: "wrap" }}>
        <Text weight="semibold">{t("consoleTitle")}</Text>
        <HStack gap={2} style={{ flexWrap: "wrap" }}>
          {!detached && !consoleDetached && (
            <Button size="sm" label={t("consoleDetach")} onClick={() => void openConsoleWindow()} />
          )}
          {(detached || consoleDetached) && (
            <Button size="sm" label={t("consoleDock")} onClick={() => void dockConsole()} />
          )}
          <Button size="sm" label={t("clearConsole")} onClick={clearConsole} />
          {!detached && (
            <Button size="sm" label={t("hideConsole")} onClick={() => setConsoleOpen(false)} />
          )}
        </HStack>
      </HStack>

      <HStack gap={2} align="end" style={{ marginBottom: 8, flexWrap: "wrap" }}>
        <div className="euml-console-filters">
          {FILTERS.map((f) => (
            <button
              key={f}
              type="button"
              className={`euml-console-chip${consoleFilter === f ? " is-active" : ""}`}
              onClick={() => setConsoleFilter(f)}
            >
              {filterLabel(f)}
            </button>
          ))}
        </div>
        <div style={{ flex: 1, minWidth: 140 }}>
          <TextInput label={t("consoleFilterSearch")} value={consoleQuery} onChange={setConsoleQuery} />
        </div>
      </HStack>

      <pre ref={preRef} className="euml-console-pre">
        {lines.length === 0
          ? t("consoleEmpty")
          : lines.map((l, i) => (
              <div key={`${l.ts}-${i}`} className={`euml-console-line euml-console-line--${l.level}`}>
                <span className="euml-console-ts">[{l.ts}]</span> {l.text}
              </div>
            ))}
      </pre>
    </div>
  );
}

export function ConsoleWindowApp() {
  const { t } = useI18n();
  return (
    <div className="euml-console-window">
      <VStack gap={0} style={{ height: "100%" }}>
        <div className="euml-console-window__head">
          <Text weight="semibold">{t("consoleTitle")}</Text>
        </div>
        <ConsolePanel variant="window" />
      </VStack>
    </div>
  );
}
