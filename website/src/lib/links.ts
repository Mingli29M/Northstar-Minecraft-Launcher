/**
 * Social and donation destinations for the marketing site.
 * Set `href` when a page is ready; `null` keeps the labeled slot visible as “coming soon”.
 */
export type SiteLink = {
  id: string;
  /** Brand / product name shown in the UI (not localized). */
  name: string;
  href: string | null;
  kind: "social" | "donate";
};

/** Profile / community */
export const SOCIAL_LINKS: SiteLink[] = [
  {
    id: "github",
    name: "GitHub",
    href: "https://github.com/Mingli29M",
    kind: "social",
  },
  {
    id: "bilibili",
    name: "Bilibili",
    // Space URL when known (profile notes “Bilibili same name”).
    href: null,
    kind: "social",
  },
  {
    id: "discord",
    name: "Discord",
    href: null,
    kind: "social",
  },
];

/** Funding — Afdian is always listed as a dedicated slot. */
export const DONATE_LINKS: SiteLink[] = [
  {
    id: "afdian",
    name: "Afdian",
    // e.g. "https://afdian.com/a/your-slug"
    href: null,
    kind: "donate",
  },
  {
    id: "github-sponsors",
    name: "GitHub Sponsors",
    href: null,
    kind: "donate",
  },
];

export const ALL_EXTERNAL_LINKS = [...SOCIAL_LINKS, ...DONATE_LINKS];
