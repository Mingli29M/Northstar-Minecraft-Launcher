import { useI18n } from "../i18n";
import type { ChangelogEntry } from "../lib/changelog";
import { APP_VERSION } from "../lib/changelog";

export { APP_VERSION };

export function useLocalizedChangelog(): ChangelogEntry[] {
  const { t } = useI18n();
  return [
    {
      version: "1.1.1",
      date: "2026-08-04",
      summary: t("cl111Summary"),
      sections: [
        {
          title: t("cl111Sec1"),
          items: [t("cl111Sec1I1"), t("cl111Sec1I2")],
        },
        {
          title: t("cl111Sec2"),
          items: [t("cl111Sec2I1")],
        },
      ],
    },
    {
      version: "1.1.0",
      date: "2026-08-04",
      summary: t("cl110Summary"),
      sections: [
        {
          title: t("cl110Sec1"),
          items: [t("cl110Sec1I1"), t("cl110Sec1I2")],
        },
      ],
    },
    {
      version: "1.0.0",
      date: "2026-08-04",
      summary: t("cl100Summary"),
      sections: [
        {
          title: t("cl100Sec1"),
          items: [t("cl100Sec1I1")],
        },
      ],
    },
  ];
}
