import { useDownloadStatus } from "../lib/downloadStatus";
import { useI18n } from "../i18n";
import { Button } from "@astryxdesign/core/Button";
import { Text } from "@astryxdesign/core/Text";
import { HStack } from "@astryxdesign/core/HStack";
import { VStack } from "@astryxdesign/core/VStack";
import { ConsolePanel } from "./ConsolePanel";

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

export function DownloadStatusBar() {
  const { t } = useI18n();
  const { progress, consoleOpen, setConsoleOpen, consoleDetached, openConsoleWindow } =
    useDownloadStatus();

  // Single-file transfers report bytes; multi-file batches report file counts.
  const bytesTotal = progress?.bytesTotal ?? null;
  const bytesDone = progress?.bytesDone ?? null;
  const byteMode = bytesTotal != null && bytesTotal > 0 && bytesDone != null;

  const pct = byteMode
    ? Math.min(100, Math.round((bytesDone! / bytesTotal!) * 100))
    : progress && progress.total > 0
      ? Math.min(100, Math.round((progress.done / progress.total) * 100))
      : 0;

  const speed = byteMode
    ? progress?.byteSpeed != null
      ? `${humanBytes(progress.byteSpeed)}/s`
      : null
    : progress?.bytesPerSec != null
      ? `${progress.bytesPerSec.toFixed(1)} ${t("filesPerSec")}`
      : null;

  const counter = byteMode
    ? `${humanBytes(bytesDone!)} / ${humanBytes(bytesTotal!)}`
    : progress
      ? `${progress.done}/${progress.total}`
      : "";

  const hasBar = byteMode || Boolean(progress && progress.total > 0);
  const visible = Boolean(progress?.active) || consoleOpen || Boolean(progress?.message);

  if (!visible && !consoleOpen) return null;

  return (
    <div className="euml-download-dock">
      {(progress?.active || progress?.message) && (
        <div className="euml-download-bar">
          <HStack justify="between" align="center" gap={3} style={{ width: "100%" }}>
            <VStack gap={0.5} style={{ flex: 1, minWidth: 0 }}>
              <Text weight="semibold" type="supporting">
                {progress?.active ? t("downloadStatus") : t("downloadIdle")}
                {progress?.phase ? ` · ${progress.phase}` : ""}
              </Text>
              <Text color="secondary" type="supporting" className="euml-hit-desc">
                {progress?.message}
                {speed ? ` · ${speed}` : ""}
              </Text>
              {hasBar && (
                <div className="euml-progress-track">
                  <div className="euml-progress-fill" style={{ width: `${pct}%` }} />
                </div>
              )}
            </VStack>
            <Text type="supporting">{counter}</Text>
            <Button
              size="sm"
              label={consoleDetached ? t("consoleFocus") : consoleOpen ? t("hideConsole") : t("showConsole")}
              onClick={() => {
                if (consoleDetached) void openConsoleWindow();
                else setConsoleOpen(!consoleOpen);
              }}
            />
          </HStack>
        </div>
      )}
      {consoleOpen && !consoleDetached && <ConsolePanel variant="dock" />}
    </div>
  );
}
