# Plan 002: Instance domain + persistence

## Outcome

Rust-backed instance model with on-disk layout under app data; CRUD via Tauri commands; frontend list/detail wired to real data.

## Depends on

- plans/001-scaffold-design-system.md

## Context

- Each instance: isolated game dir (version, loader, mods, config, saves, resourcepacks, shaderpacks)
- Persist launcher settings (theme already CSS; paths, last account id later)

## Steps

1. Define Rust structs: `Instance` (id, name, game_version, loader, loader_version, java_path override, memory_mb, jvm_args, created_at, last_played).
2. Disk layout: `{appData}/euml/instances/{id}/instance.json` + `minecraft/` (`.minecraft`-style).
3. Commands: `list_instances`, `get_instance`, `create_instance`, `update_instance`, `delete_instance`, `open_instance_folder`.
4. Global settings JSON: `{appData}/euml/settings.json` (instances root override, curseforge_api_key placeholder).
5. Frontend Library page lists instances from backend; Create flow writes via command.

## Verification

- [ ] Create/list/update/delete instance survives app restart
- [ ] Instance folder exists on disk with `instance.json`
- [ ] Open folder command works on Windows

## Non-goals

- Downloading Minecraft, auth, mod platforms
