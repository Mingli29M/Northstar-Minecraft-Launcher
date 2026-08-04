import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { useI18n } from "../i18n";
import { CHANGELOG_FILE, REPO } from "../lib/site";

const IDLE_ROWS: [string, string, string, string, string | "pcl"][] = [
  ["Prism 11.0.3", "64.1", "58.0", "0.0", "Qt6 portable"],
  ["MultiMC stable", "110.9", "50.2", "0.0", "Qt5 lin64"],
  ["HMCL 3.16.3", "296.5", "217.1", "0.3", "JavaFX jar"],
  ["Northstar 1.1.0", "337.0", "99.8", "0.3", "Tauri/WebKitGTK after opt"],
  ["PCL CE 2.15.0", "—", "—", "—", "pcl"],
];

export function AboutPage() {
  const { t } = useI18n();
  const headers = [
    t("aboutColLauncher"),
    t("aboutColWs"),
    t("aboutColPrivate"),
    t("aboutColCpu"),
    t("aboutColNotes"),
  ];

  return (
    <div className="ns-site">
      <TopNav />
      <main className="ns-page-pad">
        <VStack gap={4}>
          <VStack gap={2}>
            <Text type="display-3">{t("aboutTitle")}</Text>
            <Text color="secondary">{t("aboutLead")}</Text>
          </VStack>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">{t("aboutWhatTitle")}</Text>
              <Text>{t("aboutWhatBody")}</Text>
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">{t("aboutIndepTitle")}</Text>
              <Text color="secondary">{t("aboutIndepBody")}</Text>
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">{t("aboutResTitle")}</Text>
              <Text color="secondary">
                {t("aboutResBody")}{" "}
                <a href={`${REPO}/blob/main/BENCHMARKS.md`}>BENCHMARKS.md</a>.
              </Text>
              <div style={{ overflowX: "auto" }}>
                <table
                  style={{
                    width: "100%",
                    borderCollapse: "collapse",
                    fontSize: 13,
                    lineHeight: 1.45,
                  }}
                >
                  <thead>
                    <tr>
                      {headers.map((h, i) => (
                        <th
                          key={h}
                          style={{
                            textAlign: i === 0 || i === 4 ? "left" : "right",
                            padding: "6px 8px",
                            borderBottom: "1px solid var(--color-border, #ddd)",
                          }}
                        >
                          {h}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {IDLE_ROWS.map((row) => (
                      <tr key={row[0]}>
                        {row.map((cell, i) => (
                          <td
                            key={i}
                            style={{
                              textAlign: i === 0 || i === 4 ? "left" : "right",
                              padding: "6px 8px",
                              borderBottom: "1px solid var(--color-border, #eee)",
                              fontWeight:
                                row[0].startsWith("Northstar") && i === 0 ? 600 : undefined,
                            }}
                          >
                            {cell === "pcl" ? t("aboutNotePcl") : cell}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <Text color="secondary">{t("aboutResOutro")}</Text>
            </VStack>
          </Card>

          <Banner
            status="warning"
            title={t("aboutUnofficialTitle")}
            description={t("aboutUnofficialBody")}
          />

          <div className="ns-cta-row">
            <Button
              label={t("aboutBtnGithub")}
              variant="secondary"
              onClick={() => {
                window.location.href = REPO;
              }}
            />
            <Button
              label={t("aboutBtnChangelog")}
              variant="secondary"
              onClick={() => {
                window.location.href = CHANGELOG_FILE;
              }}
            />
          </div>
        </VStack>
      </main>
      <SiteFooter />
    </div>
  );
}
