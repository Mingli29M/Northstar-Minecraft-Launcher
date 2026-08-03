# Plan 004: Loaders + instance UI

## Outcome

Install Fabric / Quilt / Forge / NeoForge into an instance; polished create/manage instance UI (gallery + detail) matching design system.

## Depends on

- plans/003-auth-java-vanilla-launch.md

## Context

- Fabric Meta API / Quilt Meta / Forge maven / NeoForge maven installer profiles
- UI: instance-first, no card spam—list/rows or full-bleed hero per selected instance

## Steps

1. Loader install modules in Rust: resolve loader version for MC version; download installer or profile JSON; merge into instance version profile.
2. Extend `create_instance` / update to set loader + loader_version.
3. Instance Library UI: select instance → detail with Play, folder, edit name/version/loader/memory.
4. Create Instance wizard: name → MC version → loader → confirm.
5. Wire Play to launch pipeline using loader profile when present.

## Verification

- [ ] Create Fabric (or NeoForge) instance records loader fields and installs profile files
- [ ] UI create/edit/delete flows work without generic dashboard clutter
- [ ] Play uses loader-aware version JSON when set

## Non-goals

- Modrinth/CF browse, ReqGuard
