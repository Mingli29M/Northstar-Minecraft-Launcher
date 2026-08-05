/** Embedded launcher changelog shown in Settings â†?About. */

export const APP_VERSION = "1.2.0";

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

export const LAUNCHER_CHANGELOG: ChangelogEntry[] = [
  {
    version: "1.2.0",
    date: "2026-08-05",
    summary:
      "ReqGuard Modrinth SoT, upgraded crash analysis, Temurin Java download, world backups, and Litematica notice.",
    sections: [
      {
        title: "ReqGuard",
        items: [
          "Background scans with an optional deep check of the real instance jars",
          "Canonical aliases and self-dependency filtering fix false Fabric API and mod-requires-itself reports",
          "Batched Modrinth hash lookup plus project-slug mapping recognizes manual and CurseForge installs",
          "Install missing by project id with required dependency chain; install-all action",
        ],
      },
      {
        title: "Crash analysis",
        items: [
          "Newest crash-report + latest.log with exception/frame extraction and stable hint codes",
          "Automatic full-lifetime process monitoring with persisted in-app analysis after abnormal exits",
        ],
      },
      {
        title: "Prerequisites",
        items: [
          "Java status + Adoptium Temurin download on Download â†?Game",
          "Modrinth installs pull required dependencies automatically",
        ],
      },
      {
        title: "Worlds",
        items: [
          "World backup fold-down with create/restore/delete",
          "Auto-backup on launch with keep-last-N pruning",
          "Litematica detection notice with schematics path",
        ],
      },
    ],
  },
  {
    version: "1.1.2",
    date: "2026-08-05",
    summary:
      "Settings sections, Appearance bug fix (opacity/font/wallpaper), shared target version, background drag-and-drop, and real loader icons.",
    sections: [
      {
        title: "Settings",
        items: [
          "Settings organized into General, Appearance, Java, Backups, and About sections",
          "About includes All Rights Reserved license summary plus the full launcher changelog",
          "Backups section exposes auto-backup toggles (snapshots take effect in 1.2.0+)",
        ],
      },
      {
        title: "Bug fixes",
        items: [
          "Appearance settings apply correctly â€?accent, wallpaper, font, UI scale, and panel opacity now target Astryx theme scopes so cards and text update",
          "Download / Servers Target version defaults to the instance selected on Launch and stays in sync",
          "Player head icons load reliably â€?fetched and cached in Rust (Crafthead / MC-Heads / Mojang+BMCLAPI) instead of Crafatar in the WebView",
        ],
      },
      {
        title: "Appearance",
        items: [
          "Panel opacity makes every card surface translucent over wallpaper (no OS acrylic)",
          "Background image: file picker + drag-and-drop dropzone",
          "Font choices load web fonts so the selector change is visible",
        ],
      },
      {
        title: "Branding",
        items: [
          "Real loader icons for Fabric, Vanilla, Quilt, and Forge (plus NeoForge/Paper/Purpur marks)",
        ],
      },
    ],
  },
  {
    version: "1.1.1",
    date: "2026-08-04",
    summary:
      "New Northstar app icons, and launcher changelog split from the website changelog.",
    sections: [
      {
        title: "Branding",
        items: [
          "Replaced nether-star window/installer icons with the new Northstar mark",
          "Overlay shard mark available for taller brand visuals",
        ],
      },
      {
        title: "Docs",
        items: [
          "Launcher and website changelogs are now separate (CHANGELOG.md vs website/CHANGELOG.md)",
        ],
      },
    ],
  },
  {
    version: "1.1.0",
    date: "2026-08-04",
    summary:
      "Northstar display rebrand, appearance settings, and Host/network polish.",
    sections: [
      {
        title: "Branding",
        items: [
          "User-facing product name is Northstar (data folder remains %APPDATA%\\euml for install stability)",
          "Window title, User-Agent, console titles, and Host MOTD/strings updated",
        ],
      },
      {
        title: "Appearance",
        items: [
          "Settings â†?Appearance: accent color, background color/image, font family, UI scale",
          "Live CSS preview; persisted in settings.json",
        ],
      },
      {
        title: "Host & network",
        items: [
          "UPnP â†?NAT-PMP â†?PCP port-map cascade with clearer join addresses",
          "Orphan Java reattach, port-in-use detection, and Host KeepAlive route fix",
        ],
      },
    ],
  },
  {
    version: "1.0.0",
    date: "2026-08-04",
    codename: "Northstar",
    summary:
      "First public release of Northstar â€?a desktop launcher for Minecraft Java Edition with dedicated hosting, Modrinth browsing, and multi-account support.",
    sections: [
      {
        title: "Launch experience",
        items: [
          "Home screen with large Start button and quick version picker",
          "Per-instance JVM arguments, memory sliders, and Java runtime detection",
          "Offline and online account switching without leaving the Launch page",
          "Live console stream while the game boots, with crash-log shortcuts",
        ],
      },
      {
        title: "Versions & loaders",
        items: [
          "Install Vanilla, Fabric, Quilt, Forge, NeoForge, and Paper/Purpur profiles",
          "Smarter Forge / NeoForge detection so instances no longer show as Vanilla",
          "Trailing dash stripped from version ids so Modrinth search works on 1.21.x builds",
          "One-click loader reinstall and libraries/assets repair from instance settings",
        ],
      },
      {
        title: "Mods & content",
        items: [
          "Browse, search, and install mods/modpacks from Modrinth inside the app",
          "ReqGuard dependency scan before launch â€?catch missing Fabric API / libraries early",
          "Bulk mod update check with selective apply",
          "Config editor with human-readable labels and grouped sections",
        ],
      },
      {
        title: "Multiplayer & hosting",
        items: [
          "Saved servers list with ping status and quick-join",
          "Built-in dedicated server host manager (start/stop, console, send commands)",
          "Live host stats: players online, TPS/MSPT probes, and resource meters (Windows)",
          "UPnP helpers for opening game ports on supported routers",
        ],
      },
      {
        title: "Accounts & localization",
        items: [
          "Offline accounts with stable generated UUIDs",
          "LittleSkin (authlib-injector) account support for third-party skins",
          "UI languages: English, ç®€ä½“ä¸­æ–? and Deutsch",
          "Locale preference stored in launcher settings across restarts",
        ],
      },
      {
        title: "Desktop packaging",
        items: [
          "Native installers for Windows (NSIS + MSI), macOS (Apple Silicon + Intel DMG), and Linux (AppImage, deb, rpm)",
          "macOS builds ad-hoc signed by default; Developer ID + notarization when Apple CI secrets are configured",
          "GitHub Actions publish workflow attaches artifacts to tagged releases",
        ],
      },
      {
        title: "Fixes since 0.1.0 preview",
        items: [
          "Non-Windows Rust build no longer fails on host_stats borrow-after-move",
          "macOS Intel CI builds on macos-15-intel instead of broken ARM cross-compile",
          "Minecraft news panel tolerates slow or failed patch-notes fetches",
          "Config form no longer leaks raw debug/dev key names into labels",
        ],
      },
    ],
  },
  {
    version: "0.1.0",
    date: "2026-08-03",
    codename: "Preview",
    summary: "Internal preview that established the core launcher shell and CI publish path.",
    sections: [
      {
        title: "Added",
        items: [
          "Launch screen with version select and Start button",
          "Minecraft Java news & patch notes under News",
          "Settings â†?About with embedded launcher changelog",
          "Config editor, ReqGuard, Modrinth browse, LittleSkin accounts",
          "Multi-language UI scaffolding (en / zh / de)",
        ],
      },
      {
        title: "Known limitations",
        items: [
          "macOS Gatekeeper required xattr workaround without ad-hoc signing",
          "Host CPU/RAM meters Windows-only",
          "Preview builds only â€?not a public release channel",
        ],
      },
    ],
  },
];
