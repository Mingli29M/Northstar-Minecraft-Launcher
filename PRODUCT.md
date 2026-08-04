# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary: Minecraft Java Edition players who run modded or multi-instance setups and often also host for friends or LAN — including bilingual EN/中文/Deutsch users. They want a launcher that feels like a tool (PCL/HMCL-style launch flow), not a marketing dashboard.

## Product Purpose

Northstar is a desktop Minecraft launcher that combines launch, version/content management, accounts, and dedicated-server Host in one app. Success means a player can install, launch, fix missing mod deps before boot (ReqGuard), and host without juggling separate tools.

## Positioning

One desktop app for launch + Modrinth/content + dedicated Host, with ReqGuard pre-launch dependency checks — tool-like UX, proprietary (not a Prism/PCL/MultiMC fork), and not free-redistributable FOSS.

## Operating Context

- Desktop app shell: Tauri 2 (Windows, macOS, Linux) with a React/Astryx WebView UI.
- Local settings and caches under `%APPDATA%\euml\` (product name Northstar; folder name kept for upgrade stability).
- Marketing site: `website/` → GitHub Pages (`mingli29m.github.io/Northstar-Minecraft-Launcher/`).
- Releases via GitHub Actions (`release` branch / publish workflow).
- Resource comparisons documented in `BENCHMARKS.md` (same-OS idle / vanilla samples).

## Capabilities and Constraints

**Capabilities (confirmed):** Launch; Download/versions/loaders; Modrinth + `.mrpack` / Prism-MultiMC import; ReqGuard; Host (console, properties, UPnP→NAT-PMP→PCP); Microsoft / offline / LittleSkin accounts; appearance settings; locales en/zh/de.

**Constraints:**
- Product name: **Northstar** (user-facing). Internal crate/data id may remain `euml`.
- License: **All Rights Reserved** — viewing source ≠ permission to copy, modify, redistribute, or rebrand.
- Not affiliated with Mojang Studios or Microsoft; do not imply official Minecraft status.
- Do not fabricate testimonials, customers, or unmeasured performance claims; cite `BENCHMARKS.md` for memory/CPU figures.
- Independent of Prism, MultiMC, and PCL — comparison is allowed; affiliation or fork claims are not.

**Stack (existing):** Tauri 2 + Rust; React 19 + TypeScript + Vite; Tailwind CSS v4; Meta Astryx (`@astryxdesign/core` + `theme-neutral`). Marketing site shares Astryx components.

## Brand Commitments

- Name: Northstar.
- Voice: clear, tool-like, honest about tradeoffs (e.g. WebView vs Qt idle RAM).
- UI kit: Meta Astryx is the in-app and site component language (not a visual-world recipe — identity constraint only).

## Evidence on Hand

- App source: `src/`, `src-tauri/`.
- Marketing site: `website/`.
- License: `LICENSE`.
- Changelogs (split): launcher `CHANGELOG.md` + `src/lib/launcherChangelog.ts`; website `website/CHANGELOG.md` + `website/src/i18n/changelog.ts`.
- Benchmarks: `BENCHMARKS.md`.
- Public repo / releases: `Mingli29M/Northstar-Minecraft-Launcher`.

**Absent (do not invent):** paid testimonials, press quotes, official Mojang partnership, FOSS redistribution rights.

## Product Principles

1. **Tool over dashboard** — first screens prioritize launch and clear actions, not metric strips or chrome for its own sake.
2. **Launch and host together** — dedicated Host is first-class, not an afterthought.
3. **Fail before boot** — ReqGuard surfaces missing deps early.
4. **Honest ownership** — proprietary ARR; clear Mojang/Minecraft trademark distance; no fake social proof.
5. **Measured claims only** — performance and comparison numbers come from documented benchmarks or stay qualitative.

## Accessibility & Inclusion

No product-specific legal a11y mandate locked yet (undecided beyond sensible defaults). Locales en/zh/de are confirmed product requirements.
