import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@astryxdesign/core/Button";
import { Text } from "@astryxdesign/core/Text";
import { HStack } from "@astryxdesign/core/HStack";
import { VStack } from "@astryxdesign/core/VStack";
import { api } from "../lib/api";
import { useI18n } from "../i18n";
import type { ExitBlockers } from "../lib/types";

/**
 * The Rust side vetoes the window close while a dedicated server or the
 * Terracotta sidecar is live, then emits `euml:exit-blocked`. This names what is
 * still running so quitting can never silently orphan a server process.
 */
export function ExitGuard() {
  const { t } = useI18n();
  const [blockers, setBlockers] = useState<ExitBlockers | null>(null);
  const [stopping, setStopping] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<ExitBlockers>("euml:exit-blocked", (event) => {
      setBlockers(event.payload);
      // Tells the backend the prompt is reachable, so it keeps vetoing closes
      // instead of falling back to quitting unattended.
      void api.ackExitPrompt().catch(() => undefined);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  if (!blockers) return null;

  const items = [
    ...blockers.servers.map((name) => t("exitGuardServer").replace("{name}", name)),
    ...(blockers.terracotta ? [t("exitGuardTerracotta")] : []),
  ];

  return (
    <div className="euml-exit-guard" role="dialog" aria-modal="true">
      <div className="euml-exit-guard__panel">
        <VStack gap={3}>
          <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
            {t("exitGuardTitle")}
          </Text>
          <Text color="secondary">{t("exitGuardBody")}</Text>
          <ul className="euml-exit-guard__list">
            {items.map((label) => (
              <li key={label}>
                <Text>{label}</Text>
              </li>
            ))}
          </ul>
          <HStack gap={2} justify="end" style={{ flexWrap: "wrap" }}>
            <Button
              label={t("exitGuardCancel")}
              variant="secondary"
              isDisabled={stopping}
              onClick={() => setBlockers(null)}
            />
            <Button
              label={stopping ? t("exitGuardStopping") : t("exitGuardStopAndQuit")}
              variant="primary"
              isDisabled={stopping}
              onClick={() => {
                setStopping(true);
                void api.confirmExit().catch(() => setStopping(false));
              }}
            />
          </HStack>
        </VStack>
      </div>
    </div>
  );
}
