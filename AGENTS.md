# AGENTS.md

## Cursor Cloud specific instructions

Northstar (formerly EUML) is a **Tauri 2 desktop Minecraft launcher**. One product, two layers:

- Frontend: React 19 + TypeScript + Vite + Tailwind v4, using the `@astryxdesign/*` design system. Dev server runs on `http://localhost:1420` (`npm run dev`).
- Backend: Rust (`src-tauri/`, lib crate `euml_lib`). All native/launcher logic (accounts, downloads, launch, hosting, ReqGuard, UPnP) lives here and is exposed to the UI via Tauri commands.

Standard commands are already documented in `README.md` and `package.json` scripts; prefer those. Notes below are the non-obvious bits.

### Toolchain / build gotchas

- The Rust toolchain **must be stable ≥ 1.85** (dependency `zbus` needs the `edition2024` cargo feature). The update script runs `rustup default stable`; if `cargo` errors with "feature `edition2024` is required", the toolchain is too old — run `rustup update stable`.
- Linux system libraries for WebKitGTK are required and are NOT installed by the update script (they are apt packages, not project deps). They are pre-baked into the environment. If a fresh machine is missing them, install: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libgtk-3-dev libsoup-3.0-dev libssl-dev`. Note `libappindicator3-dev` and `libayatana-appindicator3-dev` conflict — install only the former (matches CI).

### Running the app

- Run the full desktop app with `npm run tauri:dev` (this auto-starts the Vite dev server; do NOT start `npm run dev` separately first). The window opens on the XFCE desktop at `DISPLAY=:1`.
- Set `export DISPLAY=:1` in the shell before launching, otherwise the window has nowhere to render.
- `libEGL warning: DRI3 error ...` on startup is expected on the software-rendered VM display and is harmless (WebKit falls back to software rendering).
- The desktop screensaver may blank the screen (black screen with a white cube) during idle periods; this is the XFCE screensaver, not an app crash. Run `xset -display :1 s off -dpms` to disable blanking, or just move the mouse to wake it.
- The first `tauri dev` build compiles the whole Rust dependency tree and can take a few minutes; subsequent runs are incremental and fast.

### Lint / test / build

- Frontend typecheck + production build: `npm run build` (`tsc -b && vite build`). There is no separate ESLint config in this repo; `tsc` is the type/lint gate.
- Rust: `cargo check`, `cargo clippy`, and `cargo test` run from `src-tauri/`. There is a small unit-test suite in `reqguard.rs`.
- Some backend metering (CPU/RAM/network in the Host page) is Windows-only; on Linux those meters intentionally report "Windows-only in this build".

### Manual testing

- A good end-to-end smoke test that exercises the Rust backend + persistence without external downloads is: open **Accounts** → type an offline username → **Add offline**. A new account with a generated UUID should appear and be marked "(Active)".
