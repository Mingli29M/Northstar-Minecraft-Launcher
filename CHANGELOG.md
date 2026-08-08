# Northstar Launcher Changelog

Launcher release notes for the desktop app (Tauri). Website marketing changes live in [`website/CHANGELOG.md`](website/CHANGELOG.md). Both are shown side by side on the site Changelog page.

> **Split (1.1.1):** Launcher and website changelogs are maintained as separate files. This file covers the desktop launcher only.

## Unreleased

### macOS
- **Microphone (and camera) permission** — `Info.plist` usage descriptions plus hardened-runtime audio-input/camera entitlements so mods like Simple Voice Chat can prompt for the mic when launched from Northstar
- **Terracotta LAN detection** — Install now runs the official Terracotta `.pkg` (admin password prompt) to register `/Applications/terracotta.app` + its LaunchAgent, matching HMCL; without that helper, host-scanning never sees Open-to-LAN worlds
- Local Network usage description updated so Minecraft/Terracotta LAN traffic can prompt under Privacy & Security

## 1.3.0 — 2026-08-07

Translucent window over the desktop, reliable Launch layout controls, richer Modrinth install/detail UX, and a real Litematica tab.

### Appearance
- **See-through window** — panel opacity now controls real OS window translucency (Acrylic/Blur) so your desktop wallpaper can show through the launcher
- Opacity applies to the window wash only; cards, lists, selectors and menus stay solid so text stays readable
- Range widened to 20–100%

### Launch
- **Fix Start button position not applying** — Settings → Appearance uses clear Top / Bottom controls, works in compact and full layouts, and no longer gets overwritten by keep-alive / settings races
- Compact mode no longer locks or forces Start position

### Mods
- Modrinth install opens a version picker with prerequisites (required / optional / incompatible), nested dependency install, and an “Installed” badge
- Mod detail page renders Markdown descriptions, a gallery with lightbox, and a fuller versions list
- Mod installs report download progress in the dock (and stay visible above the install modal)

### Versions
- New **Litematica** instance tab to import, export, delete, and open schematics (`.litematic` / `.schematic` / `.schem`)
- Worlds and schematics no longer share list state (fixes worlds briefly showing as Litematica items)

### Worlds
- Chunkbase seed map opens in the system browser (in-app iframe / WebviewWindow blocked by Chunkbase headers and Windows deadlocks)

## 1.2.4 — 2026-08-06

Host panel reliability plus Terracotta multiplayer as an AGPL sidecar (Northstar license unchanged).

### Host
- Hide helper/server consoles on Windows (`CREATE_NO_WINDOW`) so Start no longer pops many terminal windows
- Opening Host no longer waits on public-IP / UPnP gateway discovery; Network tab loads that in the background
- Status polling continues even when the UI briefly thinks the server is Stopped
- Port-based + improved orphan recovery so a live server is not marked Stopped after launcher reload
- Standout Running / Stopped badge (header + sticky bar) with PID when available

