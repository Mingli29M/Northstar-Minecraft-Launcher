# Launcher resource usage benchmarks

Measured on **2026-08-04** on the same Linux host for every row below.

## Environment

| Item | Value |
| --- | --- |
| OS | Ubuntu 24.04.4 LTS (`x86_64`), kernel `6.12.94+` |
| Display | XFCE on `DISPLAY=:1`, Mesa llvmpipe (software GL) |
| RAM | 15 GiB |
| CPU | 4× Intel Xeon (VM) |
| Java (system) | OpenJDK 21.0.10 |
| Northstar build | `npm run tauri:build` → `src-tauri/target/release/euml` (v1.1.0) |

## Builds under test

| Launcher | Build |
| --- | --- |
| Northstar | Release binary from this repo (`tauri build`) |
| Prism Launcher | 11.0.3 Linux Qt6 portable |
| MultiMC | `mmc-stable-lin64.tar.gz` from files.multimc.org (dated 2023-02-04) |
| HMCL | 3.16.3 `.jar` (JavaFX) |
| PCL CE | 2.15.0 Windows `PCL2_CE_Release_x64.exe` (no native Linux build) |

## Methodology

Windows Task Manager columns are mapped to Linux as follows:

| Windows | Linux analogue used here |
| --- | --- |
| **Working Set** | Sum of `VmRSS` over the launcher process tree (`/proc/<pid>/status`) |
| **Private Bytes** | Sum of `Private_Dirty + Private_Clean` from `/proc/<pid>/smaps_rollup` (USS-like) |
| **CPU** | Optional: `(Δutime+Δstime) / CLK_TCK / Δt × 100` over three 1-second samples |

Procedure for **idle UI**:

1. Cold-start the launcher main window alone (no Minecraft client).
2. Idle **30 seconds** with no user input.
3. Take **3 samples** at 1 s intervals; report the average.
4. Measure the **full process tree** (important for Tauri/WebKitGTK, which uses multiple processes).
5. Quit the launcher before starting the next one.

Procedure for **vanilla 1.21.11** (where launch succeeded):

1. Prepare/download vanilla **1.21.11** in that launcher.
2. Launch offline (`BenchPlayer`), wait until the Minecraft window is up.
3. Idle **30 seconds** on the title/welcome UI (no world join).
4. Sample launcher tree and the Minecraft Java process separately, then report totals.

Notes / limits:

- Software GL (llvmpipe) inflates **game CPU** vs a GPU host; memory figures are still useful for relative comparison.
- **PCL CE** did not run natively here (Windows/.NET 10). Wine reported a missing .NET desktop runtime; no fair same-OS sample.
- **MultiMC** stable (2023) could not complete a 1.21.11 offline download/launch in this environment (account/asset gate + outdated client).
- **HMCL** idle UI was measured; offline account wiring blocked an automated 1.21.11 launch in this run (version was installed to `~/.minecraft`).

## Idle UI results (current)

| Launcher | Working Set ≈ RSS (MiB) | Private Bytes ≈ USS (MiB) | CPU % (idle) | Processes |
| --- | ---: | ---: | ---: | ---: |
| Prism Launcher 11.0.3 | 64.1 | 58.0 | 0.0 | 1 |
| MultiMC (stable lin64) | 110.9 | 50.2 | 0.0 | 1 |
| HMCL 3.16.3 | 296.5 | 217.1 | 0.3 | 1 |
| **Northstar 1.1.0 (optimized)** | **337.0** | **99.8** | 0.3 | 3 |
| PCL CE 2.15.0 | — | — | — | N/A on Linux (Windows-only) |

### Northstar process breakdown (optimized idle)

| Process | RSS (MiB) | Private (MiB) |
| --- | ---: | ---: |
| `euml` (main) | 141.6 | 38.0 |
| `WebKitWebProcess` | 141.6 | 50.8 |
| `WebKitNetworkProcess` | 53.8 | 11.1 |
| **Total** | **337.0** | **99.8** |

Pre-optimization baseline on the same machine: **~660 MiB RSS / ~377 MiB private** (WebProcess alone ~430 MiB RSS).

## Vanilla 1.21.11 (launcher + game)

Measured against the **pre-optimization** Northstar build (game totals are dominated by the JVM either way):

| Launcher | Launcher RSS / Private (MiB) | Game RSS / Private (MiB) | Game CPU % | Total RSS / Private (MiB) |
| --- | ---: | ---: | ---: | ---: |
| Prism Launcher | 454.0 / 445.9 | 1099.4 / 1035.9 | ~220 | 1553.4 / 1481.8 |
| Northstar (pre-opt) | 475.8 / 197.1 | 1194.5 / 1118.9 | ~216 | 1670.3 / 1315.9 |
| MultiMC | — | — | — | Not launched (see notes) |
| HMCL | — | — | — | Not launched (see notes) |
| PCL CE | — | — | — | N/A on Linux |

Game CPU well above 100% is expected under llvmpipe (multi-threaded software rasterization) and should not be compared to GPU systems.

## Optimization work

Idle ~660 MiB was almost entirely the **WebKitGTK process model**, not the 16 MiB Rust binary. Prism-class (~64 MiB) needs a native toolkit; these changes cut the WebView tax:

| Change | Effect |
| --- | --- |
| Default `WEBKIT_DISABLE_COMPOSITING_MODE=1` + `WEBKIT_DISABLE_DMABUF_RENDERER=1` on Linux (`lib.rs`) | **Largest win** — WebProcess ~430 → ~142 MiB RSS |
| Disable WebGL / WebAudio / page cache via `with_webview` | Small additional trim |
| Lazy routes + Vite manual chunks; LRU keep-alive (max 3) | Keeps Host/Versions out of cold-start JIT; helps after navigation |
| Defer Launch news/patch-notes fetch | Quieter NetworkProcess on first paint |
| Drop unused `tauri-plugin-fs` / `shell`; stop 1.5s console polling | Small main-process / IPC savings |

**Result:** ~660 → **337 MiB** Working Set (−49%), ~377 → **100 MiB** private (−74%). Private bytes now undercut HMCL; Working Set still trails Qt.

### Further options (not in this PR)

- Smaller default window (less software-GL backing store on this VM).
- More aggressive news/image deferral or “load news on demand”.
- Longer-term: non-WebView UI shell if the goal is Prism-class idle RAM.

## Reading the numbers

- Qt launchers (Prism / MultiMC) stay light at idle.
- HMCL pays a Java/JavaFX baseline (high private bytes).
- Northstar’s remaining idle cost is shared WebKit/GTK mappings (RSS) with a much smaller unique set (private).
- With vanilla 1.21.11 running, **the game dominates** memory for every launcher.
