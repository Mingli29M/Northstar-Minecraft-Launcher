use crate::console_log;
use crate::dedicated;
use crate::java::resolve_java_path;
use crate::paths::dedicated_runtime;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

static ORPHAN_MISS: Mutex<Option<(String, Instant)>> = Mutex::new(None);

fn find_java_for_runtime_cached(id: &str, runtime: &std::path::Path) -> Option<u32> {
    if let Ok(guard) = ORPHAN_MISS.lock() {
        if let Some((cached_id, at)) = guard.as_ref() {
            if cached_id == id && at.elapsed() < Duration::from_secs(20) {
                return None;
            }
        }
    }
    match find_java_for_runtime(runtime) {
        Some(pid) => {
            if let Ok(mut guard) = ORPHAN_MISS.lock() {
                *guard = None;
            }
            Some(pid)
        }
        None => {
            if let Ok(mut guard) = ORPHAN_MISS.lock() {
                *guard = Some((id.to_string(), Instant::now()));
            }
            None
        }
    }
}

/// PID listening on TCP port (Windows), if any.
fn pid_listening_on_port(port: u16) -> Option<u32> {
    #[cfg(windows)]
    {
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let needle = format!(":{port}");
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with("TCP") {
                continue;
            }
            if !line.contains("LISTENING") {
                continue;
            }
            // Local address column contains :port
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }
            let local = parts[1];
            if local.ends_with(&needle)
                || local.ends_with(&format!("]:{port}"))
            {
                if let Ok(pid) = parts[4].parse::<u32>() {
                    if pid > 0 {
                        return Some(pid);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(windows))]
    {
        let _ = port;
        None
    }
}

fn describe_port_holder(pid: u32) -> String {
    #[cfg(windows)]
    {
        let script = format!(
            "(Get-CimInstance Win32_Process -Filter \"ProcessId={pid}\").CommandLine"
        );
        if let Ok(out) = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
        {
            let cmd = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !cmd.is_empty() {
                if let Some(other) = dedicated::list_dedicated().ok().and_then(|list| {
                    list.into_iter().find(|s| {
                        cmd.to_lowercase()
                            .contains(&s.id.to_lowercase())
                    })
                }) {
                    return format!(
                        "Host \"{}\" (pid {pid}). Stop that server first, or change this server's port.",
                        other.name
                    );
                }
                if cmd.to_lowercase().contains("euml\\dedicated")
                    || cmd.to_lowercase().contains("server.jar")
                {
                    return format!("another Minecraft/EUML server process (pid {pid})");
                }
                let short = if cmd.len() > 120 {
                    format!("{}…", &cmd[..120])
                } else {
                    cmd
                };
                return format!("pid {pid}: {short}");
            }
        }
        format!("pid {pid}")
    }
    #[cfg(not(windows))]
    {
        format!("pid {pid}")
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedStatus {
    pub id: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub upnp_mapped: bool,
}

struct RunningServer {
    /// Present when this app session spawned/owns the process (stdin available).
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    pid: u32,
    upnp_mapped: bool,
    port: u16,
}

static PROCESSES: Mutex<Option<HashMap<String, RunningServer>>> = Mutex::new(None);

fn map() -> std::sync::MutexGuard<'static, Option<HashMap<String, RunningServer>>> {
    PROCESSES.lock().unwrap_or_else(|e| e.into_inner())
}

fn with_map<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<String, RunningServer>) -> R,
{
    let mut guard = map();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    f(guard.as_mut().unwrap())
}

fn resolve_java_for_host(server: &dedicated::HostServer) -> Result<String, String> {
    if let Some(path) = &server.java_path {
        if std::path::Path::new(path).exists() {
            return Ok(path.clone());
        }
    }
    resolve_java_path(&server.game_version, None)
}

pub fn process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(windows)]
    {
        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
        const STILL_ACTIVE: u32 = 259;
        #[link(name = "kernel32")]
        extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
            fn GetExitCodeProcess(handle: *mut std::ffi::c_void, code: *mut u32) -> i32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return false;
            }
            let mut code = 0u32;
            let ok = GetExitCodeProcess(h, &mut code);
            let _ = CloseHandle(h);
            ok != 0 && code == STILL_ACTIVE
        }
    }
    #[cfg(not(windows))]
    {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }
}

