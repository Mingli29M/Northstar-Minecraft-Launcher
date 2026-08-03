/** Shared Minecraft version helpers (no bare trailing `-`). */

export function normalizeMcVersion(raw: string): string {
  const trimmed = raw.trim().replace(/-+$/g, "");
  // Require real pre/rc/snapshot after hyphen — never capture `1.21.1-Fabric` as `1.21.1-`
  const m = trimmed.match(/(\d+\.\d+(?:\.\d+)?(?:-(?:pre|rc|snapshot)\.?\d*)?)/i);
  if (m?.[1]) return m[1].replace(/-+$/g, "");
  const first = trimmed.split(/\s+/)[0] ?? "1.21.1";
  const [head, tail] = first.split("-");
  if (tail) {
    const t = tail.toLowerCase();
    if (!(t.startsWith("pre") || t.startsWith("rc") || t.startsWith("snapshot"))) {
      return head;
    }
  }
  return first.replace(/-+$/g, "") || "1.21.1";
}

export function chunkbasePlatform(gameVersion: string): string {
  return `java_${normalizeMcVersion(gameVersion).replace(/\./g, "_")}`;
}
