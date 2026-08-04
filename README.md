# Northstar

Desktop Minecraft launcher with **ReqGuard** and a UI built on Meta’s open-source **[Astryx](https://github.com/facebook/astryx)** design system (`@astryxdesign/core` + `theme-neutral`).

**Site:** [https://mingli29m.github.io/Northstar-Minecraft-Launcher/](https://mingli29m.github.io/Northstar-Minecraft-Launcher/)

## License

**All rights reserved.** See [LICENSE](LICENSE). This project is proprietary; viewing the source does not grant rights to use, copy, or redistribute it.

Third-party libraries keep their own licenses.

## UI

- Meta **Astryx**: AppShell, SideNav, Button, Card, Tabs, Selector, Banner, etc.
- Theme: `@astryxdesign/theme-neutral`
- Languages: English / 中文 / Deutsch (Settings)
- Appearance: accent color, background color/image, font family, and UI scale (Settings → Appearance)

## Stack

- Tauri 2 + Rust
- React 19 + TypeScript + Vite
- Tailwind CSS v4 + Astryx CSS layers

## Features

- Launch / Download / Versions / Host / Accounts / Settings
- Microsoft + offline + LittleSkin accounts
- Parallel library & asset downloads
- Modrinth mods, `.mrpack` / Prism import
- ReqGuard pre-launch dependency scan
- Dedicated server hosting (UPnP / NAT-PMP / PCP)
- Appearance customization (persisted in `%APPDATA%\euml\settings.json`)

## Resource usage

Same-OS idle and vanilla **1.21.11** samples (Working Set ≈ RSS, Private Bytes ≈ USS) are documented in [BENCHMARKS.md](BENCHMARKS.md).

## Develop

```bash
npm install
npm run tauri:dev
```

## Build (local)

```bash
npm run tauri:build
```

## Marketing site (GitHub Pages)

Source lives in [`website/`](website/). Normal project landing page (hero, why, features, compare, download) styled with **Meta Astryx** components — not a copy of the launcher AppShell.

Pages: `/` (landing) · `/about` · `/license`.

Idle Working Set / private-bytes samples vs Prism / MultiMC / HMCL / PCL are in [BENCHMARKS.md](BENCHMARKS.md) and on the landing Compare section + About page.

```bash
cd website
npm install
npm run dev
```

Site URL: https://mingli29m.github.io/Northstar-Minecraft-Launcher/

Enable **Settings → Pages → Source: GitHub Actions** once if needed.

## Release builds (GitHub Actions)

Push to the `release` branch, or run **Actions → publish → Run workflow**.

CI builds Windows, macOS (Apple Silicon + Intel), and Linux and attaches installers to a **draft** GitHub Release.

1. Ensure the repo has **Settings → Actions → Workflow permissions → Read and write**.
2. Create/push branch `release` (or use workflow_dispatch).
3. Download artifacts from the draft release — no need to build on a local Mac.
