use serde::Serialize;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::num::NonZeroU16;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAdapterInfo {
    pub name: String,
    pub ipv4: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapAttempt {
    pub method: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    pub lan_ip: Option<String>,
    pub port: u16,
    pub upnp_status: String,
    pub upnp_message: String,
    pub firewall_hint: String,
    pub firewall_rule_added: bool,
    #[serde(default)]
    pub adapters: Vec<NetworkAdapterInfo>,
    #[serde(default)]
    pub join_address: Option<String>,
    #[serde(default)]
    pub wlan_hint: String,
    /// Public / WAN IPv4 (from UPnP gateway or STUN-like HTTP lookup).
    #[serde(default)]
    pub public_ip: Option<String>,
    /// `publicIp:port` for friends over the internet when mapping succeeded.
    #[serde(default)]
    pub wan_join_address: Option<String>,
    #[serde(default)]
    pub internet_hint: String,
    /// `upnp` | `natpmp` | `pcp` | null
    #[serde(default)]
    pub map_method: Option<String>,
    #[serde(default)]
    pub map_attempts: Vec<MapAttempt>,
    #[serde(default)]
    pub needs_manual: bool,
    #[serde(default)]
    pub manual_hint: String,
    /// Always false — relay requires an external server (not implemented).
    #[serde(default)]
    pub relay_available: bool,
    #[serde(default)]
    pub relay_hint: String,
}

#[derive(Debug, Clone)]
struct ActiveMapping {
    method: String,
    port: u16,
    gateway: Option<IpAddr>,
    local: Option<IpAddr>,
    external_ip: Option<String>,
}

static ACTIVE: Mutex<Option<HashMap<u16, ActiveMapping>>> = Mutex::new(None);

fn active_map() -> std::sync::MutexGuard<'static, Option<HashMap<u16, ActiveMapping>>> {
    ACTIVE.lock().unwrap_or_else(|e| e.into_inner())
}

fn with_active<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<u16, ActiveMapping>) -> R,
{
    let mut g = active_map();
    if g.is_none() {
        *g = Some(HashMap::new());
    }
    f(g.as_mut().unwrap())
}

/// Best-effort LAN IPv4 via UDP connect trick (no packets sent meaningfully).
pub fn lan_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}

static ADAPTER_CACHE: Mutex<Option<(Instant, Vec<NetworkAdapterInfo>)>> = Mutex::new(None);
static PUBLIC_IP_CACHE: Mutex<Option<(Instant, String)>> = Mutex::new(None);

fn decode_console_bytes(bytes: &[u8]) -> String {
    // Prefer UTF-8; Chinese Windows PowerShell often emits GBK/CP936.
    if let Ok(s) = std::str::from_utf8(bytes) {
        if !s.contains('\u{FFFD}') && !s.bytes().any(|b| b >= 0x80) {
            return s.to_string();
        }
        // Valid UTF-8 with high bytes — keep it
        if !s.contains('\u{FFFD}') {
            return s.to_string();
        }
    }
    // Best-effort GBK (code page 936) without an extra crate:
    // interpret as Windows ANSI by asking PowerShell to re-encode, or use lossy UTF-8.
    // Prefer lossy only as last resort — try GBK via simple conversion when available.
    #[cfg(windows)]
    {
        // MultiByteToWideChar CP_ACP / 936
        const CP_GBK: u32 = 936;
        #[link(name = "kernel32")]
        extern "system" {
            fn MultiByteToWideChar(
                code_page: u32,
                flags: u32,
                bytes: *const u8,
                byte_len: i32,
                wide: *mut u16,
                wide_len: i32,
            ) -> i32;
        }
        unsafe {
            let need = MultiByteToWideChar(
                CP_GBK,
                0,
                bytes.as_ptr(),
                bytes.len() as i32,
                std::ptr::null_mut(),
                0,
            );
            if need > 0 {
                let mut wide = vec![0u16; need as usize];
                let wrote = MultiByteToWideChar(
                    CP_GBK,
                    0,
                    bytes.as_ptr(),
                    bytes.len() as i32,
                    wide.as_mut_ptr(),
                    need,
                );
                if wrote > 0 {
                    return String::from_utf16_lossy(&wide[..wrote as usize]);
                }
            }
        }
    }
    String::from_utf8_lossy(bytes).into_owned()
}

fn is_apipa(ip: &str) -> bool {
    ip.starts_with("169.254.")
}

