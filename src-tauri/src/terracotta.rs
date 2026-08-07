//! Terracotta sidecar integration.
//!
//! License posture (keeps Northstar off AGPL):
//! - We download and run **unmodified** official Terracotta binaries only.
//! - We never link Terracotta source/crates into this process.
//! - We talk only over Terracotta's local HTTP IPC (`--hmcl` mode).
//! - The UI must show Terracotta copyright / AGPL attribution.
//!
//! Upstream: https://github.com/burningtnt/Terracotta (AGPL-3.0-or-later + exception)

use crate::auth::active_account;
use crate::console_log;
use crate::download::{download_file_checked, sha512_file_matches};
use crate::paths::app_root;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tar::Archive;
use tauri::AppHandle;
use walkdir::WalkDir;

const VERSION: &str = "0.4.2";
const NODE_LIST_URL: &str = "https://terracotta.glavo.site/nodes";
const UPSTREAM_URL: &str = "https://github.com/burningtnt/Terracotta";
const UPSTREAM_LICENSE: &str = "AGPL-3.0-or-later";

/// Release mirrors, mirroring HMCL's `terracotta.json` download list.
const RELEASE_MIRRORS: [&str; 4] = [
    "https://github.com/burningtnt/Terracotta/releases/download/v${version}",
    "https://gitee.com/burningtnt/Terracotta/releases/download/v${version}",
    "https://cnb.cool/HMCL-Terracotta/Terracotta/-/releases/download/v${version}",
    "https://alist.8mi.tech/d/mirror/HMCL-Terracotta/Auto/v${version}",
];

/// Official SHA-512 of each `-pkg.tar.gz`, from HMCL's generated
/// `assets/terracotta.json` for 0.4.2. Used to reject a partial or tampered
/// archive instead of caching a broken file forever.
fn package_sha512(classifier: &str) -> Option<&'static str> {
    Some(match classifier {
        "windows-x86_64" => "6a98f524d4f00373696517306af8aa50d01d55ce4eadb27e9e4bc2f882707a0b5f20d5d4c33371d1459dcf5bf144ffed9beb414202d9ccf32b11dbbfcf19d650",
        "windows-arm64" => "fc1077247014ac0c712469498bde2ef7f6d881d5fcb7bdd5e11ebe20218fed365be19afdb8d453a79d77b729f866058522b910741767f4df947faa891434b463",
        "macos-x86_64" => "a762e4b2d6f84e899292b9e3856d009411a516d3c47f54575f843ce082f63dff2baa68ba0faa844b8b64fb12e91017386f15f5e7f975f8ee605bf8d4217cb091",
        "macos-arm64" => "09e444fea2d9fd19f3e5cb62e29055228345be163924cbd408d947646fafed1012cf48508ee6a155ede3d571e2ffaa72d09ceeb1493c8a60feb05e0699f19ba3",
        "linux-x86_64" => "d326ad95815d04568d485b5038e40ffc47ca54292fa0925eee6f5cea014024f901d661708aac2a743037b990882ad82b4d0b7bb03dc3b2fe720dbf0f3efe1c98",
        "linux-arm64" => "57c08f48d9535e93ad547d2dfc852d267992cc164a7208b42a2da0a6cbc2f21862f610e02a746b4b67150f4dec26b86a4f96eb9bd2f58d124d5b40ba50c6d55e",
        _ => return None,
    })
}

/// SHA-512 of the executable inside the package, so a half-extracted install
/// is detected as "not installed" rather than failing at launch.
fn binary_sha512(classifier: &str) -> Option<&'static str> {
    Some(match classifier {
        "windows-x86_64" => "6e98d1f2380ed22fb5a2dd4aafce6c773e9cf69100c8bb8e49e7d6983756bdb9a31f80e06bcfbe5a2742144fe806d3d687dec54d8f09d87c659341f99dd9fd80",
        "windows-arm64" => "30a15c5c53e5817c5a3634532172559327474741d3b2c7ef4e8a30acc6f59cdcf3570bf5f583e3cbe9e2abc8253e977c1abda1e9f36c88c4e99240da257347d0",
        "macos-x86_64" => "24efb85390eff88a538ed7e503fb1488e5e622730ca30c741a0e8b4c8f4e8d4868a2f9f38da8de540aeb535af2fd1e41c7081dae9c700e8a1a03b6c540218164",
        "macos-arm64" => "8e59a9d78acd57702dc044d6f2799c6af586b075a262f7d4dbbf0876e1af8d8271e04783c24ff820b801e3b14cd0190ab8403d097f3f2d98b6d911f95ed1e972",
        "linux-x86_64" => "fac328ba8957a711b03557bb913940f22d61b76608cd203fdf51024b6f94b19f5bc91c9b8a9fa80baf6968e1e6873c1880fd4cf54a2f8e3c6cf1e6ac161f8d0c",
        "linux-arm64" => "d807744c2041c98686e4b505324713badea7a0f31e8810be49ae053a63fb6dfc474ac58d678fb93eea0dd5cccff7372d9ec6135a1046f4b306cad35cd90ecacd",
        _ => return None,
    })
}

