# Plan 003: Auth, Java, vanilla launch

## Outcome

Microsoft account login (multi-account store), Java detect/adopt/download hooks, vanilla version manifest fetch, download client assets/libs, launch Minecraft process for a vanilla instance.

## Depends on

- plans/002-instance-domain.md

## Context

- MSA → Xbox → XSTS → Minecraft services profile chain
- Mojang/Microsoft version manifest + version JSON
- For v1: device-code or localhost redirect auth as practical in Tauri

## Steps

1. Account store: `{appData}/euml/accounts.json` (tokens encrypted-at-rest if feasible; else restricted file perms).
2. Commands: `list_accounts`, `begin_ms_login`, `complete_ms_login` / poll, `select_account`, `remove_account`, `refresh_account`.
3. Java: detect installed JDKs; store preferred; optional download Adoptium Temurin for required major version; command `resolve_java(instance_id)`.
4. Versions: fetch version manifest; list releases; download client jar, libraries, assets for selected version into shared `{appData}/euml/meta/` and instance dir as needed.
5. Launch: build classpath/args from version JSON + auth; spawn process; stream exit code; record `last_played`.
6. UI: Accounts page login flow; Play on instance (vanilla) gated on account + downloaded version.

## Verification

- [ ] User can add an MSA account (or mock/dev bypass documented if network-blocked in CI)
- [ ] Vanilla version downloads and launch command is constructed (full launch when credentials present)
- [ ] Java resolution returns a usable path

## Non-goals

- Mod loaders, ReqGuard, mod browsers