fn is_private_lan(ip: &str) -> bool {
    ip.starts_with("10.")
        || ip.starts_with("192.168.")
        || ip
            .parse::<Ipv4Addr>()
            .ok()
            .map(|a| {
                let o = a.octets();
                o[0] == 172 && (16..=31).contains(&o[1])
            })
            .unwrap_or(false)
}

fn adapter_noise(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("vethernet")
        || n.contains("wsl")
        || n.contains("hyper-v")
        || n.contains("virtualbox")
        || n.contains("vmware")
        || n.contains("docker")
        || n.contains("loopback")
        || n.contains("bluetooth")
        || n.contains("蓝牙")
}

pub fn list_adapters() -> Vec<NetworkAdapterInfo> {
    // PowerShell is expensive — cache briefly so Host UI polling doesn't stall the launcher
    if let Ok(guard) = ADAPTER_CACHE.lock() {
        if let Some((at, cached)) = guard.as_ref() {
            if at.elapsed() < Duration::from_secs(45) {
                return cached.clone();
            }
        }
    }

    let adapters = list_adapters_uncached();
    if let Ok(mut guard) = ADAPTER_CACHE.lock() {
        *guard = Some((Instant::now(), adapters.clone()));
    }
    adapters
}

fn list_adapters_uncached() -> Vec<NetworkAdapterInfo> {
    #[cfg(windows)]
    {
        let script = r#"
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false
$OutputEncoding = [Console]::OutputEncoding
Get-NetIPAddress -AddressFamily IPv4 |
  Where-Object { $_.IPAddress -notlike '127.*' } |
  ForEach-Object { "{0}`t{1}" -f $_.InterfaceAlias, $_.IPAddress }
"#;
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output();
        if let Ok(out) = output {
            let text = decode_console_bytes(&out.stdout);
            let mut adapters = Vec::new();
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Some((name, ip)) = line.split_once('\t').or_else(|| line.split_once('|')) {
                    adapters.push(NetworkAdapterInfo {
                        name: name.trim().to_string(),
                        ipv4: ip.trim().to_string(),
                    });
                }
            }
            // Prefer real LAN adapters first; keep virtual/APIPA at the end for debugging
            adapters.sort_by_key(|a| {
                let noise = adapter_noise(&a.name);
                let apipa = is_apipa(&a.ipv4);
                let private = is_private_lan(&a.ipv4);
                (noise as u8, apipa as u8, !private as u8)
            });
            if !adapters.is_empty() {
                return adapters;
            }
        }
    }
    lan_ip()
        .map(|ip| {
            vec![NetworkAdapterInfo {
                name: "LAN".into(),
                ipv4: ip,
            }]
        })
        .unwrap_or_default()
}

/// Public WAN IPv4: UPnP gateway first, then HTTP fallback (cached).
pub fn public_ip(prefer: Option<&str>) -> Option<String> {
    if let Some(p) = prefer {
        if !p.is_empty() && p != "0.0.0.0" {
            return Some(p.to_string());
        }
    }
    if let Ok(guard) = PUBLIC_IP_CACHE.lock() {
        if let Some((at, ip)) = guard.as_ref() {
            if at.elapsed() < Duration::from_secs(300) {
                return Some(ip.clone());
            }
        }
    }
    // Try UPnP GetExternalIPAddress
    if let Ok(gateway) = igd::search_gateway(Default::default()) {
        if let Ok(ip) = gateway.get_external_ip() {
            let s = ip.to_string();
            if let Ok(mut g) = PUBLIC_IP_CACHE.lock() {
                *g = Some((Instant::now(), s.clone()));
            }
            return Some(s);
        }
    }
    // HTTP fallback
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .ok()?;
    for url in [
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://icanhazip.com",
    ] {
        if let Ok(resp) = client.get(url).send() {
            if let Ok(text) = resp.text() {
                let ip = text.trim().to_string();
                if ip.parse::<Ipv4Addr>().is_ok() {
                    if let Ok(mut g) = PUBLIC_IP_CACHE.lock() {
                        *g = Some((Instant::now(), ip.clone()));
                    }
                    return Some(ip);
                }
            }
        }
    }
    None
}

/// Default gateway IPv4 (Windows route table / UDP heuristic fallback).
pub fn default_gateway() -> Option<IpAddr> {
    #[cfg(windows)]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue | Sort-Object RouteMetric | Select-Object -First 1).NextHop",
            ])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let hop = text.trim();
        if let Ok(ip) = hop.parse::<Ipv4Addr>() {
            if !ip.is_unspecified() {
                return Some(IpAddr::V4(ip));
            }
        }
    }
    // Fallback: assume .1 on LAN subnet
    let lan = lan_ip()?.parse::<Ipv4Addr>().ok()?;
    let octets = lan.octets();
    Some(IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], 1)))
}