fn kill_pid(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        const PROCESS_TERMINATE: u32 = 0x0001;
        #[link(name = "kernel32")]
        extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
            fn TerminateProcess(handle: *mut std::ffi::c_void, code: u32) -> i32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }
        unsafe {
            let h = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if h.is_null() {
                return Err(format!("Could not open process {pid} to terminate"));
            }
            let ok = TerminateProcess(h, 1);
            let _ = CloseHandle(h);
            if ok == 0 {
                return Err(format!("TerminateProcess failed for pid {pid}"));
            }
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        Err("kill_pid is only implemented on Windows".into())
    }
}

/// Find a Java process whose command line references this server runtime (orphan recovery).
fn find_java_for_runtime(runtime: &std::path::Path) -> Option<u32> {
    #[cfg(windows)]
    {
        let needle = runtime.to_string_lossy().replace('/', "\\").to_lowercase();
        if needle.is_empty() {
            return None;
        }
        let script = format!(
            "Get-CimInstance Win32_Process -Filter \"Name='java.exe'\" | ForEach-Object {{ \"$($_.ProcessId)|$($_.CommandLine)\" }}"
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let Some((pid_s, cmd)) = line.split_once('|') else {
                continue;
            };
            if cmd.to_lowercase().contains(&needle) {
                if let Ok(pid) = pid_s.trim().parse::<u32>() {
                    if process_alive(pid) {
                        return Some(pid);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(windows))]
    {
        let _ = runtime;
        None
    }
}

/// Lower host Java priority so it yields CPU to the launcher / client.
#[cfg(windows)]
fn set_below_normal_priority(pid: u32) {
    const ACCESS: u32 = 0x0200 | 0x1000;
    const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn SetPriorityClass(handle: *mut std::ffi::c_void, class: u32) -> i32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    unsafe {
        let h = OpenProcess(ACCESS, 0, pid);
        if !h.is_null() {
            let _ = SetPriorityClass(h, BELOW_NORMAL_PRIORITY_CLASS);
            let _ = CloseHandle(h);
        }
    }
}

#[cfg(not(windows))]
fn set_below_normal_priority(_pid: u32) {}

fn persist_pid(id: &str, pid: u32) {
    let _ = dedicated::set_running_pid(id, Some(pid));
    if let Ok(runtime) = dedicated_runtime(id) {
        let _ = fs::write(runtime.join("euml-host.pid"), pid.to_string());
    }
}

fn clear_persisted_pid(id: &str) {
    let _ = dedicated::set_running_pid(id, None);
    if let Ok(runtime) = dedicated_runtime(id) {
        let _ = fs::remove_file(runtime.join("euml-host.pid"));
    }
}

fn read_pid_file(id: &str) -> Option<u32> {
    let runtime = dedicated_runtime(id).ok()?;
    let raw = fs::read_to_string(runtime.join("euml-host.pid")).ok()?;
    let pid: u32 = raw.trim().parse().ok()?;
    if process_alive(pid) {
        Some(pid)
    } else {
        let _ = fs::remove_file(runtime.join("euml-host.pid"));
        None
    }
}

pub fn dedicated_status(id: String) -> Result<DedicatedStatus, String> {
    let mut server = dedicated::get_dedicated(&id)?;
    Ok(with_map(|m| {
        reap(m);
        if let Some(rs) = m.get(&id) {
            return DedicatedStatus {
                id: id.clone(),
                running: true,
                pid: Some(rs.pid),
                upnp_mapped: rs.upnp_mapped,
            };
        }

        // Recover after app reload: host.json → pid file → scan Java cmdline
        let recovered = server
            .running_pid
            .filter(|p| process_alive(*p))
            .or_else(|| read_pid_file(&id))
            .or_else(|| {
                dedicated_runtime(&id)
                    .ok()
                    .and_then(|rt| find_java_for_runtime_cached(&id, &rt))
            });

        if let Some(pid) = recovered {
            if server.running_pid != Some(pid) {
                let _ = dedicated::set_running_pid(&id, Some(pid));
                server.running_pid = Some(pid);
            }
            if let Ok(runtime) = dedicated_runtime(&id) {
                let _ = fs::write(runtime.join("euml-host.pid"), pid.to_string());
            }
            m.insert(
                id.clone(),
                RunningServer {
                    child: None,
                    stdin: None,
                    pid,
                    upnp_mapped: false,
                    port: server.port,
                },
            );
            return DedicatedStatus {
                id,
                running: true,
                pid: Some(pid),
                upnp_mapped: false,
            };
        }

        if server.running_pid.is_some() {
            let _ = dedicated::set_running_pid(&id, None);
        }
        DedicatedStatus {
            id,
            running: false,
            pid: None,
            upnp_mapped: false,
        }
    }))
}

fn set_upnp_mapped(id: &str, mapped: bool) {
    with_map(|m| {
        if let Some(rs) = m.get_mut(id) {
            rs.upnp_mapped = mapped;
        }
    });
}

fn reap(m: &mut HashMap<String, RunningServer>) {
    let dead: Vec<(String, u32)> = m
        .iter_mut()
        .filter_map(|(id, rs)| {
            // Owned child: only reap on confirmed exit (never on try_wait Err)
            if let Some(child) = rs.child.as_mut() {
                return match child.try_wait() {
                    Ok(Some(_)) => Some((id.clone(), rs.pid)),
                    Ok(None) => None,
                    Err(_) => None,
                };
            }
            // Detached / reattached: poll OS
            if process_alive(rs.pid) {
                None
            } else {
                Some((id.clone(), rs.pid))
            }
        })
        .collect();
    for (id, _pid) in dead {
        m.remove(&id);
        clear_persisted_pid(&id);
    }
}

pub fn start_dedicated(app: AppHandle, id: String) -> Result<DedicatedStatus, String> {
    let mut server = dedicated::get_dedicated(&id)?;
    if !server.installed {
        return Err("Server is not installed yet".into());
    }
    if !server.eula_accepted {
        return Err("Accept the Minecraft EULA before starting".into());
    }
    let runtime = dedicated_runtime(&id)?;
    if !runtime.exists() {
        return Err(dedicated::not_found_msg().into());
    }

    // Already tracked in this session
    let already = with_map(|m| {
        reap(m);
        m.contains_key(&id)
    });
    if already {
        return dedicated_status(id);
    }

    // Orphan from a previous launcher session — adopt, do not double-start (world lock)
    if let Some(pid) = server.running_pid {
        if process_alive(pid) {
            console_log::append(
                Some(&app),
                format!(
                    "[host:{id}] Reattached to existing server process (pid {pid}). World was still locked."
                ),
                "server",
            );
            with_map(|m| {
                m.insert(
                    id.clone(),
                    RunningServer {
                        child: None,
                        stdin: None,
                        pid,
                        upnp_mapped: false,
                        port: server.port,
                    },
                );
            });
            return dedicated_status(id);
        }
        clear_persisted_pid(&id);
        server.running_pid = None;
    }

    // Recover orphans that predate PID tracking (world lock still held)
    if let Some(pid) = find_java_for_runtime(&runtime) {
        console_log::append(
            Some(&app),
            format!("[host:{id}] Found orphan Java for this world (pid {pid}) — reattaching."),
            "server",
        );
        persist_pid(&id, pid);
        with_map(|m| {
            m.insert(
                id.clone(),
                RunningServer {
                    child: None,
                    stdin: None,
                    pid,
                    upnp_mapped: false,
                    port: server.port,
                },
            );
        });
        return dedicated_status(id);
    }

    // Port already taken (e.g. another Host still running after app reload)
    if let Some(holder) = pid_listening_on_port(server.port) {
        // If somehow our own jar is listening under a path we didn't match, adopt
        if let Some(own) = find_java_for_runtime(&runtime) {
            if own == holder {
                persist_pid(&id, holder);
                with_map(|m| {
                    m.insert(
                        id.clone(),
                        RunningServer {
                            child: None,
                            stdin: None,
                            pid: holder,
                            upnp_mapped: false,
                            port: server.port,
                        },
                    );
                });
                return dedicated_status(id);
            }
        }
        let who = describe_port_holder(holder);
        let msg = format!(
            "Port {} is already in use by {who}",
            server.port
        );
        console_log::append(Some(&app), format!("[host:{id}] {msg}"), "error");
        return Err(msg);
    }

    let launch = dedicated::resolve_launch(&id)?;
    let java = resolve_java_for_host(&server)?;

    let xmx = server.memory_mb;
    let xms = (xmx / 2).max(512);
    let mut cmd = Command::new(&java);
    cmd.current_dir(&runtime)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let launch_desc = match &launch {
        dedicated::LaunchSpec::Jar(jar) => {
            cmd.arg(format!("-Xmx{xmx}M"))
                .arg(format!("-Xms{xms}M"))
                .arg("-jar")
                .arg(jar)
                .arg("nogui");
            jar.display().to_string()
        }
        dedicated::LaunchSpec::ForgeArgs {
            jvm_args,
            forge_args,
        } => {
            let _ = fs::write(
                jvm_args,
                format!("# Managed by EUML\n-Xmx{xmx}M\n-Xms{xms}M\n"),
            );
            cmd.arg(format!(
                "@{}",
                jvm_args
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("user_jvm_args.txt")
            ));
            let rel = forge_args
                .strip_prefix(&runtime)
                .unwrap_or(forge_args.as_path());
            cmd.arg(format!(
                "@{}",
                rel.display().to_string().replace('\\', "/")
            ));
            cmd.arg("nogui");
            format!("forge @{}", rel.display())
        }
    };

    console_log::append(
        Some(&app),
        format!("[host:{id}] Starting {} ({})", server.name, launch_desc),
        "server",
    );

    // Prefer breakaway so tauri:dev reloads don't kill the server via Job Object
    let mut child = {
        #[cfg(windows)]
        {
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
            let mut breakaway = Command::new(&java);
            breakaway
                .current_dir(&runtime)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB);
            // Re-apply args from `cmd` by rebuilding — simplest: try flags on a clone path
            // Rebuild args the same way:
            match &launch {
                dedicated::LaunchSpec::Jar(jar) => {
                    breakaway
                        .arg(format!("-Xmx{xmx}M"))
                        .arg(format!("-Xms{xms}M"))
                        .arg("-jar")
                        .arg(jar)
                        .arg("nogui");
                }
                dedicated::LaunchSpec::ForgeArgs {
                    jvm_args,
                    forge_args,
                } => {
                    breakaway.arg(format!(
                        "@{}",
                        jvm_args
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("user_jvm_args.txt")
                    ));
                    let rel = forge_args
                        .strip_prefix(&runtime)
                        .unwrap_or(forge_args.as_path());
                    breakaway.arg(format!(
                        "@{}",
                        rel.display().to_string().replace('\\', "/")
                    ));
                    breakaway.arg("nogui");
                }
            }
            match breakaway.spawn() {
                Ok(c) => c,
                Err(_) => cmd.spawn().map_err(|e| format!("Failed to spawn server: {e}"))?,
            }
        }
        #[cfg(not(windows))]
        {
            cmd.spawn()
                .map_err(|e| format!("Failed to spawn server: {e}"))?
        }
    };

    let pid = child.id();
    set_below_normal_priority(pid);

    if let Some(mask) = server.cpu_affinity_mask {
        if mask > 0 {
            let app_a = app.clone();
            let id_a = id.clone();
            thread::spawn(move || apply_affinity(pid, mask, &app_a, &id_a));
        }
    }

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to open server stdin".to_string())?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let log_dir = runtime.join("logs");
    fs::create_dir_all(&log_dir).ok();
    if let Some(out) = stdout {
        let app_c = app.clone();
        let id_c = id.clone();
        let path = log_dir.join("euml-host-stdout.txt");
        thread::spawn(move || tee(out, path, app_c, &id_c));
    }
    if let Some(err) = stderr {
        let app_c = app.clone();
        let id_c = id.clone();
        let path = log_dir.join("euml-host-stderr.txt");
        thread::spawn(move || tee(err, path, app_c, &id_c));
    }

    let port = server.port;
    persist_pid(&id, pid);

    with_map(|m| {
        m.insert(
            id.clone(),
            RunningServer {
                child: Some(child),
                stdin: Some(stdin),
                pid,
                upnp_mapped: false,
                port,
            },
        );
    });

    server.last_started = Some(chrono::Utc::now().to_rfc3339());
    server.running_pid = Some(pid);
    let _ = dedicated::save_dedicated(&server);

    console_log::append(
        Some(&app),
        format!("[host:{id}] Running pid={pid} (port map / health check in background)"),
        "server",
    );

    {
        let app_bg = app.clone();
        let id_bg = id.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1500));
            let exited = with_map(|m| {
                if let Some(rs) = m.get_mut(&id_bg) {
                    if let Some(child) = rs.child.as_mut() {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                let pid = rs.pid;
                                m.remove(&id_bg);
                                Some((status.to_string(), pid))
                            }
                            _ => None,
                        }
                    } else if !process_alive(rs.pid) {
                        let pid = rs.pid;
                        m.remove(&id_bg);
                        Some(("exited".into(), pid))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
            if let Some((status, _pid)) = exited {
                clear_persisted_pid(&id_bg);
                console_log::append(
                    Some(&app_bg),
                    format!(
                        "[host:{id_bg}] Server exited immediately ({status}). If the world is locked, Stop the orphan process or use Host → Stop."
                    ),
                    "error",
                );
                return;
            }

            console_log::append(
                Some(&app_bg),
                format!("[host:{id_bg}] Auto-mapping port (UPnP → NAT-PMP → PCP)…"),
                "server",
            );
            let map_info = crate::upnp::map_port_cascade(port);
            if map_info.map_method.is_some() {
                set_upnp_mapped(&id_bg, true);
                console_log::append(
                    Some(&app_bg),
                    format!(
                        "[host:{id_bg}] Port mapped via {} (TCP {port})",
                        map_info.map_method.as_deref().unwrap_or("?")
                    ),
                    "server",
                );
            } else {
                let detail = map_info
                    .map_attempts
                    .iter()
                    .map(|a| format!("{}: {}", a.method, a.message))
                    .collect::<Vec<_>>()
                    .join(" | ");
                console_log::append(
                    Some(&app_bg),
                    format!(
                        "[host:{id_bg}] Auto port map failed (UPnP→NAT-PMP→PCP). Manual forward needed. {detail}"
                    ),
                    "warn",
                );
            }
        });
    }

    dedicated_status(id)
}