struct Runtime {
    /// The server process, when this launcher spawned it. `None` once we have
    /// attached to a server that some other process owns.
    child: Option<Child>,
    port: u16,
    /// Temp directory holding the `--hmcl` port handoff file. Kept outside the
    /// install dir so a reinstall never has to delete a path the sidecar holds.
    work_dir: Option<PathBuf>,
}

impl Runtime {
    fn is_alive(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => !matches!(child.try_wait(), Ok(Some(_))),
            // An adopted server belongs to another process, so ask the machine
            // rather than a handle we do not hold.
            None => read_lock_port() == Some(self.port) && port_answers(self.port),
        }
    }
}

/// Terracotta keeps one server per machine and records that server's HTTP port
/// in a lock file under its own data root, as two big-endian bytes.
fn terracotta_file_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("terracotta");
    }
    std::env::temp_dir().join("terracotta")
}

fn read_lock_port() -> Option<u16> {
    let path = terracotta_file_root().join("terracotta.lock");
    let mut buf = [0u8; 2];
    let mut file = open_shared(&path)?;
    if file.read(&mut buf).ok()? != 2 {
        return None;
    }
    Some(u16::from_be_bytes(buf)).filter(|p| *p != 0)
}

#[cfg(windows)]
fn open_shared(path: &Path) -> Option<File> {
    use std::os::windows::fs::OpenOptionsExt;
    // The live server keeps the lock open for read+write, so a default open
    // (which requests read sharing only) is denied with "access denied".
    const FILE_SHARE_READ_WRITE: u32 = 0x0000_0003;
    fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ_WRITE)
        .open(path)
        .ok()
}

#[cfg(not(windows))]
fn open_shared(path: &Path) -> Option<File> {
    File::open(path).ok()
}

/// Cheap liveness probe: something is listening on the loopback port.
fn port_answers(port: u16) -> bool {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
    TcpStream::connect_timeout(
        &SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        Duration::from_millis(400),
    )
    .is_ok()
}

/// Port of a Terracotta server that is already up, whoever started it.
fn discover_server() -> Option<u16> {
    let port = read_lock_port()?;
    if !port_answers(port) {
        return None;
    }
    http_get(port, "/meta").ok()?.get("version").map(|_| port)
}

/// Terracotta is a GUI-subsystem process that redirects its own stdio into
/// `<data root>/<timestamp>-<pid>/application.log`, so on a failed start that
/// file is the only place the reason is written down.
fn latest_sidecar_log(max_lines: usize) -> Option<String> {
    let newest = fs::read_dir(terracotta_file_root())
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let log = e.path().join("application.log");
            let at = log.metadata().and_then(|m| m.modified()).ok()?;
            Some((at, log))
        })
        .max_by_key(|(at, _)| *at)?
        .1;
    let text = fs::read_to_string(&newest).ok()?;
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return None;
    }
    let tail = &lines[lines.len().saturating_sub(max_lines)..];
    Some(tail.join("\n"))
}

static RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);

/// Identity of a checked file (path, size, mtime) plus the verification result.
type VerifiedBinary = (PathBuf, u64, Option<std::time::SystemTime>, bool);

/// Cached executable verification, keyed by identity of the file on disk.
/// `terracotta_info` is polled by the UI and the binary is tens of megabytes,
/// so it must not be re-hashed on every call.
static VERIFIED: Mutex<Option<VerifiedBinary>> = Mutex::new(None);

