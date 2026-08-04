import type { LucideIcon } from "lucide-react";
import { Code2, HeartHandshake, MessageCircle, Tv } from "lucide-react";
import { useI18n } from "../i18n";
import { DONATE_LINKS, SOCIAL_LINKS, type SiteLink } from "../lib/links";

const ICONS: Record<string, LucideIcon> = {
  github: Code2,
  bilibili: Tv,
  discord: MessageCircle,
  afdian: HeartHandshake,
  "github-sponsors": HeartHandshake,
};

function LinkRow({ link }: { link: SiteLink }) {
  const { t } = useI18n();
  const Icon = ICONS[link.id] ?? HeartHandshake;
  const label = link.id === "afdian" ? t("linkAfdian") : link.name;

  if (link.href) {
    return (
      <a className="ns-connect-link" href={link.href} target="_blank" rel="noreferrer">
        <Icon size={18} strokeWidth={1.75} aria-hidden />
        <span>{label}</span>
      </a>
    );
  }

  return (
    <span className="ns-connect-link ns-connect-link-soon" title={t("linkSoonHint")}>
      <Icon size={18} strokeWidth={1.75} aria-hidden />
      <span>
        {label}
        <span className="ns-connect-soon"> — {t("linkSoon")}</span>
      </span>
    </span>
  );
}

export function ConnectSection() {
  const { t } = useI18n();

  return (
    <section className="ns-section ns-connect" id="connect" aria-labelledby="connect-heading">
      <div className="ns-section-head">
        <h2 id="connect-heading">{t("connectTitle")}</h2>
        <p className="ns-section-lead">{t("connectLead")}</p>
      </div>

      <div className="ns-connect-grid">
        <div className="ns-connect-col">
          <h3>{t("connectSocials")}</h3>
          <ul className="ns-connect-list">
            {SOCIAL_LINKS.map((link) => (
              <li key={link.id}>
                <LinkRow link={link} />
              </li>
            ))}
          </ul>
        </div>
        <div className="ns-connect-col">
          <h3>{t("connectDonate")}</h3>
          <ul className="ns-connect-list">
            {DONATE_LINKS.map((link) => (
              <li key={link.id}>
                <LinkRow link={link} />
              </li>
            ))}
          </ul>
          <p className="ns-footnote">{t("connectDonateNote")}</p>
        </div>
      </div>
    </section>
  );
}
