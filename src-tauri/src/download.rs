use crate::paths::load_settings;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::io::{copy, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DownloadSource {
    Official,
    BmclApi,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub phase: String,
    pub done: usize,
    pub total: usize,
    pub failed: usize,
    pub current_file: Option<String>,
    pub bytes_per_sec: Option<f64>,
    pub message: String,
    pub active: bool,
}

static PART_SEQ: AtomicU64 = AtomicU64::new(1);

pub fn emit_progress(app: Option<&AppHandle>, progress: DownloadProgress) {
    let level = if progress.failed > 0 {
        "warn"
    } else if progress.phase == "done" || !progress.active {
        "info"
    } else {
        "progress"
    };
    if !progress.message.is_empty() {
        crate::console_log::append(app, progress.message.clone(), level);
    }
    if let Some(ref name) = progress.current_file {
        if progress.active {
            crate::console_log::append(app, format!("{}: {name}", progress.phase), "progress");
        }
    }
    if let Some(app) = app {
        let _ = app.emit("euml:download-progress", &progress);
    }
}

pub fn download_source() -> DownloadSource {
    match load_settings()
        .ok()
        .and_then(|s| s.download_source)
        .as_deref()
    {
        Some("bmclapi") | Some("BMCLAPI") => DownloadSource::BmclApi,
        _ => DownloadSource::Official,
    }
}

pub fn download_threads() -> usize {
    load_settings()
        .ok()
        .and_then(|s| s.download_threads)
        .unwrap_or(16)
        .clamp(4, 64) as usize
}

/// Rewrite Mojang / Fabric / Quilt / Forge CDN URLs when BMCLAPI is selected.
pub fn rewrite_url(url: &str) -> String {
    if download_source() != DownloadSource::BmclApi {
        return url.to_string();
    }
    let replacements = [
        (
            "https://piston-meta.mojang.com",
            "https://bmclapi2.bangbang93.com",
        ),
        (
            "https://launchermeta.mojang.com",
            "https://bmclapi2.bangbang93.com",
        ),
        (
            "https://launcher.mojang.com",
            "https://bmclapi2.bangbang93.com",
        ),
        (
            "https://piston-data.mojang.com",
            "https://bmclapi2.bangbang93.com",
        ),
        (
            "https://libraries.minecraft.net",
            "https://bmclapi2.bangbang93.com/maven",
        ),
        (
            "https://resources.download.minecraft.net",
            "https://bmclapi2.bangbang93.com/assets",
        ),
        (
            "https://meta.fabricmc.net",
            "https://bmclapi2.bangbang93.com/fabric-meta",
        ),
        (
            "https://maven.fabricmc.net",
            "https://bmclapi2.bangbang93.com/maven",
        ),
        (
            "https://meta.quiltmc.org",
            "https://bmclapi2.bangbang93.com/quilt-meta",
        ),
        (
            "https://maven.quiltmc.org/repository/release",
            "https://bmclapi2.bangbang93.com/maven",
        ),
        (
            "https://maven.minecraftforge.net",
            "https://bmclapi2.bangbang93.com/maven",
        ),
        (
            "https://files.minecraftforge.net/maven",
            "https://bmclapi2.bangbang93.com/maven",
        ),
        (
            "https://files.minecraftforge.net",
            "https://bmclapi2.bangbang93.com",
        ),
        (
            "https://maven.neoforged.net/releases",
            "https://bmclapi2.bangbang93.com/maven",
        ),
        (
            "https://maven.neoforged.net",
            "https://bmclapi2.bangbang93.com/maven",
        ),
    ];
    // Longer prefixes first — sort by from.len descending would be ideal; listed longest-first above for forge.
    let mut out = url.to_string();
    for (from, to) in replacements {
        if out.starts_with(from) {
            out = format!("{to}{}", &out[from.len()..]);
            break;
        }
    }
    out
}

fn shared_client() -> Result<&'static reqwest::blocking::Client, String> {
    static CLIENT: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();
    match CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(20))
            .pool_max_idle_per_host(8)
            .user_agent("EUML/0.1.0")
            .build()
            .map_err(|e| e.to_string())
    }) {
        Ok(c) => Ok(c),
        Err(e) => Err(e.clone()),
    }
}

fn file_ok(path: &Path) -> bool {
    fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false)
}

/// Stream a single file to disk (no full-body RAM buffer). Thread-safe temp names.
pub fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    if file_ok(dest) {
        return Ok(());
    }
    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let url = rewrite_url(url);
    let client = shared_client()?;
    let mut resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("{url}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("{url}: {e}"))?;

    let seq = PART_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = dest.with_extension(format!("part.{seq}"));
    {
        let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
        copy(&mut resp, &mut f).map_err(|e| format!("write {url}: {e}"))?;
        f.flush().map_err(|e| e.to_string())?;
    }
    if !file_ok(&tmp) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("empty download: {url}"));
    }
    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    fs::rename(&tmp, dest).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        e.to_string()
    })?;
    Ok(())
}

