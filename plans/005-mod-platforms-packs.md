# Plan 005: Mod platforms + packs

## Outcome

Browse/search/install mods from Modrinth; CurseForge when API key set; enable/disable mods; import/export `.mrpack`.

## Depends on

- plans/004-loaders-instance-ui.md

## Context

- Modrinth API v2; CurseForge Core API with user-supplied key in settings
- Mods live in `minecraft/mods/`; disabled via `.disabled` suffix or sidecar

## Steps

1. Rust HTTP clients for Modrinth search + version files download into instance mods folder.
2. CurseForge client gated on `settings.curseforge_api_key`.
3. Commands: `search_mods`, `install_mod`, `list_instance_mods`, `set_mod_enabled`, `uninstall_mod`.
4. `.mrpack` import: parse index, download files, create/update instance; export basic mrpack from instance.
5. UI: Mods tab on instance detail; Settings field for CF key; Import pack entry on Library.

## Verification

- [ ] Search + install a Modrinth mod into an instance folder
- [ ] Enable/disable reflected on disk
- [ ] Import `.mrpack` creates a playable instance skeleton (files present)

## Non-goals

- Full ReqGuard graph (next plan); shader/resource pack browsers (007)
