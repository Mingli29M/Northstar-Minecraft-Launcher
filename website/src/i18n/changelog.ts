import { useI18n } from "../i18n";
import type { ChangelogEntry } from "../lib/changelog";
import { APP_VERSION } from "../lib/changelog";

export { APP_VERSION };

export function useLocalizedWebsiteChangelog(): ChangelogEntry[] {
  const { t } = useI18n();
  return [
    {
      version: "1.1.2",
      date: "2026-08-05",
      summary: t("cl112Summary"),
      sections: [
        {
          title: t("cl112Sec1"),
          items: [t("cl112Sec1I1"), t("cl112Sec1I2")],
        },
      ],
    },
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

/** @deprecated Use useLocalizedWebsiteChangelog */
export function useLocalizedChangelog(): ChangelogEntry[] {
  return useLocalizedWebsiteChangelog();
}

export function useLocalizedLauncherChangelog(): ChangelogEntry[] {
  const { t } = useI18n();
  return [
    {
      version: "1.1.1",
      date: "2026-08-04",
      summary: t("lcl111Summary"),
      sections: [
        {
          title: t("lcl111Sec1"),
          items: [t("lcl111Sec1I1"), t("lcl111Sec1I2")],
        },
        {
          title: t("lcl111Sec2"),
          items: [t("lcl111Sec2I1")],
        },
      ],
    },
    {
      version: "1.1.0",
      date: "2026-08-04",
      summary: t("lcl110Summary"),
      sections: [
        {
          title: t("lcl110Sec1"),
          items: [t("lcl110Sec1I1"), t("lcl110Sec1I2")],
        },
        {
          title: t("lcl110Sec2"),
          items: [t("lcl110Sec2I1"), t("lcl110Sec2I2")],
        },
        {
          title: t("lcl110Sec3"),
          items: [t("lcl110Sec3I1"), t("lcl110Sec3I2")],
        },
      ],
    },
    {
      version: "1.0.0",
      date: "2026-08-04",
      codename: "Northstar",
      summary: t("lcl100Summary"),
      sections: [
        {
          title: t("lcl100Sec1"),
          items: [t("lcl100Sec1I1"), t("lcl100Sec1I2")],
        },
      ],
    },
    {
      version: "0.1.0",
      date: "2026-08-03",
      codename: "Preview",
      summary: t("lcl010Summary"),
      sections: [
        {
          title: t("lcl010Sec1"),
          items: [t("lcl010Sec1I1"), t("lcl010Sec1I2")],
        },
      ],
    },
  ];
}