fn binary_is_intact(path: &Path, expected: &str) -> bool {
    let meta = fs::metadata(path).ok();
    let len = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let modified = meta.as_ref().and_then(|m| m.modified().ok());

    let mut guard = VERIFIED.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((cached_path, cached_len, cached_mtime, ok)) = guard.as_ref() {
        if cached_path == path && *cached_len == len && *cached_mtime == modified {
            return *ok;
        }
    }
    let ok = sha512_file_matches(path, expected);
    *guard = Some((path.to_path_buf(), len, modified, ok));
    ok
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerracottaInfo {
    pub version: String,
    pub installed: bool,
    pub running: bool,
    pub port: Option<u16>,
    pub binary_path: Option<String>,
    pub install_dir: String,
    pub supported: bool,
    pub platform_classifier: String,
    pub upstream_name: String,
    pub upstream_url: String,
    pub upstream_license: String,
    pub attribution: String,
    pub license_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TerracottaProfile {
    pub machine_id: Option<String>,
    pub name: Option<String>,
    pub vendor: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TerracottaState {
    pub phase: String,
    pub index: Option<i64>,
    pub port: Option<u16>,
    pub room: Option<String>,
    pub url: Option<String>,
    pub difficulty: Option<String>,
    pub profiles: Vec<TerracottaProfile>,
    pub profile_index: Option<i64>,
    pub exception_type: Option<i64>,
    pub raw_state: Option<String>,
    pub message: Option<String>,
}

fn terracotta_root() -> Result<PathBuf, String> {
    let p = app_root()?.join("terracotta").join(VERSION);
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

fn platform_classifier() -> Option<&'static str> {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return Some("windows-x86_64");
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        return Some("windows-arm64");
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return Some("linux-x86_64");
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return Some("linux-arm64");
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return Some("macos-x86_64");
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return Some("macos-arm64");
    }
    #[allow(unreachable_code)]
    None
}

fn package_name(classifier: &str) -> String {
    format!("terracotta-{VERSION}-{classifier}-pkg.tar.gz")
}

fn expected_exe_name(classifier: &str) -> String {
    #[cfg(windows)]
    {
        format!("terracotta-{VERSION}-{classifier}.exe")
    }
    #[cfg(not(windows))]
    {
        format!("terracotta-{VERSION}-{classifier}")
    }
}

fn find_binary(root: &Path, classifier: &str) -> Option<PathBuf> {
    let want = expected_exe_name(classifier);
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.eq_ignore_ascii_case(&want) {
            return Some(entry.path().to_path_buf());
        }
    }
    // Fallback: any terracotta executable in the tree.
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if name.starts_with("terracotta") && (name.ends_with(".exe") || !name.contains('.')) {
            return Some(entry.path().to_path_buf());
        }
    }
    None
}

fn attribution() -> String {
    format!(
        "Powered by Terracotta v{VERSION} ({UPSTREAM_LICENSE}). Copyright of the Terracotta authors. {UPSTREAM_URL}"
    )
}

fn license_note() -> String {
    "Northstar bundles/runs unmodified Terracotta binaries and talks to them only over local HTTP IPC. Terracotta remains AGPL-3.0-or-later; Northstar itself is not relicensed by this integration (see Terracotta's AGPL exception for binary packaging + IPC)."
        .into()
}

pub fn terracotta_info() -> Result<TerracottaInfo, String> {
    let classifier = platform_classifier().unwrap_or("unsupported");
    let install_dir = terracotta_root()?;
    let binary = find_binary(&install_dir, classifier);
    // A half-extracted or corrupted executable must read as "not installed",
    // otherwise Start fails forever with no obvious way to recover.
    let verified = match (&binary, binary_sha512(classifier)) {
        (Some(path), Some(want)) => binary_is_intact(path, want),
        (Some(_), None) => true,
        (None, _) => false,
    };
    let running = with_runtime(|rt| rt.as_ref().map(|r| (true, Some(r.port))))
        .unwrap_or((false, None));
    Ok(TerracottaInfo {
        version: VERSION.into(),
        installed: verified,
        running: running.0,
        port: running.1,
        binary_path: binary.map(|p| p.to_string_lossy().to_string()),
        install_dir: install_dir.to_string_lossy().to_string(),
        supported: platform_classifier().is_some(),
        platform_classifier: classifier.into(),
        upstream_name: "Terracotta".into(),
        upstream_url: UPSTREAM_URL.into(),
        upstream_license: UPSTREAM_LICENSE.into(),
        attribution: attribution(),
        license_note: license_note(),
    })
}

fn with_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&mut Option<Runtime>) -> R,
{
    let mut guard = RUNTIME.lock().unwrap_or_else(|e| e.into_inner());
    if guard.as_mut().is_some_and(|rt| !rt.is_alive()) {
        *guard = None;
    }
    f(&mut guard)
}

fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let dec = GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    archive.unpack(dest).map_err(|e| e.to_string())?;
    Ok(())
}

