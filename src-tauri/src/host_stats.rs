use crate::console_log;
use crate::dedicated;
use crate::host_process;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HostLiveStats {
    pub players_online: u32,
    pub players_max: Option<u32>,
    pub player_names: Vec<String>,
    pub tps: Option<f32>,
    pub mspt: Option<f32>,
    pub entity_count: Option<u32>,
    pub mob_count: Option<u32>,
    /// Process CPU usage 0–100+ (can exceed 100 on multi-core)
    pub cpu_percent: Option<f32>,
    /// Process working set (MB)
    pub ram_used_mb: Option<f64>,
    /// System total physical RAM (MB)
    pub ram_total_mb: Option<f64>,
    /// System used physical RAM (MB)
    pub ram_system_used_mb: Option<f64>,
    /// Aggregate NIC download rate (bytes/sec)
    pub net_down_bps: Option<f64>,
    /// Aggregate NIC upload rate (bytes/sec)
    pub net_up_bps: Option<f64>,
    pub note: String,
}

struct ProcSample {
    cpu_seconds: f64,
    at: Instant,
}

struct NetSample {
    recv: u64,
    sent: u64,
    at: Instant,
}

struct SamplerState {
    proc: HashMap<u32, ProcSample>,
    net: Option<NetSample>,
}

static SAMPLER: Mutex<Option<SamplerState>> = Mutex::new(None);

fn sampler() -> std::sync::MutexGuard<'static, Option<SamplerState>> {
    SAMPLER.lock().unwrap_or_else(|e| e.into_inner())
}

fn list_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)There are (\d+) of a max of (\d+) players online(?::\s*(.*))?").unwrap()
    })
}

fn tps_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(?:TPS from last 1m.*?[:：]\s*([\d.]+)|TPS:\s*([\d.]+)|Average TPS:\s*([\d.]+))",
        )
        .unwrap()
    })
}

fn mspt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(?:Mean tick time|MSPT)[:：]?\s*([\d.]+)").unwrap())
}

fn entity_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(?:Total entities|Entity count|Entities)[:：]?\s*(\d+)").unwrap()
    })
}

