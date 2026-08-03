# Plan 001: Scaffold + design system

## Outcome

Tauri 2 + React 19 + TypeScript + Vite + Tailwind v4 app boots on Windows. Design tokens (CSS variables), fonts (Syne + IBM Plex Sans), and app shell (nav + main) match locked UI direction.

## Depends on

- none

## Context

- Greenfield repo at workspace root
- Product name: EUML
- Avoid Inter, purple gradients, cream+terracotta, card-spam dashboards

## Steps

1. Scaffold Tauri 2 + Vite React-TS (`npm create tauri-app` or equivalent manual layout).
2. Add Tailwind CSS v4 with CSS variables: `--bg`, `--surface`, `--ink`, `--muted`, `--copper`, `--copper-dim`, `--danger`, `--warn`, `--ok`.
3. Load Google fonts (or self-host): Syne (display), IBM Plex Sans (body).
4. Build AppShell: left rail or top brand bar with EUML wordmark hero-weight on home; routes placeholder: Home, Library, Accounts, Settings.
5. Home first viewport: brand EUML, one headline, one supporting line, one primary CTA (Create instance)—no stats strips.
6. Add 2–3 CSS/motion cues (e.g. shell fade-in, copper underline on active nav, subtle bg grain/gradient).
7. README with run instructions (`npm install`, `npm run tauri dev`).

## Verification

- [ ] `npm run tauri dev` (or `npm run dev` + tauri) opens a window
- [ ] Tokens and fonts visible; no Inter/purple defaults
- [ ] Shell navigates between placeholder pages

## Non-goals

- Auth, instances, launch, mods
