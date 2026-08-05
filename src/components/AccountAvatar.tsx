import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { Account } from "../lib/types";

type Props = {
  account: Account;
  className?: string;
  sizeHint?: "sm" | "md" | "lg";
};

/** Player head loaded via Rust (cached local data URL) with initials fallback. */
export function AccountAvatar({ account, className, sizeHint = "md" }: Props) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setSrc(null);
    void api
      .resolveAccountAvatar(account.kind, account.uuid, account.username)
      .then((url) => {
        if (!cancelled) setSrc(url);
      })
      .catch(() => {
        if (!cancelled) setSrc(null);
      });
    return () => {
      cancelled = true;
    };
  }, [account.id, account.kind, account.uuid, account.username]);

  const cls =
    className ??
    (sizeHint === "sm"
      ? "euml-avatar euml-avatar--sm"
      : sizeHint === "lg"
        ? "euml-avatar euml-avatar--lg"
        : "euml-avatar");

  if (src) {
    return <img src={src} alt="" className={cls} loading="lazy" />;
  }

  return (
    <div className={cls} style={{ display: "grid", placeItems: "center", fontSize: sizeHint === "sm" ? 10 : 12 }}>
      {account.username.slice(0, 2).toUpperCase()}
    </div>
  );
}
