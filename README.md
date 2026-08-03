# EUML

Desktop Minecraft launcher with **ReqGuard** and a UI built on Meta’s open-source **[Astryx](https://github.com/facebook/astryx)** design system (`@astryxdesign/core` + `theme-neutral`).

## License

**All rights reserved.** See [LICENSE](LICENSE). This project is proprietary; viewing the source does not grant rights to use, copy, or redistribute it.

Third-party libraries keep their own licenses.

## UI

- Meta **Astryx**: AppShell, SideNav, Button, Card, Tabs, Selector, Banner, etc.
- Theme: `@astryxdesign/theme-neutral`
- Languages: English / 中文 / Deutsch (Settings)

## Stack

- Tauri 2 + Rust
- React 19 + TypeScript + Vite
- Tailwind CSS v4 + Astryx CSS layers

## Features

- Launch / Download / Versions / Host / Accounts / Settings
- Microsoft + offline accounts
- Parallel library & asset downloads
- Modrinth mods, `.mrpack` / Prism import
- ReqGuard pre-launch dependency scan
- Dedicated server hosting (UPnP / NAT-PMP / PCP)

## Develop

```bash
npm install
npm run tauri:dev
```

## Build (local)

```bash
npm run tauri:build
```

## Release builds (GitHub Actions)

Push to the `release` branch, or run **Actions → publish → Run workflow**.

CI builds Windows, macOS (Apple Silicon + Intel), and Linux and attaches installers to a **draft** GitHub Release.

1. Ensure the repo has **Settings → Actions → Workflow permissions → Read and write**.
2. Create/push branch `release` (or use workflow_dispatch).
3. Download artifacts from the draft release — no need to build on a local Mac.
