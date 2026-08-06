/** Shared changelog types and current app version for the marketing site. */

export const APP_VERSION = "1.1.2";

export type ChangelogSection = {
  title: string;
  items: string[];
};

export type ChangelogEntry = {
  version: string;
  date: string;
  codename?: string;
  summary: string;
  sections: ChangelogSection[];
};
