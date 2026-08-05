import { Link } from "react-router-dom";
import { useI18n } from "../i18n";
import { DONATE_LINKS, SOCIAL_LINKS, type SiteLink } from "../lib/links";
import { CHANGELOG_FILE, LICENSE_RAW, REPO, SITE_URL } from "../lib/site";

function FooterLinks({ links }: { links: SiteLink[] }) {
  const { t } = useI18n();
  return (
    <ul>
      {links.map((link) => {
        const label = link.id === "afdian" ? t("linkAfdian") : link.name;
        return (
          <li key={link.id}>
            {link.href ? (
              <a href={link.href} target="_blank" rel="noreferrer">
                {label}
              </a>
            ) : (
              <span className="ns-footer-soon">
                {label} ({t("linkSoon")})
              </span>
            )}
          </li>
        );
      })}
    </ul>
  );
}

export function SiteFooter() {
  const { t } = useI18n();
  return (
    <footer className="ns-footer">
      <div className="ns-footer-inner">
        <div className="ns-footer-grid">
          <div className="ns-footer-col">
            <h3>{t("footerColSite")}</h3>
            <ul>
              <li>
                <Link to="/">{t("brand")}</Link>
              </li>
              <li>
                <Link to="/changelog">{t("footerChangelog")}</Link>
              </li>
              <li>
                <Link to="/about">{t("navAbout")}</Link>
              </li>
              <li>
                <a href={SITE_URL}>{t("footerColSiteLink")}</a>
              </li>
            </ul>
          </div>
          <div className="ns-footer-col">
            <h3>{t("connectSocials")}</h3>
            <FooterLinks links={SOCIAL_LINKS} />
          </div>
          <div className="ns-footer-col">
            <h3>{t("connectDonate")}</h3>
            <FooterLinks links={DONATE_LINKS} />
          </div>
          <div className="ns-footer-col">
            <h3>{t("footerColLegal")}</h3>
            <ul>
              <li>
                <Link to="/license">{t("footerLicense")}</Link>
              </li>
              <li>
                <a href={LICENSE_RAW}>{t("footerLicenseFile")}</a>
              </li>
              <li>
                <a href={CHANGELOG_FILE}>{t("changelogViewWebsiteMd")}</a>
              </li>
              <li>
                <a href={REPO}>{t("footerGithub")}</a>
              </li>
            </ul>
          </div>
        </div>

        <div className="ns-footer-meta">
          <p>
            {t("footerRights")} {t("footerDownloads")}
          </p>
          <p>{t("footerDisclaimer")}</p>
        </div>
      </div>
    </footer>
  );
}