fn manual_hint(port: u16, lan: Option<&str>) -> String {
    format!(
        "Automatic port mapping failed. On your router, forward external TCP {port} → {lan}:{port} (or your PC's LAN IP). Then friends can join via your public IP:{port}.",
        lan = lan.unwrap_or("LAN-IP")
    )
}

pub fn network_info(port: u16, mapped: bool) -> NetworkInfo {
    network_info_full(port, mapped, None, Vec::new(), false)
}

pub fn network_info_full(
    port: u16,
    mapped: bool,
    map_method: Option<String>,
    attempts: Vec<MapAttempt>,
    needs_manual: bool,
) -> NetworkInfo {
    let adapters = list_adapters();
    let lan = lan_ip().or_else(|| {
        adapters
            .iter()
            .find(|a| is_private_lan(&a.ipv4) && !adapter_noise(&a.name) && !is_apipa(&a.ipv4))
            .or_else(|| adapters.iter().find(|a| is_private_lan(&a.ipv4)))
            .map(|a| a.ipv4.clone())
    });
    let method = map_method
        .clone()
        .or_else(|| with_active(|m| m.get(&port).map(|a| a.method.clone())));
    let stored_ext = with_active(|m| m.get(&port).and_then(|a| a.external_ip.clone()));
    let actually_mapped = mapped || method.is_some();
    let public = if actually_mapped || needs_manual {
        public_ip(stored_ext.as_deref())
    } else {
        stored_ext.or_else(|| public_ip(None))
    };
    let wan_join = public.as_ref().map(|ip| format!("{ip}:{port}"));
    let (upnp_status, upnp_message) = if actually_mapped {
        let m = method.as_deref().unwrap_or("upnp");
        let wan = wan_join
            .as_deref()
            .map(|w| format!(" Friends over the internet can join: {w}"))
            .unwrap_or_else(|| {
                " Could not detect public IP — check whatismyipaddress.com and use that IP:port."
                    .into()
            });
        (
            "mapped".into(),
            format!("TCP {port} mapped via {m}.{wan}"),
        )
    } else if needs_manual {
        (
            "failed".into(),
            "UPnP, NAT-PMP, and PCP all failed.".into(),
        )
    } else {
        (
            "unmapped".into(),
            "No automatic port mapping active.".into(),
        )
    };
    let join_address = lan.as_ref().map(|ip| format!("{ip}:{port}"));
    let internet_hint = if actually_mapped {
        if let Some(ref w) = wan_join {
            format!(
                "Internet: give friends {w} (Direct Connection / Add Server). Same Wi‑Fi: use {lan}:{port}.",
                lan = lan.as_deref().unwrap_or("LAN-IP")
            )
        } else {
            format!(
                "Port is mapped, but public IP is unknown. Look up your WAN IP and join as WAN-IP:{port}. LAN: {lan}:{port}.",
                lan = lan.as_deref().unwrap_or("LAN-IP")
            )
        }
    } else if needs_manual {
        format!(
            "No auto map. Forward TCP {port} on your router to {lan}, then friends join PUBLIC-IP:{port}.",
            lan = lan.as_deref().unwrap_or("LAN-IP")
        )
    } else {
        "Map the port (UPnP) so friends can join from the internet.".into()
    };
    NetworkInfo {
        lan_ip: lan.clone(),
        port,
        upnp_status,
        upnp_message,
        firewall_hint: format!(
            "Allow inbound TCP {port} in Windows Firewall. LAN address: {}",
            lan.as_deref().unwrap_or("unknown")
        ),
        firewall_rule_added: false,
        adapters,
        join_address,
        wlan_hint: "Same Wi‑Fi/Ethernet: use the LAN join address above.".into(),
        public_ip: public,
        wan_join_address: wan_join,
        internet_hint,
        map_method: method,
        map_attempts: attempts,
        needs_manual,
        manual_hint: if needs_manual {
            manual_hint(port, lan.as_deref())
        } else {
            String::new()
        },
        relay_available: false,
        relay_hint: "Relay requires an external relay server and is not available in this build. Use LAN play or manual port forwarding instead.".into(),
    }
}

