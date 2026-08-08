import { useCallback, useEffect, useState } from "react";
import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { Spinner } from "@astryxdesign/core/Spinner";
import { DismissibleBanner } from "../components/DismissibleBanner";
import { api } from "../lib/api";
import { useI18n } from "../i18n";
import type { MessageKey } from "../i18n/messages";
import type { TerracottaInfo, TerracottaState } from "../lib/types";

/** Upstream state names from Terracotta's `/state` endpoint. */
const PHASE_LABELS: Record<string, MessageKey> = {
  offline: "terracottaPhaseOffline",
  starting: "terracottaPhaseStarting",
  waiting: "terracottaPhaseWaiting",
  "host-scanning": "terracottaPhaseHostScanning",
  "host-starting": "terracottaPhaseHostStarting",
  "host-ok": "terracottaPhaseHostOk",
  "guest-connecting": "terracottaPhaseGuestConnecting",
  "guest-starting": "terracottaPhaseGuestStarting",
  "guest-ok": "terracottaPhaseGuestOk",
  exception: "terracottaPhaseException",
  error: "terracottaPhaseError",
  unknown: "terracottaPhaseUnknown",
};

export function TerracottaPage() {
  const { t } = useI18n();
  const [info, setInfo] = useState<TerracottaInfo | null>(null);
  const [state, setState] = useState<TerracottaState | null>(null);
  const [room, setRoom] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const next = await api.terracottaInfo();
    setInfo(next);
    if (next.running) {
      setState(await api.terracottaState());
    } else {
      setState(null);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    refresh().catch((e) => {
      if (!cancelled) setError(String(e));
    });
    return () => {
      cancelled = true;
    };
  }, [refresh]);

  useEffect(() => {
    if (!info?.running) return;
    const timer = window.setInterval(() => {
      api
        .terracottaState()
        .then(setState)
        .catch(() => undefined);
    }, 1500);
    return () => clearInterval(timer);
  }, [info?.running]);

  async function run(action: () => Promise<unknown>, okMsg?: string) {
    setBusy(true);
    setError(null);
    setStatusMsg(null);
    try {
      await action();
      await refresh();
      if (okMsg) setStatusMsg(okMsg);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  const phase = state?.phase ?? (info?.running ? "unknown" : "offline");
  const phaseKey = PHASE_LABELS[phase] ?? "terracottaPhaseUnknown";
  const phaseLabel = t(phaseKey);
  const roomCode = state?.room ?? null;
  const joinUrl = state?.url ?? null;
  const scanning = phase === "host-scanning";
  const isMacos = (info?.platformClassifier ?? "").startsWith("macos");
  const hostScanHint = isMacos ? t("terracottaHostNoWorldMac") : t("terracottaHostNoWorld");

  return (
    <VStack gap={4} className="euml-page" style={{ maxWidth: 720 }}>
      <VStack gap={2}>
        <Text type="display-3">{t("terracottaTitle")}</Text>
        <Text color="secondary">{t("terracottaHint")}</Text>
      </VStack>

      <Banner
        status="info"
        title={t("terracottaAttributionTitle")}
        description={info?.attribution ?? t("terracottaAttributionFallback")}
      />

      <Card padding={3} className="euml-panel">
        <Text color="secondary" type="supporting">
          {info?.licenseNote ?? t("terracottaLicenseNote")}
        </Text>
        <HStack gap={2} style={{ marginTop: 10, flexWrap: "wrap" }}>
          <Button
            size="sm"
            variant="secondary"
            label={t("terracottaUpstream")}
            onClick={() =>
              window.open(
                info?.upstreamUrl ?? "https://github.com/burningtnt/Terracotta",
                "_blank",
                "noopener,noreferrer",
              )
            }
          />
          <Text color="secondary" type="supporting">
            {info?.upstreamLicense ?? "AGPL-3.0-or-later"} · v{info?.version ?? "…"}
          </Text>
        </HStack>
      </Card>

      {error && (
        <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />
      )}
      {statusMsg && (
        <DismissibleBanner
          status="success"
          title={statusMsg}
          onDismiss={() => setStatusMsg(null)}
        />
      )}

      {!info ? (
        <HStack gap={2} align="center">
          <Spinner size="sm" />
          <Text color="secondary">{t("loading")}</Text>
        </HStack>
      ) : !info.supported ? (
        <Banner
          status="warning"
          title={t("terracottaUnsupported")}
          description={t("terracottaUnsupportedHint")}
        />
      ) : (
        <VStack gap={3}>
          <Card padding={4} className="euml-panel">
            <VStack gap={3}>
              <HStack justify="between" align="center" style={{ flexWrap: "wrap" }} gap={2}>
                <VStack gap={1}>
                  <Text weight="semibold">{t("terracottaSidecar")}</Text>
                  <HStack gap={2} align="center" style={{ flexWrap: "wrap" }}>
                    <span
                      className={`euml-host-status ${info.running ? "is-running" : "is-stopped"}`}
                    >
                      <span className="euml-host-status__dot" aria-hidden />
                      {info.running ? t("terracottaRunning") : t("terracottaStopped")}
                    </span>
                    <Text color="secondary" type="supporting">
                      {info.installed ? t("terracottaInstalled") : t("terracottaNotInstalled")}
                      {info.port ? ` · :${info.port}` : ""}
                    </Text>
                  </HStack>
                </VStack>
                <HStack gap={2} style={{ flexWrap: "wrap" }}>
                  {!info.installed && (
                    <Button
                      label={busy ? t("terracottaInstalling") : t("terracottaInstall")}
                      variant="primary"
                      isDisabled={busy}
                      onClick={() =>
                        void run(
                          () => api.terracottaInstall(),
                          t("terracottaInstallOk"),
                        )
                      }
                    />
                  )}
                  {info.installed && !info.running && (
                    <Button
                      label={busy ? t("preparing") : t("terracottaStart")}
                      variant="primary"
                      isDisabled={busy}
                      onClick={() => void run(() => api.terracottaStart())}
                    />
                  )}
                  {info.running && (
                    <Button
                      label={t("terracottaStop")}
                      variant="secondary"
                      isDisabled={busy}
                      onClick={() => void run(() => api.terracottaStop())}
                    />
                  )}
                  {info.installed && (
                    <Button
                      label={t("terracottaReinstall")}
                      variant="secondary"
                      // Install stops a running sidecar first, so this stays
                      // available as a recovery path for a broken install.
                      isDisabled={busy}
                      onClick={() =>
                        void run(
                          () => api.terracottaInstall(),
                          t("terracottaInstallOk"),
                        )
                      }
                    />
                  )}
                </HStack>
              </HStack>
              {isMacos && !info.installed && (
                <Text color="secondary" type="supporting">
                  {t("terracottaInstallMacHint")}
                </Text>
              )}
              <Text color="secondary" type="supporting">
                {t("terracottaPhase")}: {phaseLabel}
                {state?.difficulty ? ` · ${state.difficulty}` : ""}
              </Text>
              {state?.message && (
                <Text color="secondary" type="supporting">
                  {state.message}
                </Text>
              )}
            </VStack>
          </Card>

          <Card padding={4} className="euml-panel">
            <VStack gap={3}>
              <Text weight="semibold">{t("terracottaHostTitle")}</Text>
              <Text color="secondary" type="supporting">
                {t("terracottaHostHint")}
              </Text>
              <Button
                label={t("terracottaCreateRoom")}
                variant="primary"
                isDisabled={busy || !info.running}
                onClick={() => void run(() => api.terracottaHost())}
              />
              {scanning && !roomCode && (
                <Text color="secondary" type="supporting">
                  {hostScanHint}
                </Text>
              )}
              {roomCode && (
                <VStack gap={1}>
                  <Text weight="semibold">{t("terracottaRoomCode")}</Text>
                  <HStack gap={2} align="center" style={{ flexWrap: "wrap" }}>
                    <Text type="display-3" style={{ fontSize: 28, letterSpacing: 2 }}>
                      {roomCode}
                    </Text>
                    <Button
                      size="sm"
                      variant="secondary"
                      label={t("terracottaCopy")}
                      onClick={() => void navigator.clipboard.writeText(roomCode)}
                    />
                  </HStack>
                </VStack>
              )}
            </VStack>
          </Card>

          <Card padding={4} className="euml-panel">
            <VStack gap={3}>
              <Text weight="semibold">{t("terracottaJoinTitle")}</Text>
              <Text color="secondary" type="supporting">
                {t("terracottaJoinHint")}
              </Text>
              <TextInput
                label={t("terracottaRoomCode")}
                value={room}
                onChange={setRoom}
              />
              <HStack gap={2} style={{ flexWrap: "wrap" }}>
                <Button
                  label={t("terracottaJoinRoom")}
                  variant="primary"
                  isDisabled={busy || !info.running || !room.trim()}
                  onClick={() => void run(() => api.terracottaJoin(room.trim()))}
                />
                <Button
                  label={t("terracottaIdle")}
                  variant="secondary"
                  isDisabled={busy || !info.running}
                  onClick={() => void run(() => api.terracottaIdle())}
                />
              </HStack>
              {joinUrl && (
                <VStack gap={1}>
                  <HStack gap={2} align="center" style={{ flexWrap: "wrap" }}>
                    <Text weight="semibold">{t("terracottaJoinUrl")}:</Text>
                    <Text>{joinUrl}</Text>
                    <Button
                      size="sm"
                      variant="secondary"
                      label={t("terracottaCopy")}
                      onClick={() => void navigator.clipboard.writeText(joinUrl)}
                    />
                  </HStack>
                  <Text color="secondary" type="supporting">
                    {t("terracottaJoinInMinecraft")}
                  </Text>
                </VStack>
              )}
            </VStack>
          </Card>

          {state?.profiles && state.profiles.length > 0 && (
            <Card padding={4} className="euml-panel">
              <VStack gap={2}>
                <Text weight="semibold">{t("terracottaPeers")}</Text>
                {state.profiles.map((p, i) => (
                  <Text key={`${p.machineId ?? i}-${p.name ?? ""}`} type="supporting">
                    {p.name ?? t("terracottaAnonymous")}
                    {p.vendor ? ` · ${p.vendor}` : ""}
                    {p.kind ? ` · ${p.kind}` : ""}
                  </Text>
                ))}
              </VStack>
            </Card>
          )}
        </VStack>
      )}
    </VStack>
  );
}
