# Northstar Launcher Changelog

Launcher release notes for the desktop app (Tauri). Website marketing changes live in [`website/CHANGELOG.md`](website/CHANGELOG.md).

> **Split (1.1.1):** Launcher and website changelogs are maintained separately. This file covers the desktop launcher only.

## 1.1.2 — 2026-08-05

Settings sections, license/changelog in About, panel opacity, loader icons, and shared target-version selection.

### Settings
- Settings organized into General, Appearance, Java, Backups, and About sections
- About includes All Rights Reserved license summary plus the full launcher changelog
- Backups section exposes auto-backup toggles (snapshots take effect in 1.2.0+)

### Bug fixes
- **Appearance settings apply correctly** — accent, wallpaper, font, UI scale, and panel opacity now target Astryx theme scopes (`[data-astryx-theme]`), so cards and text actually update
- Download / Servers “Target version” defaults to the instance selected on Launch (and stays in sync when changed)

### Appearance
- Panel opacity (“画面透明度”) makes every card surface translucent over wallpaper (no OS acrylic)
- Background image: file picker + drag-and-drop dropzone (path/URL still supported)
- Font choices load web fonts so the selector change is visible without local installs
- Real loader icons: Fabric, Vanilla (grass), Quilt, Forge (anvil)

### Branding
- Bundled loader icons for Vanilla, Fabric, Quilt, Forge, NeoForge, Paper, and Purpur

## 1.1.1 — 2026-08-04

Branding icons refreshed; changelog split from the marketing site.

### Branding
- Replaced Minecraft-style nether-star app icons with the new geometric Northstar mark (window / installer icons)
- Overlay shard mark used for brand visuals that need the taller “star with overlay” treatment

### Docs
- Launcher and website changelogs are now separate (`CHANGELOG.md` vs `website/CHANGELOG.md`)

## 1.1.0 — 2026-08-04

Northstar display rebrand, appearance settings, and Host/network polish.

### Branding
- User-facing product name is **Northstar** (data folder remains `%APPDATA%\euml` for install stability)
- Window title, User-Agent, console titles, and Host MOTD/strings updated

### Appearance
- Settings → Appearance: accent color, background color/image, font family, UI scale
- Live CSS preview; persisted in `settings.json`

### Host & network
- UPnP → NAT-PMP → PCP port-map cascade with clearer join addresses
- Orphan Java reattach, port-in-use detection, and Host KeepAlive route fix

## 1.0.0 — Northstar — 2026-08-04

First public release of Northstar.

### Launch experience
- PCL/HMCL-style home screen with large Start button and quick version picker
- Per-instance JVM arguments, memory sliders, and Java runtime detection
- Offline and online account switching without leaving the Launch page
- Live console stream while the game boots, with crash-log shortcuts

### Versions & loaders
- Install Vanilla, Fabric, Quilt, Forge, NeoForge, and Paper/Purpur profiles
- Smarter Forge / NeoForge detection so instances no longer show as Vanilla
- Trailing dash stripped from version ids so Modrinth search works on 1.21.x builds
- One-click loader reinstall and libraries/assets repair from instance settings

### Mods & content
- Browse, search, and install mods/modpacks from Modrinth inside the app
- ReqGuard dependency scan before launch
- Bulk mod update check with selective apply
- Config editor with human-readable labels and Mod Menu–style section groups

### Multiplayer & hosting
- Saved servers list with ping status and quick-join
- Built-in dedicated server host manager (start/stop, console, send commands)
- Live host stats: players online, TPS/MSPT probes, and resource meters (Windows)
- UPnP helpers for opening game ports on supported routers

### Accounts & localization
- Offline accounts with stable generated UUIDs
- LittleSkin (authlib-injector) account support
- UI languages: English, 简体中文, and Deutsch

### Desktop packaging
- Native installers for Windows (NSIS + MSI), macOS (Apple Silicon + Intel DMG), and Linux (AppImage, deb, rpm)
- macOS: Developer ID + notarization when Apple CI secrets are set; otherwise ad-hoc signed
- GitHub Actions publish workflow on the `release` branch

### Fixes since 0.1.0 preview
- Non-Windows Rust build: host_stats borrow-after-move
- macOS Intel CI on `macos-15-intel` (no broken ARM cross-compile)
- Minecraft news panel tolerates slow/failed patch-notes fetches
- Config form no longer shows raw debug/dev key names

## 0.1.0 — Preview — 2026-08-03

### Added
- Launch screen with PCL/HMCL-style version selector and large Start button
- Minecraft news & patch notes (Java Edition) under News
- Launcher About / changelog in Settings
- Config editor with human-readable labels and Mod Menu–style sections
- ReqGuard dependency scanning, Modrinth browse/install, LittleSkin accounts
- Multi-language UI (English, 中文, Deutsch)

### Fixed
- Forge / NeoForge instances no longer mis-detected as Vanilla
- Trailing `-` on game versions breaking Modrinth search (`1.21.1-`)
- Config form showing raw debug/dev key names instead of readable labels
- Minecraft news failing to render when patch-notes fetch was slow or failed