fn map_upnp(port: u16) -> Result<Option<String>, String> {
    let gateway = igd::search_gateway(Default::default())
        .map_err(|e| format!("No UPnP gateway: {e}"))?;
    let local = lan_ip().ok_or_else(|| "Could not determine LAN IP".to_string())?;
    let ipv4: Ipv4Addr = local
        .parse()
        .map_err(|e| format!("LAN IP is not IPv4: {e}"))?;
    let local_addr = std::net::SocketAddrV4::new(ipv4, port);
    gateway
        .add_port(
            igd::PortMappingProtocol::TCP,
            port,
            local_addr,
            0,
            "Northstar Dedicated Server",
        )
        .map_err(|e| format!("UPnP AddPortMapping failed: {e}"))?;
    let external = gateway.get_external_ip().ok().map(|ip| ip.to_string());
    if let Some(ref ip) = external {
        if let Ok(mut g) = PUBLIC_IP_CACHE.lock() {
            *g = Some((Instant::now(), ip.clone()));
        }
    }
    Ok(external)
}

fn map_nat_pmp(port: u16) -> Result<(IpAddr, IpAddr), String> {
    let gateway = default_gateway().ok_or_else(|| "Could not determine default gateway".to_string())?;
    let local: IpAddr = lan_ip()
        .ok_or_else(|| "Could not determine LAN IP".to_string())?
        .parse()
        .map_err(|e| format!("Bad LAN IP: {e}"))?;
    let nz = NonZeroU16::new(port).ok_or_else(|| "Port cannot be 0".to_string())?;
    let opts = crab_nat::PortMappingOptions {
        external_port: Some(nz),
        lifetime_seconds: Some(7200),
        timeout_config: None,
    };
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        crab_nat::natpmp::port_mapping(gateway.into(), crab_nat::InternetProtocol::Tcp, nz, opts)
            .await
            .map_err(|e| format!("NAT-PMP failed: {e:?}"))
    })?;
    Ok((gateway, local))
}

fn map_pcp(port: u16) -> Result<(IpAddr, IpAddr), String> {
    let gateway = default_gateway().ok_or_else(|| "Could not determine default gateway".to_string())?;
    let local: IpAddr = lan_ip()
        .ok_or_else(|| "Could not determine LAN IP".to_string())?
        .parse()
        .map_err(|e| format!("Bad LAN IP: {e}"))?;
    let nz = NonZeroU16::new(port).ok_or_else(|| "Port cannot be 0".to_string())?;
    let opts = crab_nat::PortMappingOptions {
        external_port: Some(nz),
        lifetime_seconds: Some(7200),
        timeout_config: None,
    };
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        let base = crab_nat::pcp::BaseMapRequest::new(
            gateway.into(),
            local,
            crab_nat::InternetProtocol::Tcp,
            nz,
        );
        crab_nat::pcp::port_mapping(base, None, None, opts)
            .await
            .map_err(|e| format!("PCP failed: {e:?}"))
    })?;
    Ok((gateway, local))
}

/// Cascade: UPnP → NAT-PMP → PCP. On total failure returns NetworkInfo with needs_manual.
pub fn map_port_cascade(port: u16) -> NetworkInfo {
    let mut attempts = Vec::new();

    // 1. UPnP
    match map_upnp(port) {
        Ok(external_ip) => {
            let msg = external_ip
                .as_ref()
                .map(|ip| format!("Mapped via UPnP — internet join {ip}:{port}"))
                .unwrap_or_else(|| "Mapped via UPnP".into());
            attempts.push(MapAttempt {
                method: "upnp".into(),
                ok: true,
                message: msg,
            });
            with_active(|m| {
                m.insert(
                    port,
                    ActiveMapping {
                        method: "upnp".into(),
                        port,
                        gateway: None,
                        local: None,
                        external_ip,
                    },
                );
            });
            return network_info_full(port, true, Some("upnp".into()), attempts, false);
        }
        Err(e) => attempts.push(MapAttempt {
            method: "upnp".into(),
            ok: false,
            message: e,
        }),
    }

    // 2. NAT-PMP
    match map_nat_pmp(port) {
        Ok((gateway, local)) => {
            let external_ip = public_ip(None);
            let msg = external_ip
                .as_ref()
                .map(|ip| format!("Mapped via NAT-PMP — internet join {ip}:{port}"))
                .unwrap_or_else(|| "Mapped via NAT-PMP".into());
            attempts.push(MapAttempt {
                method: "natpmp".into(),
                ok: true,
                message: msg,
            });
            with_active(|m| {
                m.insert(
                    port,
                    ActiveMapping {
                        method: "natpmp".into(),
                        port,
                        gateway: Some(gateway),
                        local: Some(local),
                        external_ip,
                    },
                );
            });
            return network_info_full(port, true, Some("natpmp".into()), attempts, false);
        }
        Err(e) => attempts.push(MapAttempt {
            method: "natpmp".into(),
            ok: false,
            message: e,
        }),
    }

    // 3. PCP
    match map_pcp(port) {
        Ok((gateway, local)) => {
            let external_ip = public_ip(None);
            let msg = external_ip
                .as_ref()
                .map(|ip| format!("Mapped via PCP — internet join {ip}:{port}"))
                .unwrap_or_else(|| "Mapped via PCP".into());
            attempts.push(MapAttempt {
                method: "pcp".into(),
                ok: true,
                message: msg,
            });
            with_active(|m| {
                m.insert(
                    port,
                    ActiveMapping {
                        method: "pcp".into(),
                        port,
                        gateway: Some(gateway),
                        local: Some(local),
                        external_ip,
                    },
                );
            });
            return network_info_full(port, true, Some("pcp".into()), attempts, false);
        }
        Err(e) => attempts.push(MapAttempt {
            method: "pcp".into(),
            ok: false,
            message: e,
        }),
    }

    network_info_full(port, false, None, attempts, true)
}

