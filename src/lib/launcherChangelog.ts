/** Embedded launcher changelog shown in Settings → About. */

export const APP_VERSION = "0.1.0";

export type ChangelogEntry = {
  version: string;
  date: string;
  highlights: string[];
};

export const LAUNCHER_CHANGELOG: ChangelogEntry[] = [
  {
    version: "0.1.0",
    date: "2026-08-03",
    highlights: [
      "PCL/HMCL-style version select and Start button on Launch",
      "Minecraft Java news & patch notes",
      "Config editor: readable labels + section groups",
      "Stronger Forge / NeoForge detection",
      "ReqGuard, Modrinth, LittleSkin, multi-language UI",
    ],
  },
];
