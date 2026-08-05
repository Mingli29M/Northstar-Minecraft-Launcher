import type { DedicatedLoader, LoaderKind } from "./types";
import vanillaIcon from "../assets/loaders/vanilla.png";
import fabricIcon from "../assets/loaders/fabric.png";
import quiltIcon from "../assets/loaders/quilt.png";
import forgeIcon from "../assets/loaders/forge.png";
import neoforgeIcon from "../assets/loaders/neoforge.svg";
import paperIcon from "../assets/loaders/paper.svg";
import purpurIcon from "../assets/loaders/purpur.svg";

/** Bundled loader mark (asset URL). Order preference: Fabric, Vanilla, Quilt, Forge. */
export function loaderIconSrc(loader: LoaderKind | DedicatedLoader | string): string {
  switch (loader) {
    case "fabric":
      return fabricIcon;
    case "quilt":
      return quiltIcon;
    case "forge":
      return forgeIcon;
    case "neoforge":
      return neoforgeIcon;
    case "paper":
      return paperIcon;
    case "purpur":
      return purpurIcon;
    default:
      return vanillaIcon;
  }
}

export function loaderShort(loader: LoaderKind | DedicatedLoader | string): string {
  switch (loader) {
    case "fabric":
      return "Fab";
    case "quilt":
      return "Qui";
    case "forge":
      return "For";
    case "neoforge":
      return "Neo";
    case "paper":
      return "Pap";
    case "purpur":
      return "Pur";
    default:
      return "Van";
  }
}
