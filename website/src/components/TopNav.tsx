import { Link } from "react-router-dom";
import { useI18n } from "../i18n";
import type { Locale } from "../i18n/messages";

const BASE = import.meta.env.BASE_URL.replace(/\/?$/, "/");

function BrandMark({ className }: { className?: string }) {
  return (
    <span className={`ns-star-tint ${className ?? ""}`.trim()}>
      <img
        src={`${import.meta.env.BASE_URL}nether-star-16.png`}
        alt=""
        width={26}
        height={26}
        decoding="async"
      />
    </span>
  );
}

export function TopNav() {
  const { t, locale, setLocale } = useI18n();

  const primary = [
    { href: `${BASE}#capabilities`, label: t("navCapabilities") },
    { href: `${BASE}#compare`, label: t("navCompare") },
    { href: `${BASE}#connect`, label: t("navConnect") },
  ];

  const secondary = [
    { href: "/changelog", label: t("navChangelog") },
    { href: "/about", label: t("navAbout") },
    { href: "/license", label: t("navLicense") },
  ];

  return (
    <header className="ns-topnav">
      <div className="ns-topnav-bar">
        <Link to="/" className="ns-topnav-brand">
          <BrandMark className="ns-topnav-mark" />
          <span>{t("brand")}</span>
        </Link>

        <nav className="ns-topnav-primary" aria-label="Primary">
          {primary.map((l) => (
            <a key={l.href} href={l.href}>
              {l.label}
            </a>
          ))}
        </nav>

        <div className="ns-topnav-end">
          <nav className="ns-topnav-secondary" aria-label="Secondary">
            {secondary.map((l) => (
              <Link key={l.href} to={l.href}>
                {l.label}
              </Link>
            ))}
          </nav>
          <a className="ns-nav-cta" href={`${BASE}#download`}>
            {t("navDownload")}
          </a>
          <label className="ns-lang">
            <span className="ns-visually-hidden">{t("langLabel")}</span>
            <select
              value={locale}
              aria-label={t("langLabel")}
              onChange={(e) => setLocale(e.target.value as Locale)}
            >
              <option value="en">{t("langEn")}</option>
              <option value="zh">{t("langZh")}</option>
              <option value="de">{t("langDe")}</option>
            </select>
          </label>
        </div>

        <nav className="ns-topnav-more" aria-label="More">
          {primary.map((l) => (
            <a key={`m-${l.href}`} href={l.href}>
              {l.label}
            </a>
          ))}
          {secondary.map((l) => (
            <Link key={`m-${l.href}`} to={l.href}>
              {l.label}
            </Link>
          ))}
        </nav>
      </div>
    </header>
  );
}
