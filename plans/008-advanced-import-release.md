# Plan 008: Advanced settings, import, release

## Outcome

Per-instance JVM args/env/pre-post commands; import MultiMC/Prism-style instances; launcher README + `tauri build` packaging notes; polish settings.

## Depends on

- plans/006-reqguard.md
- plans/007-library-extras.md

## Context

- Prism/MultiMC: `instance.cfg` + `mmc-pack.json` + `.minecraft`
- Ship Windows installer/msi or nsis via Tauri bundler

## Steps

1. Persist and apply custom JVM args, env vars, wrapper pre/post commands on launch.
2. Import command: pick folder → detect Prism/MultiMC → copy into EUML instance layout.
3. Settings: instances path override, CF key, Java global default, telemetry off (none).
4. README: features, ReqGuard explanation, dev/build instructions.
5. Configure Tauri bundle identifiers; document `npm run tauri build`.
6. Final pass: empty states, motion polish, ReqGuard entry visible on Play.

## Verification

- [ ] Custom JVM args appear in launch command line
- [ ] Import from a Prism-like folder produces a listed instance
- [ ] README accurate; release build command documented
- [ ] Major plan verification checklist can be signed off

## Non-goals

- Auto-updater server infrastructure (stub settings OK)
