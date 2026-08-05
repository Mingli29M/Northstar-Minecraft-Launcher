import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { DismissibleBanner } from "../components/DismissibleBanner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { TextInput } from "@astryxdesign/core/TextInput";
import { VStack } from "@astryxdesign/core/VStack";
import { HStack } from "@astryxdesign/core/HStack";
import { api } from "../lib/api";
import { AccountAvatar } from "../components/AccountAvatar";
import { useI18n } from "../i18n";
import type { Account } from "../lib/types";

function kindLabel(kind: Account["kind"], t: (k: "offline" | "microsoft" | "littleskin") => string) {
  if (kind === "offline") return t("offline");
  if (kind === "littleskin") return t("littleskin");
  return t("microsoft");
}

export function AccountsPage() {
  const { t } = useI18n();
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [offlineName, setOfflineName] = useState("");
  const [lsEmail, setLsEmail] = useState("");
  const [lsPassword, setLsPassword] = useState("");
  const [lsBusy, setLsBusy] = useState(false);
  const [login, setLogin] = useState<{
    user_code: string;
    verification_uri: string;
    device_code: string;
    interval: number;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    api.listAccounts().then(setAccounts).catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (!login) return;
    let cancelled = false;
    const tick = async () => {
      try {
        const acc = await api.pollMsLogin(login.device_code);
        if (cancelled) return;
        if (acc) {
          setAccounts(await api.listAccounts());
          setLogin(null);
          setStatus(acc.username);
          return;
        }
        setTimeout(tick, Math.max(login.interval, 2) * 1000);
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    };
    const timer = setTimeout(tick, login.interval * 1000);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [login]);

  return (
    <VStack gap={4} style={{ maxWidth: 640 }} className="euml-page">
      <VStack gap={2}>
        <Text type="display-3">{t("accountsTitle")}</Text>
        <Text color="secondary">{t("accountsHint")}</Text>
      </VStack>
      {error && <DismissibleBanner status="error" title={error} onDismiss={() => setError(null)} />}
      {status && <DismissibleBanner status="success" title={status} onDismiss={() => setStatus(null)} />}

      <Button
        label={t("msLogin")}
        variant="primary"
        onClick={async () => {
          setError(null);
          const res = await api.beginMsLogin();
          setLogin(res);
          await openUrl(res.verification_uri);
        }}
      />

      {login && (
        <DismissibleBanner
          status="info"
          title={`${login.user_code} — ${login.verification_uri}`}
          onDismiss={() => setLogin(null)}
        />
      )}

      <Card padding={3}>
        <VStack gap={2}>
          <Text weight="semibold">{t("littleskinLogin")}</Text>
          <Text color="secondary" type="supporting">
            {t("littleskinHint")}
          </Text>
          <HStack gap={2}>
            <TextInput label={t("littleskinEmail")} value={lsEmail} onChange={setLsEmail} />
            <TextInput
              label={t("littleskinPassword")}
              type="password"
              value={lsPassword}
              onChange={setLsPassword}
            />
            <Button
              label={lsBusy ? t("loading") : t("littleskinLogin")}
              isDisabled={lsBusy || !lsEmail.trim() || !lsPassword}
              isLoading={lsBusy}
              onClick={async () => {
                setLsBusy(true);
                setError(null);
                try {
                  setAccounts(await api.addLittleskinAccount(lsEmail.trim(), lsPassword));
                  setLsPassword("");
                  setStatus(t("littleskinOk"));
                } catch (e) {
                  setError(String(e));
                } finally {
                  setLsBusy(false);
                }
              }}
            />
          </HStack>
        </VStack>
      </Card>

      <Card padding={3}>
        <HStack gap={2}>
          <TextInput label={t("offlineName")} value={offlineName} onChange={setOfflineName} />
          <Button
            label={t("addOffline")}
            onClick={async () => {
              if (!offlineName.trim()) return;
              setAccounts(await api.addOfflineAccount(offlineName.trim()));
              setOfflineName("");
            }}
          />
        </HStack>
      </Card>

      <Card padding={0}>
        {accounts.map((a) => {
          return (
            <HStack
              key={a.id}
              justify="between"
              align="center"
              style={{ padding: "12px 14px", borderBottom: "1px solid var(--color-border)" }}
              gap={3}
            >
              <AccountAvatar account={a} />
              <VStack gap={0.5} style={{ flex: 1, minWidth: 0 }}>
                <Text weight="semibold">
                  {a.username} {a.active ? `(${t("active")})` : ""}
                </Text>
                <Text color="secondary" type="supporting">
                  {kindLabel(a.kind, t)} · {a.uuid}
                </Text>
              </VStack>
              <HStack gap={2}>
                {!a.active && (
                  <Button size="sm" label={t("use")} onClick={async () => setAccounts(await api.selectAccount(a.id))} />
                )}
                <Button
                  size="sm"
                  variant="destructive"
                  label={t("delete")}
                  onClick={async () => setAccounts(await api.removeAccount(a.id))}
                />
              </HStack>
            </HStack>
          );
        })}
        {accounts.length === 0 && (
          <div style={{ padding: 16 }}>
            <Text color="secondary">{t("none")}</Text>
          </div>
        )}
      </Card>
    </VStack>
  );
}
