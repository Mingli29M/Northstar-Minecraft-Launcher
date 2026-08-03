import type { Account, LoaderKind } from "./types";

/** Player head / skin preview URL for an account. */
export function accountAvatarUrl(account: Account): string | null {
  const uuid = account.uuid?.replace(/-/g, "") ?? "";
  if (account.kind === "littleskin") {
    return `https://littleskin.cn/avatar/player/${encodeURIComponent(account.username)}`;
  }
  if (account.kind === "microsoft" && uuid && !/^0+$/.test(uuid)) {
    return `https://crafatar.com/avatars/${uuid}?overlay=true&size=64`;
  }
  return null;
}

/** Official-ish loader mark (inline SVG data URL). */
export function loaderIconSrc(loader: LoaderKind): string {
  switch (loader) {
    case "fabric":
      // Fabric spool mark (simplified)
      return (
        "data:image/svg+xml," +
        encodeURIComponent(
          `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="12" fill="#dbb69b"/><path fill="#1c1917" d="M32 10c-8 0-14 4-14 12v4c0 3 2 5 5 6v16c0 4 4 8 9 8s9-4 9-8V32c3-1 5-3 5-6v-4c0-8-6-12-14-12zm0 6c4 0 7 2 7 6v2H25v-2c0-4 3-6 7-6zm0 36c-2 0-4-1-4-3V34h8v15c0 2-2 3-4 3z"/></svg>`,
        )
      );
    case "quilt":
      return (
        "data:image/svg+xml," +
        encodeURIComponent(
          `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="12" fill="#c7b3eb"/><path fill="#1c1917" d="M18 18h12v12H18zm16 0h12v12H34zM18 34h12v12H18zm16 0h12v12H34z"/></svg>`,
        )
      );
    case "forge":
      return (
        "data:image/svg+xml," +
        encodeURIComponent(
          `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="12" fill="#6a7a8a"/><path fill="#fff" d="M14 40h36v6H14zm4-8h28v6H18zm4-8h20v6H22zm4-8h12v6H26z"/></svg>`,
        )
      );
    case "neoforge":
      return (
        "data:image/svg+xml," +
        encodeURIComponent(
          `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="12" fill="#e86339"/><path fill="#fff" d="M18 18h28v8H18zm0 12h20v8H18zm0 12h28v8H18z"/></svg>`,
        )
      );
    default:
      return (
        "data:image/svg+xml," +
        encodeURIComponent(
          `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="12" fill="#5b8c5a"/><circle cx="32" cy="32" r="14" fill="#fff" opacity=".9"/></svg>`,
        )
      );
  }
}

export function loaderShort(loader: LoaderKind): string {
  switch (loader) {
    case "fabric":
      return "Fab";
    case "quilt":
      return "Qui";
    case "forge":
      return "For";
    case "neoforge":
      return "Neo";
    default:
      return "Van";
  }
}