/// Windows keeps a lock for a moment after a process dies and refuses to delete
/// read-only files, both of which surface as "os error 5". Clear the read-only
/// bit and retry briefly before giving up.
fn force_remove_dir_all(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if let Ok(meta) = entry.metadata() {
            let mut perms = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            if perms.readonly() {
                perms.set_readonly(false);
                let _ = fs::set_permissions(entry.path(), perms);
            }
        }
    }
    let mut last = String::new();
    for attempt in 0..5 {
        match fs::remove_dir_all(dir) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last = e.to_string();
                thread::sleep(Duration::from_millis(150 * (attempt + 1)));
            }
        }
    }
    Err(last)
}

/// Terminate Terracotta processes that were launched from our install dir.
/// Scoped by executable path so a Terracotta owned by another launcher (HMCL)
/// is never touched.
fn kill_stale_sidecars(root: &Path, app: Option<&AppHandle>) {
    let _ = terracotta_stop(app);

    #[cfg(windows)]
    {
        let prefix = root.to_string_lossy().replace('\'', "''");
        let script = format!(
            "Get-CimInstance Win32_Process | Where-Object {{ $_.ExecutablePath -like '{prefix}*' }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }}"
        );
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        crate::win_cmd::hide_console(&mut cmd);
        let _ = cmd.status();
        // Give the kernel a moment to release file handles.
        thread::sleep(Duration::from_millis(300));
    }
    #[cfg(not(windows))]
    {
        let pattern = root.join("terracotta").to_string_lossy().to_string();
        let _ = Command::new("pkill")
            .arg("-f")
            .arg(&pattern)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// Download + extract the official unmodified platform package.
///
/// Extraction is staged in a sibling directory and swapped in, so a locked or
/// partially written install directory can never leave the user with neither
/// the old nor the new copy.
pub fn terracotta_install(app: Option<&AppHandle>) -> Result<TerracottaInfo, String> {
    let classifier = platform_classifier()
        .ok_or_else(|| "Terracotta is not supported on this OS/CPU yet".to_string())?;
    let root = terracotta_root()?;
    let pkg = package_name(classifier);
    let cache = app_root()?.join("terracotta").join("cache");
    fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let archive = cache.join(&pkg);

    // A sidecar still holding the install dir is the usual cause of
    // "Access is denied. (os error 5)" on reinstall.
    kill_stale_sidecars(&root, app);

    console_log::append(
        app,
        format!("[terracotta] Downloading unmodified {pkg}…"),
        "info",
    );

    let expected = package_sha512(classifier);
    let mut last_err = String::new();
    let mut ok = false;
    for mirror in RELEASE_MIRRORS {
        let url = format!("{}/{pkg}", mirror.replace("${version}", VERSION));
        match download_file_checked(&url, &archive, expected) {
            Ok(()) => {
                ok = true;
                break;
            }
            Err(e) => {
                console_log::append(app, format!("[terracotta] Mirror failed: {e}"), "warn");
                last_err = e;
            }
        }
    }
    if !ok {
        crate::download::emit_idle(app, "Terracotta download failed");
        return Err(format!("Failed to download Terracotta package: {last_err}"));
    }

    console_log::append(
        app,
        format!("[terracotta] Extracting to {}…", root.display()),
        "info",
    );

    let parent = root
        .parent()
        .ok_or_else(|| "Invalid Terracotta install path".to_string())?;
    let staging = parent.join(format!("{VERSION}.staging"));
    let retired = parent.join(format!("{VERSION}.old"));
    force_remove_dir_all(&staging)?;
    force_remove_dir_all(&retired)?;

    if let Err(e) = extract_tar_gz(&archive, &staging) {
        let _ = force_remove_dir_all(&staging);
        return Err(format!("Failed to extract Terracotta package: {e}"));
    }

    if find_binary(&staging, classifier).is_none() {
        let _ = force_remove_dir_all(&staging);
        return Err("Terracotta package did not contain the expected executable".into());
    }

    // Swap: retire the old tree first so a locked file fails before we delete.
    if root.exists() && fs::rename(&root, &retired).is_err() {
        force_remove_dir_all(&root).map_err(|e| {
            format!(
                "Could not replace the existing Terracotta install at {} ({e}). \
                 Close any running Terracotta window and try again.",
                root.display()
            )
        })?;
    }
    fs::rename(&staging, &root).map_err(|e| {
        format!(
            "Could not move the new Terracotta files into {} ({e}).",
            root.display()
        )
    })?;
    let _ = force_remove_dir_all(&retired);

    #[cfg(unix)]
    if let Some(bin) = find_binary(&root, classifier) {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&bin).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bin, perms).map_err(|e| e.to_string())?;
    }

    let notice = root.join("NORTHSTAR_TERRACOTTA_NOTICE.txt");
    let _ = fs::write(
        notice,
        format!(
            "{}\n{}\nThis directory contains unmodified Terracotta release files.\nDo not modify binaries if you rely on the AGPL packaging exception.\n",
            attribution(),
            license_note()
        ),
    );

    console_log::append(app, "[terracotta] Install complete.", "info");
    crate::download::emit_idle(app, "Terracotta installed");
    terracotta_info()
}