### Terracotta
- New **Terracotta** nav tab downloads and runs the official unmodified `0.4.2` package as a separate process
- Talks only over Terracotta’s local HTTP IPC (`--hmcl`); does **not** link Terracotta source into Northstar
- Host / join room flow with required AGPL attribution in the tab and Settings → About
- Terracotta remains AGPL-3.0-or-later; Northstar stays proprietary (All Rights Reserved)
- **Fix "Access is denied (os error 5)" on Reinstall** — the sidecar no longer uses its own install folder as its working directory, and the port handoff file moved to a temp dir, so the folder is never locked while being replaced
- Reinstall now stops sidecars started from our install folder first (scoped by executable path, so another launcher's Terracotta is untouched), stages the extract in a sibling folder, and swaps it in
- Package downloads are verified against the official SHA-512 and retried across four mirrors, so a truncated archive is replaced instead of being cached as "good" forever
- **Fix Terracotta being reported as errored** — a missed `/state` poll is retried (as HMCL does) and only counts as an error once the process is actually gone; startup shows "Starting…"
- Install state is checksum-verified, so a half-extracted executable reads as "not installed" instead of failing at launch
- Room/connection states now show readable labels ("Hosting", "Connecting to room…") instead of raw `host-ok` style names, and Terracotta's own connection errors are explained
- **Fix Start never sticking** — on Windows `--hmcl` is only a trampoline that re-spawns the binary detached and exits within 8 seconds, so the launcher watched the wrong process and decided the sidecar had died seconds after a successful start. Northstar now starts the real server directly and owns it
- **Fix orphaned sidecars** — because Stop was killing that already-exited trampoline, the real server survived every Stop and quit, holding Terracotta's machine-wide lock and a file lock on the executable. That single leftover process was also the underlying cause of repeat "os error 5" reinstall failures
- Stop now asks the sidecar to shut down cleanly (releasing the lock) before falling back to terminating it
- Start attaches to a sidecar that is already up instead of failing, and Northstar re-attaches on launch to one left behind by a crash — scoped to servers it started, so a Terracotta owned by HMCL or PCL is never adopted, stopped, or counted by the exit prompt
- A failed start now prints the tail of Terracotta's own log to the launcher console; it is a windowless process that redirects its output to a file, so nothing was previously visible to explain a failure

### Downloads
- Large single-file downloads (Terracotta, Java runtimes, server jars, installers, plugins) now report progress — previously they ran completely silently with no dock entry or console line
- Progress dock shows transferred size and speed (`12.3 MB / 45.6 MB`, `3.1 MB/s`) for single-file transfers, keeping file counts for multi-file batches

### Reliability
- Closing Northstar while a dedicated server or Terracotta is running now asks first, naming what would be shut down, instead of silently orphaning the process
- **Stop and quit** shuts everything down cleanly; **Keep running** cancels the close

### Appearance
- **Fix Appearance settings doing nothing** — Astryx parks accent/card tokens in `@layer astryx-theme` + `@scope`, so our old inline CSS variables never won the cascade. Appearance now injects an unlayered stylesheet that actually overrides accent, fonts, panel opacity and background, applies live, auto-saves, and pushes Launch layout flags (compact / Start position) into the keep-alive Launch page immediately

### Launch
- **Fix a newly downloaded or imported instance not appearing in the start menu** — the Launch page stays mounted in the background for speed and was only reading the instance list once, so it kept showing a stale list until the app was restarted. It now refreshes whenever you return to it, keeping your current selection
- **Compact Launch page** and **Start button position** live in Settings → Appearance (not as extra Launch toggles). Compact mode strips Launch to the version picker, Start and ReqGuard override, and docks Start at the bottom; the position setting applies on the full page
- **Fix downloads that treated pack names as Minecraft versions** — strings like `Create：Complete` are no longer accepted as `game_version`. Prepare now tells you to set a real version in Version settings instead of looking for a Mojang id that does not exist
- **Local metadata scan off means no jar unzipping** — with the experimental local scan disabled, ReqGuard no longer opens every mod jar; with both local and deep off, the Launch panel skips scanning entirely

### Localization
- Removed Chinese text that appeared in the English and German UI (Terracotta notes, multiplayer hint)
- The offline-account "username cannot be empty" error is no longer hard-coded in Chinese
- The Servers page Terracotta button now opens the in-app tab instead of an external download page

## 1.2.3 — 2026-08-06

Security hardening for local file access and avatar downloads.

### Security
- **Asset protocol scope tightened** — `asset://` no longer allows `$HOME/**` or a catch-all `"**"`; only app wallpaper and avatar cache directories are readable
- Background images from Browse / drag-and-drop are copied into the app wallpapers folder via `import_background_image` (keeps Appearance working inside the allow-list)
- Settings free-text background field rejects absolute filesystem paths (remote / `data:` URLs still allowed)
- **Avatar fetch SSRF controls** — HTTPS-only host allowlist, no HTTP redirects, hex UUID validation, and a 2 MiB response size cap so profile-supplied skin URLs cannot reach internal hosts or exhaust memory

## 1.2.2 — 2026-08-05

ReqGuard install fixes and experimental local-scan toggle.

### ReqGuard
- Fix Install / Install-all buttons: surface install errors, pass Modrinth `project_id`, and re-scan with the same Settings modes after install
- Install-all now uses the configured (deep/local) scan instead of a local-only pass that ignored Modrinth SoT issues
- **Local metadata scan** is opt-in and labeled **Experimental** (off by default); Play is only gated when it is enabled
- Deep Modrinth validation remains the recommended path and stays in the background worker

## 1.2.1 — 2026-08-05

Hangar plugins for Paper/Purpur Host and ReqGuard polish.

### Host plugins
- **Plugins tab** on Paper/Purpur dedicated servers: search [Hangar](https://hangar.papermc.io), install jars into `runtime/plugins/`
- Installed list with enable/disable (`.jar` / `.jar.disabled`) and delete

### ReqGuard
- `reqguard_resolve_all` exposed on Launch and Versions (install all missing)
- Modrinth dependency lookup User-Agent bumped to `Northstar/1.2.1`

## 1.2.0 — 2026-08-05

ReqGuard Modrinth source-of-truth, crash analysis, Java Temurin download, world backups, and Litematica notice.

### ReqGuard
- ReqGuard scans run in a background worker; optional deep validation checks the actual instance jars against Modrinth
- Canonical mod-id aliases, multi-mod jar ownership filtering, and self-dependency rejection fix false Fabric API and “mod requires itself” reports
- Local launch gate stays offline and fast; exotic version ranges warn instead of hard-error
- Modrinth dependency SoT uses batched SHA1 lookup across every jar, then reconciles project slugs with local mod ids so manual/CurseForge installs are recognized
- Resolve installs by Modrinth project id (with required dependency chain); “Install all missing”

### Crash analysis
- Prefer newest crash-report + `latest.log`; extract exception/frames; stable hint codes for UI
- Monitor Minecraft for its full lifetime, analyze abnormal exits automatically, and persist an in-app message for the next launcher session

### Prerequisites
- Java status panel on Download → Game; one-click Adoptium Temurin download into launcher-managed Java dir
- Modrinth installs auto-pull required dependency chain

### Worlds
- Per-world backup fold-down (create / restore / delete) under `saves/<world>/backups/`
- Settings → Backups `auto_backup_worlds` runs on launch with keep-last-N pruning
- Litematica detection notice with schematics folder path

## 1.1.2 — 2026-08-05

Settings sections, license/changelog in About, panel opacity, loader icons, and shared target-version selection.

### Settings
- Settings organized into General, Appearance, Java, Backups, and About sections
- About includes All Rights Reserved license summary plus the full launcher changelog
- Backups section exposes auto-backup toggles (snapshots take effect in 1.2.0+)

### Bug fixes
- **Appearance settings apply correctly** — accent, wallpaper, font, UI scale, and panel opacity now target Astryx theme scopes (`[data-astryx-theme]`), so cards and text actually update
- Download / Servers “Target version” defaults to the instance selected on Launch (and stays in sync when changed)
- **Player head icons load reliably** — fetched/cached in the Rust backend (Crafthead / MC-Heads / Mojang+BMCLAPI skin crop) instead of loading Crafatar directly in the WebView

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
- Replaced nether-star app icons with the new Northstar mark (window / installer icons)
- Overlay shard mark used for taller brand visuals

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
- Home screen with large Start button and quick version picker
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
- Config editor with human-readable labels and grouped sections

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
- Launch screen with version selector and large Start button
- Minecraft news & patch notes (Java Edition) under News
- Launcher About / changelog in Settings
- Config editor with human-readable labels and grouped sections
- ReqGuard dependency scanning, Modrinth browse/install, LittleSkin accounts
- Multi-language UI (English, 中文, Deutsch)

### Fixed
- Forge / NeoForge instances no longer mis-detected as Vanilla
- Trailing `-` on game versions breaking Modrinth search (`1.21.1-`)
- Config form showing raw debug/dev key names instead of readable labels
- Minecraft news failing to render when patch-notes fetch was slow or failed
