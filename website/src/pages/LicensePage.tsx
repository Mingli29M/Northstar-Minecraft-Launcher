import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { useI18n } from "../i18n";
import { LICENSE_RAW, LICENSE_TEXT } from "../lib/site";

export function LicensePage() {
  const { t } = useI18n();

  return (
    <div className="ns-site">
      <TopNav />
      <main className="ns-page-pad">
        <VStack gap={4}>
          <VStack gap={2}>
            <Text type="display-3">{t("licenseTitle")}</Text>
            <Text color="secondary">{t("licenseLead")}</Text>
          </VStack>

          <Banner
            status="error"
            title={t("licenseBannerTitle")}
            description={t("licenseBannerBody")}
          />

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">{t("licenseDocTitle")}</Text>
              <pre className="ns-license-block">{LICENSE_TEXT}</pre>
              <Text color="secondary">{t("licenseBindingNote")}</Text>
              <Button
                label={t("licenseViewGithub")}
                variant="secondary"
                onClick={() => {
                  window.location.href = LICENSE_RAW;
                }}
              />
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">{t("licenseBrandingTitle")}</Text>
              <Text color="secondary">{t("licenseBrandingBody")}</Text>
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">{t("licenseThirdTitle")}</Text>
              <Text color="secondary">{t("licenseThirdBody")}</Text>
            </VStack>
          </Card>
        </VStack>
      </main>
      <SiteFooter />
    </div>
  );
}