pub fn stop_dedicated(app: AppHandle, id: String) -> Result<DedicatedStatus, String> {
    // Ensure orphaned PIDs are visible in the map
    let _ = dedicated_status(id.clone());

    let entry = with_map(|m| {
        reap(m);
        m.remove(&id)
    });
    let Some(mut rs) = entry else {
        clear_persisted_pid(&id);
        return Ok(DedicatedStatus {
            id,
            running: false,
            pid: None,
            upnp_mapped: false,
        });
    };

    console_log::append(
        Some(&app),
        format!("[host:{id}] Sending stop…"),
        "server",
    );

    if let Some(stdin) = rs.stdin.as_mut() {
        let _ = writeln!(stdin, "stop");
        let _ = stdin.flush();
    }

    let mut stopped = false;
    if let Some(child) = rs.child.as_mut() {
        for _ in 0..8 {
            thread::sleep(Duration::from_millis(400));
            match child.try_wait() {
                Ok(Some(_)) => {
                    stopped = true;
                    break;
                }
                Ok(None) => continue,
                Err(_) => break,
            }
        }
        if !stopped {
            let _ = child.kill();
            let _ = child.wait();
            console_log::append(
                Some(&app),
                format!("[host:{id}] Force-killed"),
                "warn",
            );
        }
    } else {
        // Reattached session — no stdin/child; terminate by PID
        thread::sleep(Duration::from_millis(500));
        if process_alive(rs.pid) {
            match kill_pid(rs.pid) {
                Ok(()) => console_log::append(
                    Some(&app),
                    format!("[host:{id}] Terminated reattached process pid={}", rs.pid),
                    "warn",
                ),
                Err(e) => console_log::append(
                    Some(&app),
                    format!("[host:{id}] Failed to terminate pid={}: {e}", rs.pid),
                    "error",
                ),
            }
        }
    }

    clear_persisted_pid(&id);

    if rs.upnp_mapped {
        let port = rs.port;
        let app_u = app.clone();
        let id_u = id.clone();
        thread::spawn(move || {
            let _ = crate::upnp::unmap_port(port);
            console_log::append(
                Some(&app_u),
                format!("[host:{id_u}] Port mapping removed"),
                "server",
            );
        });
    }

    console_log::append(Some(&app), format!("[host:{id}] Stopped"), "server");
    Ok(DedicatedStatus {
        id,
        running: false,
        pid: None,
        upnp_mapped: false,
    })
}