/// Send probe commands and parse recent console lines; attach CPU/RAM/net rates.
pub fn refresh_live_stats(app: Option<&tauri::AppHandle>, id: &str) -> Result<HostLiveStats, String> {
    let server = dedicated::get_dedicated(id)?;
    let status = host_process::dedicated_status(id.to_string())?;
    if !status.running {
        return Ok(HostLiveStats {
            note: "Server is not running".into(),
            ..Default::default()
        });
    }

    let _ = host_process::dedicated_send_command(id.to_string(), "list".into());
    match server.loader.as_str() {
        "paper" | "purpur" => {
            let _ = host_process::dedicated_send_command(id.to_string(), "tps".into());
        }
        "forge" | "neoforge" => {
            let _ = host_process::dedicated_send_command(id.to_string(), "forge tps".into());
        }
        _ => {}
    }

    std::thread::sleep(Duration::from_millis(200));

    let prefix = format!("[host:{id}]");
    let lines = console_log::history();
    let mut stats = HostLiveStats {
        note: match server.loader.as_str() {
            "vanilla" | "fabric" | "quilt" => {
                "TPS/entity counts need Paper/Purpur (or Forge) commands".into()
            }
            _ => String::new(),
        },
        ..Default::default()
    };

    for line in lines.iter().rev().take(80) {
        let text = line.text.strip_prefix(&prefix).unwrap_or(&line.text).trim();
        if let Some(caps) = list_re().captures(text) {
            if let Some(n) = caps.get(1).and_then(|m| m.as_str().parse().ok()) {
                stats.players_online = n;
            }
            if let Some(m) = caps.get(2).and_then(|m| m.as_str().parse().ok()) {
                stats.players_max = Some(m);
            }
            if let Some(names) = caps.get(3).map(|m| m.as_str().trim()) {
                if !names.is_empty() {
                    stats.player_names = names
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
        if stats.tps.is_none() {
            if let Some(caps) = tps_re().captures(text) {
                for i in 1..=3 {
                    if let Some(v) = caps.get(i).and_then(|m| m.as_str().parse().ok()) {
                        stats.tps = Some(v);
                        break;
                    }
                }
            }
        }
        if stats.mspt.is_none() {
            if let Some(caps) = mspt_re().captures(text) {
                stats.mspt = caps.get(1).and_then(|m| m.as_str().parse().ok());
            }
        }
        if stats.entity_count.is_none() {
            if let Some(caps) = entity_re().captures(text) {
                stats.entity_count = caps.get(1).and_then(|m| m.as_str().parse().ok());
            }
        }
        if text.contains(" joined the game") {
            let name = text
                .split(" joined the game")
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() && !stats.player_names.iter().any(|n| n == &name) {
                stats.player_names.push(name);
                stats.players_online = stats.players_online.max(stats.player_names.len() as u32);
            }
        }
    }
    stats.mob_count = stats.entity_count;

    if let Some(pid) = status.pid {
        fill_resource_metrics(&mut stats, pid);
    }

    let _ = app;
    Ok(stats)
}

fn fill_resource_metrics(stats: &mut HostLiveStats, pid: u32) {
    #[cfg(windows)]
    {
        if let Some((cpu_sec, ram_bytes, sys_total, sys_avail)) = read_process_and_ram(pid) {
            stats.ram_used_mb = Some(ram_bytes as f64 / (1024.0 * 1024.0));
            stats.ram_total_mb = Some(sys_total as f64 / (1024.0 * 1024.0));
            let used = sys_total.saturating_sub(sys_avail);
            stats.ram_system_used_mb = Some(used as f64 / (1024.0 * 1024.0));

            let now = Instant::now();
            let mut guard = sampler();
            if guard.is_none() {
                *guard = Some(SamplerState {
                    proc: HashMap::new(),
                    net: None,
                });
            }
            let state = guard.as_mut().unwrap();
            if let Some(prev) = state.proc.get(&pid) {
                let dt = now.duration_since(prev.at).as_secs_f64();
                if dt > 0.05 {
                    let dcpu = (cpu_sec - prev.cpu_seconds).max(0.0);
                    // CPU seconds are total across cores; percent of one core would be (dcpu/dt)*100
                    // Report % of all logical CPUs: (dcpu/dt)/cores * 100
                    let cores = host_process::logical_cpu_count().max(1) as f64;
                    let pct = (dcpu / dt) / cores * 100.0;
                    stats.cpu_percent = Some(pct.clamp(0.0, 100.0 * cores) as f32);
                }
            }
            state.proc.insert(
                pid,
                ProcSample {
                    cpu_seconds: cpu_sec,
                    at: now,
                },
            );
        }

        if let Some((recv, sent)) = read_net_bytes() {
            let now = Instant::now();
            let mut guard = sampler();
            if guard.is_none() {
                *guard = Some(SamplerState {
                    proc: HashMap::new(),
                    net: None,
                });
            }
            let state = guard.as_mut().unwrap();
            if let Some(prev) = &state.net {
                let dt = now.duration_since(prev.at).as_secs_f64();
                if dt > 0.05 {
                    let down = (recv.saturating_sub(prev.recv)) as f64 / dt;
                    let up = (sent.saturating_sub(prev.sent)) as f64 / dt;
                    stats.net_down_bps = Some(down);
                    stats.net_up_bps = Some(up);
                }
            }
            state.net = Some(NetSample {
                recv,
                sent,
                at: now,
            });
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        if stats.note.is_empty() {
            stats.note = "CPU/RAM/WLAN meters are Windows-only in this build".into();
        }
    }
}

#[cfg(windows)]
fn read_process_and_ram(pid: u32) -> Option<(f64, u64, u64, u64)> {
    // Returns: cpuSeconds, workingSetBytes, totalPhys, availPhys
    let script = format!(
        r#"
$p = Get-Process -Id {pid} -ErrorAction SilentlyContinue
if (-not $p) {{ 'null'; exit }}
$os = Get-CimInstance Win32_OperatingSystem
$obj = [pscustomobject]@{{
  cpu = [double]$p.CPU
  ws = [uint64]$p.WorkingSet64
  total = [uint64]$os.TotalVisibleMemorySize * 1024
  avail = [uint64]$os.FreePhysicalMemory * 1024
}}
$obj | ConvertTo-Json -Compress
"#
    );
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let text = text.trim();
    if text.is_empty() || text == "null" {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let cpu = v.get("cpu")?.as_f64()?;
    let ws = v.get("ws")?.as_u64()?;
    let total = v.get("total")?.as_u64()?;
    let avail = v.get("avail")?.as_u64()?;
    Some((cpu, ws, total, avail))
}

#[cfg(windows)]
fn read_net_bytes() -> Option<(u64, u64)> {
    // Sum ReceivedBytes / SentBytes across adapters (WLAN + Ethernet)
    let script = r#"
$stats = Get-NetAdapterStatistics -ErrorAction SilentlyContinue
if (-not $stats) { 'null'; exit }
$recv = 0UL; $sent = 0UL
foreach ($s in $stats) {
  $recv += [uint64]$s.ReceivedBytes
  $sent += [uint64]$s.SentBytes
}
(@{ recv = $recv; sent = $sent }) | ConvertTo-Json -Compress
"#;
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let text = text.trim();
    if text.is_empty() || text == "null" {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let recv = v.get("recv")?.as_u64()?;
    let sent = v.get("sent")?.as_u64()?;
    Some((recv, sent))
}
