import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@astryxdesign/core/Button";
import { DismissibleBanner } from "../components/DismissibleBanner";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { Selector } from "@astryxdesign/core/Selector";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { api } from "../lib/api";
import { useI18n } from "../i18n";
import { FavoriteButton } from "../components/FavoriteButton";
import { useFavorites } from "../lib/favorites";
import { loadPreferredInstanceId, rememberPreferredInstance } from "../lib/preferredInstance";
import { favoriteId, normalizeServerKey } from "../lib/types";
import type { Instance, ServerEntry } from "../lib/types";

export function ServersPage() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { isFavorite } = useFavorites();
  const [instances, setInstances] = useState<Instance[]>([]);
  const [instanceId, setInstanceId] = useState("");
  const [servers, setServers] = useState<ServerEntry[]>([]);
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [name, setName] = useState("");
  const [ip, setIp] = useState("");
  const [editIndex, setEditIndex] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void loadPreferredInstanceId().then(({ instances: list, instanceId: preferred }) => {
      setInstances(list);
      if (preferred) setInstanceId(preferred);
    });
  }, []);

  useEffect(() => {
    if (!instanceId) {
      setServers([]);
      return;
    }
    void rememberPreferredInstance(instanceId);
    api
      .listServers(instanceId)
      .then(setServers)
      .catch((e) => setError(String(e)));
  }, [instanceId]);

  async function onSaveServer() {
    if (!instanceId) return setError(t("pickInstance"));
    setError(null);
    try {
      if (editIndex != null) {
        setServers(await api.updateServer(instanceId, editIndex, name, ip));
        setEditIndex(null);
      } else {
        setServers(await api.addServer(instanceId, name, ip));
      }
      setName("");
      setIp("");
    } catch (e) {
      setError(String(e));
    }
  }

  async function join(addr: string) {
    if (!instanceId) return;
    setBusy(true);
    setError(null);
    setStatus(t("preparing"));
    try {
      await api.prepareInstance(instanceId);
      setStatus(t("launching"));
      setStatus(await api.launchInstance(instanceId, false, addr));
    } catch (e) {
      setError(String(e));
      setStatus(null);
    } finally {
      setBusy(false);
    }
  }

  return (
    <VStack gap={4} className="euml-page">
      <Text type="display-3">{t("serversTitle")}</Text>
      <Text color="secondary">{t("serversHint")}</Text>
      {error && <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />}
      {status && <DismissibleBanner status="info" title={status} onDismiss={() => setStatus(null)} />}

      <Selector
        label={t("targetVersion")}
        value={instanceId}
        onChange={setInstanceId}
        options={[
          { value: "", label: "—" },
          ...instances.map((i) => ({ value: i.id, label: `${i.name} (${i.game_version})` })),
        ]}
      />

      <Card padding={4} className="euml-fade-in">
        <VStack gap={3}>
          <HStack gap={2}>
            <TextInput label={t("serverName")} value={name} onChange={setName} />
            <TextInput label={t("serverAddress")} value={ip} onChange={setIp} />
            <Button
              label={editIndex != null ? t("save") : t("addServer")}
              variant="primary"
              onClick={onSaveServer}
            />
            {editIndex != null && (
              <Button
                label={t("cancel")}
                variant="secondary"
                onClick={() => {
                  setEditIndex(null);
                  setName("");
                  setIp("");
                }}
              />
            )}
          </HStack>
        </VStack>
      </Card>

      <HStack gap={2} align="center">
        <Button
          size="sm"
          label={favoritesOnly ? t("favorites") : t("showFavoritesOnly")}
          variant={favoritesOnly ? "primary" : "secondary"}
          onClick={() => setFavoritesOnly((v) => !v)}
        />
      </HStack>

      <Card padding={0} className="euml-fade-in">
        {[...servers]
          .map((s, i) => ({ s, i }))
          .sort((a, b) => {
            const af = isFavorite(favoriteId("server", normalizeServerKey(a.s.ip))) ? 0 : 1;
            const bf = isFavorite(favoriteId("server", normalizeServerKey(b.s.ip))) ? 0 : 1;
            return af - bf;
          })
          .filter(({ s }) =>
            favoritesOnly ? isFavorite(favoriteId("server", normalizeServerKey(s.ip))) : true,
          )
          .map(({ s, i }) => (
          <HStack
            key={`${s.name}-${s.ip}-${i}`}
            justify="between"
            align="center"
            className="euml-list-row"
            style={{ borderBottom: "1px solid var(--color-border)" }}
          >
            <VStack gap={0.5} style={{ flex: 1, minWidth: 0 }}>
              <Text weight="semibold">{s.name}</Text>
              <Text color="secondary" type="supporting">
                {s.ip}
              </Text>
            </VStack>
            <HStack gap={2} align="center">
              <FavoriteButton
                kind="server"
                itemKey={normalizeServerKey(s.ip)}
                label={s.name}
                subtitle={s.ip}
              />
              <Button
                size="sm"
                label={t("joinServer")}
                variant="primary"
                isDisabled={busy || !instanceId}
                isLoading={busy}
                onClick={() => join(s.ip)}
              />
              <Button
                size="sm"
                label={t("edit")}
                variant="secondary"
                onClick={() => {
                  setEditIndex(i);
                  setName(s.name);
                  setIp(s.ip);
                }}
              />
              <Button
                size="sm"
                label={t("delete")}
                variant="destructive"
                onClick={async () => {
                  setServers(await api.removeServer(instanceId, i));
                  if (editIndex === i) {
                    setEditIndex(null);
                    setName("");
                    setIp("");
                  }
                }}
              />
            </HStack>
          </HStack>
        ))}
        {servers.length === 0 && (
          <div style={{ padding: 16 }}>
            <Text color="secondary">{t("none")}</Text>
          </div>
        )}
        {servers.length > 0 &&
          favoritesOnly &&
          servers.every((s) => !isFavorite(favoriteId("server", normalizeServerKey(s.ip)))) && (
            <div style={{ padding: 16 }}>
              <Text color="secondary">{t("favoritesEmpty")}</Text>
            </div>
          )}
      </Card>

      <Card padding={3} className="euml-fade-in">
        <VStack gap={2}>
          <Text color="secondary" type="supporting">
            {t("terracottaNote")}
          </Text>
          <Button
            label={t("openTerracotta")}
            variant="secondary"
            onClick={() => navigate("/terracotta")}
          />
        </VStack>
      </Card>
    </VStack>
  );
}