pub fn dedicated_send_command(id: String, command: String) -> Result<(), String> {
    let cmd = command.trim();
    if cmd.is_empty() {
        return Err("Empty command".into());
    }
    // Refresh reattached sessions into the map
    let _ = dedicated_status(id.clone());
    with_map(|m| {
        reap(m);
        let rs = m
            .get_mut(&id)
            .ok_or_else(|| "Server is not running".to_string())?;
        let stdin = rs.stdin.as_mut().ok_or_else(|| {
            "Server is running from a previous session — console stdin is not attached. Stop and Start once to restore console commands.".to_string()
        })?;
        writeln!(stdin, "{cmd}").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
        Ok(())
    })
}

fn tee(reader: impl std::io::Read, path: std::path::PathBuf, app: AppHandle, id: &str) {
    let mut file = File::create(&path).ok();
    let buf = BufReader::new(reader);
    let mut emitted = 0u32;
    let mut window_start = Instant::now();
    const MAX_UI_LINES_PER_SEC: u32 = 12;

    for line in buf.lines() {
        let Ok(line) = line else { break };
        if let Some(f) = file.as_mut() {
            let _ = writeln!(f, "{line}");
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lvl = if trimmed.contains("ERROR")
            || trimmed.contains("Exception")
            || trimmed.contains("FATAL")
        {
            "error"
        } else if trimmed.contains("WARN") {
            "warn"
        } else {
            "server"
        };

        if window_start.elapsed() >= Duration::from_secs(1) {
            window_start = Instant::now();
            emitted = 0;
        }
        let important = lvl == "error" || lvl == "warn";
        if important || emitted < MAX_UI_LINES_PER_SEC {
            console_log::append(Some(&app), format!("[host:{id}] {trimmed}"), lvl);
            if !important {
                emitted += 1;
            }
        }
    }
}

pub fn stop_if_running(app: Option<&AppHandle>, id: &str) {
    if let Some(app) = app {
        let _ = stop_dedicated(app.clone(), id.to_string());
    } else {
        let _ = dedicated_status(id.to_string());
        let _ = with_map(|m| {
            if let Some(mut rs) = m.remove(id) {
                if let Some(stdin) = rs.stdin.as_mut() {
                    let _ = writeln!(stdin, "stop");
                }
                thread::sleep(Duration::from_millis(800));
                if let Some(child) = rs.child.as_mut() {
                    let _ = child.kill();
                } else if process_alive(rs.pid) {
                    let _ = kill_pid(rs.pid);
                }
                clear_persisted_pid(id);
                if rs.upnp_mapped {
                    let _ = crate::upnp::unmap_port(rs.port);
                }
            }
        });
    }
}

fn apply_affinity(pid: u32, mask: u64, app: &AppHandle, id: &str) {
    #[cfg(windows)]
    {
        let script = format!(
            "try {{ (Get-Process -Id {pid}).ProcessorAffinity = {mask}; 'ok' }} catch {{ $_.Exception.Message }}"
        );
        match Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
        {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                let err = String::from_utf8_lossy(&out.stderr);
                if text.contains("ok") {
                    console_log::append(
                        Some(app),
                        format!("[host:{id}] CPU affinity set to mask 0x{mask:X}"),
                        "server",
                    );
                } else {
                    let msg = if text.trim().is_empty() {
                        err.trim().to_string()
                    } else {
                        text.trim().to_string()
                    };
                    console_log::append(
                        Some(app),
                        format!("[host:{id}] CPU affinity failed: {msg}"),
                        "warn",
                    );
                }
            }
            Err(e) => {
                console_log::append(
                    Some(app),
                    format!("[host:{id}] CPU affinity unavailable: {e}"),
                    "warn",
                );
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (pid, mask);
        console_log::append(
            Some(app),
            format!("[host:{id}] CPU affinity is only supported on Windows"),
            "warn",
        );
    }
}

pub fn logical_cpu_count() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
}