fn player_name() -> String {
    active_account()
        .ok()
        .flatten()
        .map(|a| a.username)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Player".into())
}

fn fetch_public_nodes() -> Vec<String> {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let Ok(resp) = client
        .get(NODE_LIST_URL)
        .header("User-Agent", "Northstar/1.2.4")
        .send()
    else {
        return Vec::new();
    };
    let Ok(data) = resp.json::<Value>() else {
        return Vec::new();
    };
    data.as_array()
        .into_iter()
        .flatten()
        .filter_map(|n| n.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()))
        .collect()
}

struct Spawned {
    /// `None` when the process we launched is only a launcher for the real
    /// server, so that killing it later would be pointless.
    child: Option<Child>,
    work_dir: PathBuf,
    port: u16,
}

/// Argument that makes the binary *be* the server instead of launching one.
///
/// On Windows `--hmcl` is only a trampoline: it re-spawns itself as a detached
/// `--hmcl2` and returns as soon as the port file appears (at most eight
/// seconds). Holding that trampoline as our child meant the launcher concluded
/// the sidecar had died while the real server kept running — orphaned, still
/// holding Terracotta's machine-wide lock and a file lock on the executable,
/// which is what made a reinstall fail with "os error 5". Asking for `--hmcl2`
/// directly makes the server our own child. Elsewhere `--hmcl` already runs the
/// server in-process.
#[cfg(windows)]
const SERVER_MODES: [&str; 2] = ["--hmcl2", "--hmcl"];
#[cfg(not(windows))]
const SERVER_MODES: [&str; 1] = ["--hmcl"];

fn spawn_sidecar(binary: &Path, app: Option<&AppHandle>) -> Result<Spawned, String> {
    let mut last = String::new();
    for mode in SERVER_MODES {
        match spawn_sidecar_mode(binary, mode, app) {
            Ok(spawned) => return Ok(spawned),
            Err(e) => {
                console_log::append(
                    app,
                    format!("[terracotta] `{mode}` mode did not come up: {e}"),
                    "warn",
                );
                last = e;
            }
        }
    }
    if let Some(log) = latest_sidecar_log(25) {
        console_log::append(app, format!("[terracotta] Sidecar log:\n{log}"), "error");
    }
    Err(last)
}