/// Parallel download of many (url, dest) pairs. Runs on a dedicated Rayon pool.
/// `base_done` / `display_total` adjust progress reporting when some files were
/// already present and skipped before enqueueing (e.g. assets).
pub fn download_many(
    jobs: Vec<(String, PathBuf)>,
    app: Option<&AppHandle>,
    phase: &str,
) -> Result<(usize, usize), String> {
    download_many_progress(jobs, app, phase, 0, None)
}

pub fn download_many_progress(
    jobs: Vec<(String, PathBuf)>,
    app: Option<&AppHandle>,
    phase: &str,
    base_done: usize,
    display_total: Option<usize>,
) -> Result<(usize, usize), String> {
    let job_count = jobs.len();
    let total = display_total.unwrap_or(base_done + job_count);
    if job_count == 0 {
        emit_progress(
            app,
            DownloadProgress {
                phase: phase.to_string(),
                done: base_done,
                total,
                failed: 0,
                current_file: None,
                bytes_per_sec: None,
                message: format!("{phase}: {base_done}/{total}"),
                active: true,
            },
        );
        return Ok((0, 0));
    }
    let threads = download_threads();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|e| e.to_string())?;

    let ok = AtomicUsize::new(0);
    let fail = AtomicUsize::new(0);
    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let last_emit = Mutex::new(Instant::now());
    let started = Instant::now();
    let app = app.cloned();
    let phase_owned = phase.to_string();

    emit_progress(
        app.as_ref(),
        DownloadProgress {
            phase: phase_owned.clone(),
            done: base_done,
            total,
            failed: 0,
            current_file: None,
            bytes_per_sec: None,
            message: format!("Starting {phase} ({job_count} files, {threads} threads)…"),
            active: true,
        },
    );

    pool.install(|| {
        jobs.par_iter().for_each(|(url, dest)| {
            let name = dest
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| url.clone());
            let result = if file_ok(dest) {
                Ok(())
            } else {
                download_file(url, dest)
            };
            match result {
                Ok(()) => {
                    ok.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    fail.fetch_add(1, Ordering::Relaxed);
                    if let Ok(mut lock) = errors.lock() {
                        if lock.len() < 8 {
                            lock.push(e);
                        }
                    }
                }
            }
            let batch_done = ok.load(Ordering::Relaxed) + fail.load(Ordering::Relaxed);
            let failed = fail.load(Ordering::Relaxed);
            let done = base_done + batch_done;
            let should_emit = {
                if let Ok(mut last) = last_emit.lock() {
                    if last.elapsed().as_millis() >= 120 || batch_done == job_count {
                        *last = Instant::now();
                        true
                    } else {
                        false
                    }
                } else {
                    batch_done == job_count
                }
            };
            if should_emit {
                let elapsed = started.elapsed().as_secs_f64().max(0.001);
                let speed = batch_done as f64 / elapsed;
                emit_progress(
                    app.as_ref(),
                    DownloadProgress {
                        phase: phase_owned.clone(),
                        done,
                        total,
                        failed,
                        current_file: Some(name),
                        bytes_per_sec: Some(speed),
                        message: format!("{phase_owned}: {done}/{total}"),
                        active: true,
                    },
                );
            }
        });
    });

    let failed = fail.load(Ordering::Relaxed);
    let succeeded = ok.load(Ordering::Relaxed);
    if failed > 0 {
        let sample = errors
            .lock()
            .map(|e| e.join("; "))
            .unwrap_or_default();
        eprintln!("download failures ({failed}): {sample}");
    }
    emit_progress(
        app.as_ref(),
        DownloadProgress {
            phase: phase_owned.clone(),
            done: base_done + succeeded + failed,
            total,
            failed,
            current_file: None,
            bytes_per_sec: None,
            message: format!("{phase_owned} done — ok {succeeded}, fail {failed}"),
            active: true,
        },
    );
    Ok((succeeded, failed))
}

/// Helper for callers that need to clear the dock on failure.
pub fn emit_idle(app: Option<&AppHandle>, message: impl Into<String>) {
    emit_progress(
        app,
        DownloadProgress {
            phase: "idle".into(),
            done: 0,
            total: 0,
            failed: 0,
            current_file: None,
            bytes_per_sec: None,
            message: message.into(),
            active: false,
        },
    );
}

/// Shared Arc client handle for modules that build many requests (optional).
#[allow(dead_code)]
pub fn client_arc() -> Result<Arc<reqwest::blocking::Client>, String> {
    Ok(Arc::new(shared_client()?.clone()))
}
