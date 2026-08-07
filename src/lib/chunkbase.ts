import { chunkbasePlatform } from "./mcVersion";

export type ChunkbaseDimension = "overworld" | "nether" | "end";

/** Build a Chunkbase Seed Map URL for the given world seed + instance game version. */
export function buildChunkbaseSeedMapUrl(
  seed: string,
  gameVersion: string,
  dimension: ChunkbaseDimension = "overworld",
): string {
  const trimmed = seed.trim();
  const platform = chunkbasePlatform(gameVersion);
  const hash = new URLSearchParams({
    seed: trimmed || "0",
    platform,
    dimension,
    x: "0",
    z: "0",
    zoom: "0.5",
  });
  // Chunkbase reads state from the hash fragment, not the query string.
  return `https://www.chunkbase.com/apps/seed-map#${hash.toString()}`;
}