fn spawn_sidecar_mode(
    binary: &Path,
    mode: &str,
    app: Option<&AppHandle>,
) -> Result<Spawned, String> {
    // The port handoff file lives in a temp dir (as HMCL does) rather than the
    // install dir, so reinstalling never has to delete a path the sidecar owns.
    let work_dir = std::env::temp_dir().join(format!(
        "northstar-terracotta-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&work_dir).map_err(|e| e.to_string())?;
    let port_file = work_dir.join("http");

    console_log::append(
        app,
        format!(
            "[terracotta] Starting unmodified sidecar: {} {mode} …",
            binary.display()
        ),
        "info",
    );

    let mut cmd = Command::new(binary);
    // No `current_dir` on the install dir: on Windows a process's working
    // directory cannot be deleted, which broke reinstall with os error 5.
    cmd.arg(mode)
        .arg(&port_file)
        .current_dir(&work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    crate::win_cmd::hide_console(&mut cmd);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn Terracotta: {e}"))?;
    // A trampoline exits on purpose, so its handle must not be mistaken for the
    // server's lifetime.
    let owns_server = mode != "--hmcl" || !cfg!(windows);

    let started = Instant::now();
    let mut exited_at: Option<Instant> = None;
    loop {
        if let Some(port) = read_port_file(&port_file) {
            return Ok(Spawned {
                child: owns_server.then_some(child),
                work_dir,
                port,
            });
        }
        if started.elapsed() > Duration::from_secs(45) {
            let _ = child.kill();
            let _ = force_remove_dir_all(&work_dir);
            return Err("Timed out waiting for Terracotta to report its HTTP port".into());
        }
        if let Ok(Some(status)) = child.try_wait() {
            let since = *exited_at.get_or_insert_with(Instant::now);
            // A server that finds one already running hands the port back and
            // exits, so give the file a moment and then look for that server.
            if since.elapsed() > Duration::from_secs(6) {
                if let Some(port) = discover_server() {
                    return Ok(Spawned {
                        child: None,
                        work_dir,
                        port,
                    });
                }
                let _ = force_remove_dir_all(&work_dir);
                return Err(format!(
                    "Terracotta exited before reporting a port (status {status})"
                ));
            }
        }
        thread::sleep(Duration::from_millis(150));
    }
}

fn read_port_file(port_file: &Path) -> Option<u16> {
    let raw = fs::read_to_string(port_file).ok()?;
    let port = serde_json::from_str::<Value>(&raw)
        .ok()?
        .get("port")?
        .as_u64()?;
    u16::try_from(port).ok().filter(|p| *p != 0)
}

/// Where the port of the sidecar *we* started is remembered, so a later session
/// can tell our own leftover server apart from one belonging to another launcher.
fn session_port_file() -> Result<PathBuf, String> {
    Ok(terracotta_root()?.join("session-port"))
}

fn remember_session_port(port: u16) {
    if let Ok(path) = session_port_file() {
        let _ = fs::write(path, port.to_string());
    }
}

fn forget_session_port() {
    if let Ok(path) = session_port_file() {
        let _ = fs::remove_file(path);
    }
}

fn session_port() -> Option<u16> {
    fs::read_to_string(session_port_file().ok()?)
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Attach to a sidecar that outlived a previous launcher session, so the UI
/// reflects reality instead of offering a Start that has nothing to do.
///
/// Only a server this launcher started is adopted implicitly: silently taking
/// ownership of one belonging to HMCL or PCL would make our exit guard refuse to
/// quit — and our Stop button shut down — another application's process.
pub fn adopt_existing(app: Option<&AppHandle>) {
    if with_runtime(|rt| rt.is_some()) {
        return;
    }
    let Some(port) = discover_server() else {
        return;
    };
    if session_port() != Some(port) {
        return;
    }
    console_log::append(
        app,
        format!("[terracotta] Reattached to a sidecar left running on http://127.0.0.1:{port}"),
        "info",
    );
    with_runtime(|rt| {
        *rt = Some(Runtime {
            child: None,
            port,
            work_dir: None,
        })
    });
}

pub fn terracotta_start(app: Option<&AppHandle>) -> Result<TerracottaInfo, String> {
    let classifier = platform_classifier()
        .ok_or_else(|| "Terracotta is not supported on this OS/CPU yet".to_string())?;
    let root = terracotta_root()?;
    let binary = find_binary(&root, classifier).ok_or_else(|| {
        "Terracotta is not installed. Click Install to download the official unmodified package."
            .to_string()
    })?;

    // Already running?
    if with_runtime(|rt| rt.is_some()) {
        return terracotta_info();
    }

    // Only one Terracotta server can exist per machine (it holds a global lock),
    // so a second one would just hand its port back and exit. Attach to the live
    // one instead — this also recovers a server left behind by an earlier crash.
    if let Some(port) = discover_server() {
        console_log::append(
            app,
            format!("[terracotta] Attached to the sidecar already running on http://127.0.0.1:{port}"),
            "info",
        );
        with_runtime(|rt| {
            *rt = Some(Runtime {
                child: None,
                port,
                work_dir: None,
            })
        });
        remember_session_port(port);
        let _ = http_get_retry(port, "/state/ide", 5);
        return terracotta_info();
    }

    let spawned = spawn_sidecar(&binary, app)?;
    let port = spawned.port;
    with_runtime(|rt| {
        *rt = Some(Runtime {
            child: spawned.child,
            port,
            work_dir: Some(spawned.work_dir),
        });
    });
    remember_session_port(port);

    console_log::append(
        app,
        format!("[terracotta] Sidecar ready on http://127.0.0.1:{port}"),
        "info",
    );
    // Move to the idle/waiting state. The sidecar may need a moment more before
    // its HTTP server answers, so this is retried rather than treated as fatal.
    let _ = http_get_retry(port, "/state/ide", 5);
    terracotta_info()
}

pub fn terracotta_stop(app: Option<&AppHandle>) -> Result<TerracottaInfo, String> {
    forget_session_port();
    let Some(mut runtime) = with_runtime(|rt| rt.take()) else {
        return terracotta_info();
    };
    // Terracotta shuts itself down cleanly through this endpoint (it is how a
    // newer build takes over from an older one), which also releases the
    // machine-wide lock and the file lock on the executable.
    let _ = http_get(runtime.port, "/panic?peaceful=true");
    for _ in 0..20 {
        if !port_answers(runtime.port) {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    if let Some(child) = runtime.child.as_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
    if let Some(dir) = runtime.work_dir.as_deref() {
        let _ = force_remove_dir_all(dir);
    }
    console_log::append(app, "[terracotta] Sidecar stopped.", "info");
    terracotta_info()
}

/// True while this process owns a live sidecar. Used by the exit guard.
pub fn is_running() -> bool {
    with_runtime(|rt| rt.is_some())
}

fn http_get(port: u16, path_and_query: &str) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{port}{path_and_query}");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(&url)
        .header("User-Agent", "Northstar/1.2.4")
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    // Some endpoints return empty / non-JSON; treat as ok object.
    let text = resp.text().map_err(|e| e.to_string())?;
    if text.trim().is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    serde_json::from_str(&text).or_else(|_| Ok(serde_json::json!({ "raw": text })))
}

/// The sidecar's HTTP server briefly refuses connections around startup and
/// state transitions. HMCL retries five times; a single failure is not an error.
fn http_get_retry(port: u16, path_and_query: &str, attempts: usize) -> Result<Value, String> {
    let mut last = String::new();
    for attempt in 0..attempts.max(1) {
        match http_get(port, path_and_query) {
            Ok(v) => return Ok(v),
            Err(e) => {
                last = e;
                thread::sleep(Duration::from_millis(120 * (attempt as u64 + 1)));
            }
        }
    }
    Err(last)
}

fn parse_state(port: u16, v: Value) -> TerracottaState {
    let state_name = v
        .get("state")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();
    let profiles = v
        .get("profiles")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
        .map(|p| TerracottaProfile {
            machine_id: p
                .get("machine_id")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            name: p.get("name").and_then(|x| x.as_str()).map(|s| s.to_string()),
            vendor: p
                .get("vendor")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            kind: p.get("kind").and_then(|x| x.as_str()).map(|s| s.to_string()),
        })
        .collect();

    let exception_type = v.get("type").and_then(|x| x.as_i64());
    let message = if state_name == "exception" {
        Some(exception_label(exception_type).to_string())
    } else {
        None
    };

    TerracottaState {
        phase: state_name.clone(),
        index: v.get("index").and_then(|x| x.as_i64()),
        port: Some(port),
        room: v.get("room").and_then(|x| x.as_str()).map(|s| s.to_string()),
        url: v.get("url").and_then(|x| x.as_str()).map(|s| s.to_string()),
        difficulty: v
            .get("difficulty")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        profiles,
        profile_index: v.get("profile_index").and_then(|x| x.as_i64()),
        exception_type,
        raw_state: Some(state_name),
        message,
    }
}

/// Upstream `TerracottaState.Exception.Type` ordinals.
fn exception_label(kind: Option<i64>) -> &'static str {
    match kind {
        Some(0) => "Could not reach the host",
        Some(1) => "The host closed the connection",
        Some(2) => "The guest tunnel crashed",
        Some(3) => "The host tunnel crashed",
        Some(4) => "The Minecraft server closed the connection",
        Some(5) => "The relay returned an invalid response",
        _ => "Terracotta reported a connection problem",
    }
}

fn require_port() -> Result<u16, String> {
    with_runtime(|rt| {
        rt.as_ref()
            .map(|r| r.port)
            .ok_or_else(|| "Terracotta sidecar is not running. Start it first.".to_string())
    })
}

pub fn terracotta_state() -> Result<TerracottaState, String> {
    let port = match require_port() {
        Ok(p) => p,
        Err(_) => {
            return Ok(TerracottaState {
                phase: "offline".into(),
                ..Default::default()
            });
        }
    };
    match http_get_retry(port, "/state", 3) {
        Ok(v) => Ok(parse_state(port, v)),
        // A poll that misses is normal while the sidecar is still coming up or
        // switching states. Only call it an error once the process is gone.
        Err(e) => Ok(TerracottaState {
            phase: if is_running() { "starting" } else { "error" }.into(),
            port: Some(port),
            message: if is_running() { None } else { Some(e) },
            ..Default::default()
        }),
    }
}

fn append_nodes_query(base: &str) -> String {
    let mut q = base.to_string();
    let player_raw = player_name();
    let player = urlencoding::encode(&player_raw);
    if q.contains('?') {
        q.push_str(&format!("&player={player}"));
    } else {
        q.push_str(&format!("?player={player}"));
    }
    for node in fetch_public_nodes() {
        q.push_str(&format!("&public_nodes={}", urlencoding::encode(&node)));
    }
    q
}

/// Terracotta only accepts host/guest transitions from `waiting`. Calling
/// `/state/ide` first mirrors what users do with "Return to idle" and avoids
/// a bare HTTP 400 when the sidecar is still hosting / reconnecting / in
/// exception.
fn ensure_waiting(port: u16) -> Result<(), String> {
    let _ = http_get_retry(port, "/state/ide", 3)?;
    // Give the sidecar a beat to publish Waiting before the next transition.
    thread::sleep(Duration::from_millis(80));
    Ok(())
}

/// Normalize pasted room codes (`u/…`, spaces, missing `U/` prefix).
fn normalize_room_code(raw: &str) -> Result<String, String> {
    let mut s = raw.trim().to_ascii_uppercase().replace([' ', '\t', '\n', '\r'], "");
    if s.is_empty() {
        return Err("Room code is required".into());
    }
    // Accept bare `XXXX-XXXX-XXXX-XXXX` pastes.
    if !s.contains('/') && s.len() == "XXXX-XXXX-XXXX-XXXX".len() {
        s = format!("U/{s}");
    }
    if !is_valid_room_code(&s) {
        return Err(format!(
            "Invalid room code `{s}`. Expected something like U/XXXX-XXXX-XXXX-XXXX."
        ));
    }
    Ok(s)
}

/// Same checksum rules as Terracotta's `Room::parse` (base-34, multiple of 7).
fn is_valid_room_code(code: &str) -> bool {
    const CHARS: &[u8] = b"0123456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    const NEEDLE: &str = "U/XXXX-XXXX-XXXX-XXXX";
    let chars: Vec<char> = code.to_ascii_uppercase().chars().collect();
    if chars.len() < NEEDLE.len() {
        return false;
    }
    let lookup = |c: char| -> Option<u8> {
        let c = match c {
            'I' => '1',
            'O' => '0',
            _ => c,
        };
        CHARS.iter().position(|&b| b as char == c).map(|i| i as u8)
    };
    for window in chars.windows(NEEDLE.len()) {
        if window[0] != 'U' || window[1] != '/' {
            continue;
        }
        let body = &window[2..];
        let mut value: u128 = 0;
        let mut ok = true;
        for i in (0.."XXXX-XXXX-XXXX-XXXX".len()).rev() {
            if i == 4 || i == 9 || i == 14 {
                if body[i] != '-' {
                    ok = false;
                    break;
                }
            } else {
                match lookup(body[i]) {
                    Some(v) => value = value * 34 + u128::from(v),
                    None => {
                        ok = false;
                        break;
                    }
                }
            }
        }
        if ok && value % 7 == 0 {
            return true;
        }
    }
    false
}

/// Host: start LAN scan / create room.
pub fn terracotta_host() -> Result<TerracottaState, String> {
    let port = require_port()?;
    ensure_waiting(port)?;
    let path = append_nodes_query("/state/scanning");
    match http_get_retry(port, &path, 3) {
        Ok(_) => terracotta_state(),
        Err(e) => Err(format!(
            "Could not start hosting ({e}). If a session is stuck, click Return to idle and try again."
        )),
    }
}

/// Guest: join a room code.
pub fn terracotta_join(room: String) -> Result<TerracottaState, String> {
    let room = normalize_room_code(&room)?;
    let port = require_port()?;
    ensure_waiting(port)?;
    let path = append_nodes_query(&format!(
        "/state/guesting?room={}",
        urlencoding::encode(&room)
    ));
    match http_get_retry(port, &path, 3) {
        Ok(_) => terracotta_state(),
        Err(e) => {
            if e.contains("400") {
                Err(format!(
                    "Terracotta rejected join for `{room}` (sidecar not idle or code rejected). Click Return to idle, then try again. ({e})"
                ))
            } else {
                Err(e)
            }
        }
    }
}

/// Return to idle waiting state.
pub fn terracotta_idle() -> Result<TerracottaState, String> {
    let port = require_port()?;
    let _ = http_get_retry(port, "/state/ide", 3)?;
    terracotta_state()
}