/// Legacy single-method UPnP (used by older call sites). Prefer [`map_port_cascade`].
pub fn map_port(port: u16) -> Result<(), String> {
    let info = map_port_cascade(port);
    if info.map_method.is_some() {
        Ok(())
    } else {
        Err(info
            .map_attempts
            .last()
            .map(|a| a.message.clone())
            .unwrap_or_else(|| "Port mapping failed".into()))
    }
}

pub fn unmap_port(port: u16) -> Result<(), String> {
    let session = with_active(|m| m.remove(&port));
    let method = session
        .as_ref()
        .map(|s| s.method.as_str())
        .unwrap_or("upnp");

    match method {
        "natpmp" => {
            if let Some(s) = &session {
                let _ = unmap_nat_pmp(s);
            }
        }
        "pcp" => {
            if let Some(s) = &session {
                let _ = unmap_pcp(s);
            }
        }
        _ => {
            let gateway = igd::search_gateway(Default::default())
                .map_err(|e| format!("No UPnP gateway: {e}"))?;
            gateway
                .remove_port(igd::PortMappingProtocol::TCP, port)
                .map_err(|e| format!("UPnP RemovePortMapping failed: {e}"))?;
        }
    }
    Ok(())
}

fn unmap_nat_pmp(session: &ActiveMapping) -> Result<(), String> {
    let gateway = session
        .gateway
        .or_else(default_gateway)
        .ok_or_else(|| "No gateway for NAT-PMP unmap".to_string())?;
    let nz = NonZeroU16::new(session.port).ok_or_else(|| "Port cannot be 0".to_string())?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let _ = rt.block_on(async {
        crab_nat::natpmp::try_drop_mapping(
            gateway.into(),
            crab_nat::InternetProtocol::Tcp,
            Some(nz),
            None,
        )
        .await
    });
    Ok(())
}

fn unmap_pcp(session: &ActiveMapping) -> Result<(), String> {
    let gateway = session
        .gateway
        .or_else(default_gateway)
        .ok_or_else(|| "No gateway for PCP unmap".to_string())?;
    let local = session
        .local
        .or_else(|| lan_ip().and_then(|s| s.parse().ok()))
        .ok_or_else(|| "No local IP for PCP unmap".to_string())?;
    let nz = NonZeroU16::new(session.port).ok_or_else(|| "Port cannot be 0".to_string())?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let _ = rt.block_on(async {
        // Lifetime 0 deletes the mapping (nonce unknown after restart — best-effort)
        let base = crab_nat::pcp::BaseMapRequest::new(
            gateway.into(),
            local,
            crab_nat::InternetProtocol::Tcp,
            nz,
        );
        let opts = crab_nat::PortMappingOptions {
            external_port: Some(nz),
            lifetime_seconds: Some(0),
            timeout_config: None,
        };
        crab_nat::pcp::port_mapping(base, None, None, opts).await
    });
    Ok(())
}

pub fn try_add_firewall_rule(port: u16) -> Result<(), String> {
    #[cfg(windows)]
    {
        let name = format!("Northstar Dedicated TCP {port}");
        let status = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={name}"),
                "dir=in",
                "action=allow",
                "protocol=TCP",
                &format!("localport={port}"),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "netsh failed (exit {}). Run Northstar as admin or add the rule manually.",
                status.code().unwrap_or(-1)
            ))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = port;
        Err("Firewall helper is only implemented on Windows".into())
    }
}

pub fn mapping_method_for(port: u16) -> Option<String> {
    with_active(|m| m.get(&port).map(|a| a.method.clone()))
}
