/** Shared Minecraft version helpers (no bare trailing `-`). */

/** True for strings that can plausibly be Mojang version ids. */
export function isPlausibleMcVersion(raw: string): boolean {
  const v = raw.trim();
  if (!v || v.length > 32) return false;
  if (/^\d{2,}w\d{1,2}[a-z]$/i.test(v)) return true;
  if (/^\d+\.\d+(?:\.\d+)?(?:-(?:pre|rc|snapshot)\.?\d*)?$/i.test(v)) return true;
  if (/^[abc]\d+(?:\.\d+)+(?:[._][0-9a-z]+)*$/i.test(v)) return true;
  return false;
}

/**
 * Pull a real Minecraft version id out of messy strings like "1.21.1-Fabric NNEW".
 * Returns "" when nothing plausible is found — never a pack name like "Create：Complete".
 */
export function normalizeMcVersion(raw: string): string {
  const trimmed = raw.trim().replace(/-+$/g, "");
  if (!trimmed) return "";
  if (isPlausibleMcVersion(trimmed)) return trimmed;

  const weekly = trimmed.match(/(\d{2,}w\d{1,2}[a-z])/i);
  if (weekly?.[1]) return weekly[1];

  // Require real pre/rc/snapshot after hyphen — never capture `1.21.1-Fabric` as `1.21.1-`
  const m = trimmed.match(/(\d+\.\d+(?:\.\d+)?(?:-(?:pre|rc|snapshot)\.?\d*)?)/i);
  if (m?.[1]) {
    const v = m[1].replace(/-+$/g, "");
    if (isPlausibleMcVersion(v)) return v;
  }

  const classic = trimmed.match(/\b([abc]\d+(?:\.\d+)+(?:[._][0-9a-z]+)*)\b/i);
  if (classic?.[1]) return classic[1];

  return "";
}

/**
 * Map a Mojang version id to Chunkbase's `platform=` token.
 * Chunkbase buckets several patch releases (e.g. 1.21–1.21.1 → java_1_21).
 */
export function chunkbasePlatform(gameVersion: string): string {
  const v = normalizeMcVersion(gameVersion) || "1.21.1";
  const m = v.match(/^(\d+)\.(\d+)(?:\.(\d+))?$/);
  if (!m) return `java_${v.replace(/\./g, "_")}`;
  const major = Number(m[1]);
  const minor = Number(m[2]);
  const patch = m[3] != null ? Number(m[3]) : 0;

  // Prefer known Chunkbase Java buckets (see seed-map version dropdown).
  if (major === 1 && minor === 21) {
    if (patch <= 1) return "java_1_21";
    if (patch <= 3) return "java_1_21_2";
    if (patch === 4) return "java_1_21_4";
    if (patch === 5) return "java_1_21_5";
    if (patch <= 8) return "java_1_21_6";
    return "java_1_21_9";
  }
  if (major === 1 && minor === 20) return "java_1_20";
  if (major === 1 && minor === 19) {
    if (patch >= 3) return "java_1_19_3";
    return "java_1_19";
  }
  if (major === 1 && minor >= 7 && minor <= 18) {
    return `java_1_${minor}`;
  }
  return `java_${v.replace(/\./g, "_")}`;
}
