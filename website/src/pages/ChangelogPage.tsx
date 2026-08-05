import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { useI18n } from "../i18n";
import {
  useLocalizedLauncherChangelog,
  useLocalizedWebsiteChangelog,
} from "../i18n/changelog";
import type { ChangelogEntry } from "../lib/changelog";
import { APP_VERSION } from "../lib/changelog";
import { CHANGELOG_FILE, LAUNCHER_CHANGELOG_FILE } from "../lib/site";

function ChangelogColumn({
  title,
  mdLabel,
  mdHref,
  entries,
}: {
  title: string;
  mdLabel: string;
  mdHref: string;
  entries: ChangelogEntry[];
}) {
  return (
    <section className="ns-changelog-col">
      <header className="ns-changelog-col-head">
        <h2>{title}</h2>
        <a href={mdHref}>{mdLabel}</a>
      </header>
      <div className="ns-changelog">
        {entries.map((entry) => (
          <article key={entry.version} className="ns-changelog-entry">
            <header className="ns-changelog-head">
              <h3>
                v{entry.version}
                {entry.codename ? ` — ${entry.codename}` : ""}
              </h3>
              <time dateTime={entry.date}>{entry.date}</time>
            </header>
            <p className="ns-changelog-summary">{entry.summary}</p>
            {entry.sections.map((section) => (
              <div key={section.title} className="ns-changelog-section">
                <h4>{section.title}</h4>
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
      <div className="ns-cta-row" style={{ marginTop: "1.25rem" }}>
        <Button
          label={mdLabel}
          variant="secondary"
          onClick={() => {
            window.location.href = mdHref;
          }}
        />
      </div>
    </section>
  );
}

export function ChangelogPage() {
  const { t } = useI18n();
  const websiteEntries = useLocalizedWebsiteChangelog();
  const launcherEntries = useLocalizedLauncherChangelog();

  return (
    <div className="ns-site">
      <TopNav />
      <main className="ns-page-pad ns-page-wide">
        <header className="ns-page-header">
          <h1>{t("changelogTitle")}</h1>
          <p className="ns-page-lead">
            {t("changelogLeadBefore")} {APP_VERSION}
            {t("changelogLeadAfter")}
          </p>
        </header>

        <Banner
          status="info"
          title={t("changelogCurrent", { version: APP_VERSION })}
          description={t("changelogBannerBody")}
        />

        <div className="ns-changelog-split">
          <ChangelogColumn
            title={t("changelogColWebsite")}
            mdLabel={t("changelogViewWebsiteMd")}
            mdHref={CHANGELOG_FILE}
            entries={websiteEntries}
          />
          <ChangelogColumn
            title={t("changelogColLauncher")}
            mdLabel={t("changelogViewLauncherMd")}
            mdHref={LAUNCHER_CHANGELOG_FILE}
            entries={launcherEntries}
          />
        </div>
      </main>
      <SiteFooter />
    </div>
  );
}
