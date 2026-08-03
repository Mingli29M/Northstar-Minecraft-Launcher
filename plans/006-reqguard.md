# Plan 006: ReqGuard

## Outcome

Parse mod jar metadata, build dependency graph, show pre-launch Notice UI, gate Play on hard failures (with explicit override), and offer one-click install of missing deps via Modrinth (then CurseForge).

## Depends on

- plans/005-mod-platforms-packs.md

## Context

- Fabric/Quilt: `fabric.mod.json` / `quilt.mod.json` — depends, recommends, suggests, breaks, conflicts
- Forge/NeoForge: `META-INF/mods.toml` / `neoforge.mods.toml` — required, optional, incompatible, discouraged
- Also validate minecraft + loader version constraints against instance

## Steps

1. Rust: open each `.jar` as zip; extract metadata; normalize to `ModConstraint` model.
2. Version-range matcher (semver-ish / Maven ranges as practical for MC mods).
3. Command `reqguard_scan(instance_id)` → issues list with severity: error | warn | info.
4. Invalidate/re-scan on mods folder changes and before launch.
5. UI Notice panel on instance: list missing/break/conflict; actions Install / Ignore warn / Override launch.
6. Play button: run scan; block on errors unless override flag for this session.
7. Resolve: map mod id → Modrinth project/version compatible with instance MC+loader; install; re-scan.

## Verification

- [ ] Instance missing a required dep shows error in Notice before launch
- [ ] Install missing from Notice adds jar and clears error when available
- [ ] Breaks/conflicts surface correctly from sample metadata
- [ ] Override still launches after explicit confirm

## Non-goals

- Fixing runtime crashes unrelated to declared metadata
