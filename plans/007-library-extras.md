# Plan 007: Library extras

## Outcome

Manage resource packs and shader packs; browse worlds and screenshots; view instance logs with level filtering.

## Depends on

- plans/004-loaders-instance-ui.md

## Context

- Folders: `resourcepacks/`, `shaderpacks/`, `saves/`, `screenshots/`, `logs/`

## Steps

1. Commands to list/open/install zip into resourcepacks/shaderpacks; toggle enable if applicable.
2. List worlds (folder names + `level.dat` mtime if easy); open screenshots directory / show thumbnails in UI.
3. Log viewer: read `logs/latest.log` (or newest); filter by ERROR/WARN/INFO; auto-refresh optional.
4. UI tabs on instance detail: Content, Worlds, Screenshots, Logs.

## Verification

- [ ] Installing a resource pack zip lands in the right folder
- [ ] Logs view shows file content with filter
- [ ] Screenshots/worlds list reflects disk

## Non-goals

- In-game resource pack ordering protocol beyond folder presence
