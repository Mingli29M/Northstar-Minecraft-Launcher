# Major plan: EUML full launcher + ReqGuard

## Goal

Ship **EUML**, a desktop Minecraft launcher covering Prism/Modrinth/CurseForge/ATLauncher-class features, plus **ReqGuard**: pre-read mod jar dependency metadata and notify users before launch with actionable fixes. Modern UI: obsidian/slate + copper, Syne + IBM Plex Sans—not generic AI chrome.

## Sub-plans (execution order)

| # | File | Outcome | Depends on | Status |
|---|------|---------|------------|--------|
| 1 | plans/001-scaffold-design-system.md | Tauri/React boots; design tokens; app shell | — | done |
| 2 | plans/002-instance-domain.md | Instance model, disk layout, settings | 001 | done |
| 3 | plans/003-auth-java-vanilla-launch.md | MSA auth, Java, vanilla launch | 002 | done |
| 4 | plans/004-loaders-instance-ui.md | Loaders + create/manage instances UI | 003 | done |
| 5 | plans/005-mod-platforms-packs.md | Modrinth/CF + pack import/export | 004 | done |
| 6 | plans/006-reqguard.md | Metadata parsers, Notice UI, launch gate | 005 | done |
| 7 | plans/007-library-extras.md | Packs/shaders, worlds, screenshots, logs | 004 | done |
| 8 | plans/008-advanced-import-release.md | JVM controls, imports, packaging | 006, 007 | done |

## Finish-all protocol

1. Execute plans in table order; do not skip unmet depends-on.
2. After each sub-plan: run verification; set Status to `done`.
3. If blocked: note blocker; do not mark done.
4. When all rows are `done`, run major verification and stop.

## Locked decisions

- Stack: Tauri 2 + Rust + React 19 + TypeScript + Vite + Tailwind CSS v4
- Platforms: Modrinth primary; CurseForge optional user API key
- Loaders: Vanilla, Fabric, Quilt, Forge, NeoForge
- UI: deep slate/obsidian, copper accent, Syne + IBM Plex Sans

## Major verification

- [x] Scaffold + design system boot (`npm run build` / cargo check)
- [x] Instance CRUD + disk layout
- [x] MSA device-code auth + Java detect + vanilla prepare/launch pipeline
- [x] Loader install (Fabric/Quilt profile; Forge/NeoForge staged) + Library UI
- [x] Modrinth install + mrpack import/export; CF search when key set
- [x] ReqGuard scan / launch gate / resolve missing
- [x] Content, worlds, screenshots, logs
- [x] Advanced JVM/env/hooks + Prism import + README/build docs
- [ ] End-to-end MSA launch on user machine (requires owned game + network)

## Out of scope

- FPS client injectors, server hosting panel, shipping CF API key, mobile/web, skin editors
