import { useDownloadStatus } from "../lib/downloadStatus";
import { useI18n } from "../i18n";
import { Button } from "@astryxdesign/core/Button";
import { Text } from "@astryxdesign/core/Text";
import { HStack } from "@astryxdesign/core/HStack";
import { VStack } from "@astryxdesign/core/VStack";
import { ConsolePanel } from "./ConsolePanel";

export function DownloadStatusBar() {
  const { t } = useI18n();
  const { progress, consoleOpen, setConsoleOpen, consoleDetached, openConsoleWindow } =
    useDownloadStatus();
  const pct =
    progress && progress.total > 0 ? Math.min(100, Math.round((progress.done / progress.total) * 100)) : 0;
  const speed =
    progress?.bytesPerSec != null
      ? `${progress.bytesPerSec.toFixed(1)} ${t("filesPerSec")}`
      : null;
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
              {progress && progress.total > 0 && (
                <div className="euml-progress-track">
                  <div className="euml-progress-fill" style={{ width: `${pct}%` }} />
                </div>
              )}
            </VStack>
            <Text type="supporting">{progress ? `${progress.done}/${progress.total}` : ""}</Text>
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
