/** Embedded launcher changelog shown in Settings → About. */

export const APP_VERSION = "1.3.0";

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
    version: "1.3.0",
    date: "2026-08-07",
    summary:
      "Translucent window over the desktop, reliable Start position, Modrinth install/detail upgrades, and a Litematica tab.",
    sections: [
      {
        title: "Appearance",
        items: [
          "Window opacity uses real OS translucency so the desktop wallpaper can show through",
          "Cards, lists, and menus stay solid for readability; only the window wash fades",
          "Opacity range widened to 20–100%",
        ],
      },
      {
        title: "Launch",
        items: [
          "Fix Start button position not applying — Top/Bottom controls work in compact and full layouts",
          "Compact mode no longer locks or forces Start position",
        ],
      },
      {
        title: "Mods",
        items: [
          "Modrinth install version picker with prerequisites, nested dependency install, and Installed badge",
          "Detail page Markdown, gallery lightbox, and fuller versions list",
          "Mod installs report download progress in the dock",
        ],
      },
      {
        title: "Versions",
        items: [
          "Litematica tab to import, export, delete, and open schematics",
          "Worlds and schematics use separate list state",
        ],
      },
      {
        title: "Worlds",
        items: [
          "Chunkbase seed map opens in the system browser (iframe / in-app window not viable)",
        ],
      },
    ],
  },
  {
    version: "1.2.4",
    date: "2026-08-06",
    summary:
      "Host panel reliability, Terracotta multiplayer, visible download progress, and a safe-exit prompt.",
    sections: [
      {
        title: "Host",
        items: [
          "Windows CREATE_NO_WINDOW on helper tools and server Java so Start no longer opens many terminals",
          "Defer public-IP / UPnP discovery so Host opens without a long freeze",
          "Keep status polling when Stopped; recover live servers via port + orphan scan",
          "Standout Running / Stopped badge with optional PID",
        ],
      },
      {
        title: "Terracotta",
        items: [
          "New Terracotta tab downloads and runs the official unmodified 0.4.2 sidecar (HMCL HTTP IPC)",
          "Host / join room codes without Cargo-linking Terracotta; Northstar license unchanged",
          "Fix 'Access is denied (os error 5)' on Reinstall — the install folder is no longer locked as the sidecar's working directory",
          "Fix false 'errored' status — state polls retry before being treated as a failure",
          "Downloads are SHA-512 verified across four mirrors, so a truncated archive is replaced instead of cached",
          "Readable connection states and explained Terracotta errors instead of raw state names",
          "Fix Start never sticking — on Windows --hmcl only re-spawns the binary detached and exits, so the launcher was watching the wrong process and declared the sidecar dead seconds after it started",
          "Fix sidecars surviving Stop and quit, which left one holding Terracotta's machine-wide lock and a file lock on the executable (the real cause of repeat 'os error 5' reinstalls)",
          "Start now attaches to a sidecar that is already running, and the launcher re-attaches to one its own previous session left behind",
          "A failed start prints the tail of Terracotta's own log, which is otherwise written only to a file",
          "Visible AGPL attribution in the tab and Settings → About",
        ],
      },
      {
        title: "Launch",
        items: [
          "Fix a newly downloaded or imported instance not showing up in the start menu until restart",
          "Compact Launch page and Start button position moved to Settings → Appearance; compact docks Start at the bottom",
          "Reject pack names like Create：Complete as Minecraft versions; prepare asks you to set a real version instead",
          "With local metadata scan off, jars are no longer unzipped; with both scans off, Launch skips ReqGuard work",
        ],
      },
      {
        title: "Downloads",
        items: [
          "Large single-file downloads (Terracotta, Java, server jars, plugins) now report progress instead of running silently",
          "Progress dock shows transferred size and speed for single-file transfers",
        ],
      },
      {
        title: "Reliability",
        items: [
          "Closing the launcher while a server or Terracotta runs now asks first and names what would stop",
        ],
      },
      {
        title: "Localization",
        items: [
          "Removed Chinese text that leaked into the English and German UI",
          "Offline-account validation error is localized instead of hard-coded Chinese",
        ],
      },
    ],
  },
  {
    version: "1.2.3",
    date: "2026-08-06",
    summary:
      "Security hardening: tightened asset:// file scope and avatar download SSRF controls.",
    sections: [
      {
        title: "Security",
        items: [
          "Asset protocol scope limited to app wallpaper and avatar cache dirs (removed $HOME/** and catch-all **)",
          "Background Browse / drag-and-drop copies images into the app wallpapers folder via import_background_image",
          "Settings free-text background field rejects absolute filesystem paths (remote / data: URLs still allowed)",
          "Avatar fetches: HTTPS host allowlist, no redirects, hex UUID checks, and a 2 MiB response size cap (SSRF / DoS mitigation)",
        ],
      },
    ],
  },
  {
    version: "1.2.2",
    date: "2026-08-05",
    summary: "ReqGuard install-button fixes and experimental local metadata scan toggle.",
    sections: [
      {
        title: "ReqGuard",
        items: [
          "Install / Install-all buttons report errors and re-scan with the active Settings modes",
          "Install-all uses deep Modrinth SoT issues instead of a local-only pass",
          "Local metadata scan is Experimental and off by default; Play is gated only when it is enabled",
        ],
      },
    ],
  },
  {
    version: "1.2.1",
    date: "2026-08-05",
    summary: "Hangar plugin installs for Paper/Purpur Host servers and ReqGuard polish.",
    sections: [
      {
        title: "Host plugins",
        items: [
          "Paper/Purpur dedicated servers get a Plugins tab with Hangar search and install",
          "Enable, disable, or delete installed plugin jars under runtime/plugins/",
        ],
      },
      {
        title: "ReqGuard",
        items: [
          "Install-all missing dependencies remains available on Launch and Versions",
          "Modrinth API User-Agent updated to Northstar/1.2.1",
        ],
      },
    ],
  },
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
          "Java status + Adoptium Temurin download on Download → Game",
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
          "Appearance settings apply correctly — accent, wallpaper, font, UI scale, and panel opacity now target Astryx theme scopes so cards and text update",
          "Download / Servers Target version defaults to the instance selected on Launch and stays in sync",
          "Player head icons load reliably — fetched and cached in Rust (Crafthead / MC-Heads / Mojang+BMCLAPI) instead of Crafatar in the WebView",
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
          "Settings → Appearance: accent color, background color/image, font family, UI scale",
          "Live CSS preview; persisted in settings.json",
        ],
      },
      {
        title: "Host & network",
        items: [
          "UPnP → NAT-PMP → PCP port-map cascade with clearer join addresses",
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
      "First public release of Northstar — a desktop launcher for Minecraft Java Edition with dedicated hosting, Modrinth browsing, and multi-account support.",
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
          "ReqGuard dependency scan before launch — catch missing Fabric API / libraries early",
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
          "UI languages: English, 简体中文, and Deutsch",
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
          "Settings → About with embedded launcher changelog",
          "Config editor, ReqGuard, Modrinth browse, LittleSkin accounts",
          "Multi-language UI scaffolding (en / zh / de)",
        ],
      },
      {
        title: "Known limitations",
        items: [
          "macOS Gatekeeper required xattr workaround without ad-hoc signing",
          "Host CPU/RAM meters Windows-only",
          "Preview builds only — not a public release channel",
        ],
      },
    ],
  },
];
