export const REPO = "https://github.com/Mingli29M/Northstar-Minecraft-Launcher";
/** All release assets (Windows / macOS / Linux) live on GitHub Releases — not on Pages. */
export const RELEASES = `${REPO}/releases`;
export const RELEASES_LATEST = `${REPO}/releases/latest`;
export const LICENSE_RAW = `${REPO}/blob/main/LICENSE`;
/** Website changelog (marketing site). Launcher notes: LAUNCHER_CHANGELOG_FILE. */
export const CHANGELOG_FILE = `${REPO}/blob/main/website/CHANGELOG.md`;
/** Desktop launcher changelog at repo root. */
export const LAUNCHER_CHANGELOG_FILE = `${REPO}/blob/main/CHANGELOG.md`;
export const SITE_URL = "https://mingli29m.github.io/Northstar-Minecraft-Launcher/";

export const LICENSE_TEXT = `Copyright (c) 2026 Northstar contributors. All rights reserved.

ALL RIGHTS RESERVED

This software and its source code are proprietary. No license is granted
to copy, modify, distribute, sublicense, or use this software except as
expressly permitted in writing by the copyright holders.

Permission to view this repository (if made public or shared privately)
does not constitute a grant of any rights under copyright or otherwise.

Third-party dependencies remain under their respective licenses
(see package-lock.json, Cargo.lock, and dependency notices).`;

export function openDownload() {
  window.location.href = RELEASES_LATEST;
}
