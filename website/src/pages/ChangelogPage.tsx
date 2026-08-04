import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { useI18n } from "../i18n";
import { useLocalizedChangelog } from "../i18n/changelog";
import { APP_VERSION } from "../lib/changelog";
import { CHANGELOG_FILE } from "../lib/site";

export function ChangelogPage() {
  const { t } = useI18n();
  const entries = useLocalizedChangelog();

  return (
    <div className="ns-site">
      <TopNav />
      <main className="ns-page-pad ns-page-wide">
        <header className="ns-page-header">
          <h1>{t("changelogTitle")}</h1>
          <p className="ns-page-lead">
            {t("changelogLeadBefore")} {APP_VERSION}
            {t("changelogLeadAfter")} <a href={CHANGELOG_FILE}>website/CHANGELOG.md</a>.
          </p>
        </header>

        <Banner
          status="info"
          title={t("changelogCurrent", { version: APP_VERSION })}
          description={t("changelogBannerBody")}
        />

        <div className="ns-changelog">
          {entries.map((entry) => (
            <article key={entry.version} className="ns-changelog-entry">
              <header className="ns-changelog-head">
                <h2>
                  v{entry.version}
                  {entry.codename ? ` — ${entry.codename}` : ""}
                </h2>
                <time dateTime={entry.date}>{entry.date}</time>
              </header>
              <p className="ns-changelog-summary">{entry.summary}</p>
              {entry.sections.map((section) => (
                <div key={section.title} className="ns-changelog-section">
                  <h3>{section.title}</h3>
                  <ul>
                    {section.items.map((item) => (
                      <li key={item}>{item}</li>
                    ))}
                  </ul>
                </div>
              ))}
            </article>
          ))}
        </div>

        <div className="ns-cta-row" style={{ marginTop: "1.5rem" }}>
          <Button
            label={t("changelogViewMd")}
            variant="secondary"
            onClick={() => {
              window.location.href = CHANGELOG_FILE;
            }}
          />
        </div>
      </main>
      <SiteFooter />
    </div>
  );
}
