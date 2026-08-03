import type { Instance, LoaderKind } from "./types";

function tokens(s: string): string[] {
  return s.toLowerCase().split(/[^a-z0-9]+/).filter(Boolean);
}

function hasLoaderToken(parts: string[], needle: string): boolean {
  return parts.some(
    (p) => p === needle || p.startsWith(`${needle}loader`) || p.startsWith(`${needle}mod`),
  );
}

function isForgeToken(parts: string[]): boolean {
  const cleaned = parts.filter((p) => p !== "neoforge" && p !== "forgified");
  return cleaned.some((p) => p === "forge" || p === "minecraftforge" || p.startsWith("forgeloader"));
}

/** Only used when persisted loader is vanilla. Fabric/Quilt beat bare "forge". */
export function effectiveLoader(inst: Pick<Instance, "name" | "game_version" | "loader">): LoaderKind {
  if (inst.loader !== "vanilla") return inst.loader;
  const parts = tokens(`${inst.name} ${inst.game_version}`);
  if (parts.includes("neoforge") || (parts.includes("neo") && parts.includes("forge"))) {
    return "neoforge";
  }
  if (hasLoaderToken(parts, "fabric")) return "fabric";
  if (hasLoaderToken(parts, "quilt")) return "quilt";
  if (isForgeToken(parts)) return "forge";
  return "vanilla";
}
