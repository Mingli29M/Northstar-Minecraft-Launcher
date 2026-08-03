import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { NewsPanel } from "../components/NewsPanel";
import { useI18n } from "../i18n";

export function NewsPage() {
  const { t } = useI18n();
  return (
    <VStack gap={4} className="euml-page" style={{ maxWidth: 900 }}>
      <VStack gap={1}>
        <Text type="display-3">{t("navNews")}</Text>
        <Text color="secondary">{t("newsPageHint")}</Text>
      </VStack>
      <NewsPanel />
    </VStack>
  );
}
