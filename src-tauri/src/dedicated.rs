use crate::download::{download_file, rewrite_url};
use crate::paths::{dedicated_dir, dedicated_root, dedicated_runtime, meta_dir};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostServer {
    pub id: String,
    pub name: String,
    pub game_version: String,
    /// `vanilla` | `fabric` | `forge` | `neoforge` | `quilt` | `paper` | `purpur`
    pub loader: String,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default = "default_memory")]
    pub memory_mb: u32,
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
    /// Optional Windows CPU affinity bitmask (bit 0 = logical CPU 0).
    #[serde(default)]
    pub cpu_affinity_mask: Option<u64>,
    pub created_at: String,
    #[serde(default)]
    pub last_started: Option<String>,
    /// OS PID of the last started Java process (survives app restarts for status/reattach).
    #[serde(default)]
    pub running_pid: Option<u32>,
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub eula_accepted: bool,
}

fn default_memory() -> u32 {
    2048
}
fn default_port() -> u16 {
    25565
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedProperties {
    pub motd: String,
    pub max_players: u32,
    pub difficulty: String,
    pub gamemode: String,
    pub online_mode: bool,
    pub white_list: bool,
    pub spawn_monsters: bool,
    pub view_distance: u32,
    pub server_port: u16,
    pub level_name: String,
}

impl Default for DedicatedProperties {
    fn default() -> Self {
        Self {
            motd: "EUML Dedicated Server".into(),
            max_players: 20,
            difficulty: "easy".into(),
            gamemode: "survival".into(),
            online_mode: true,
            white_list: false,
            spawn_monsters: true,
            view_distance: 10,
            server_port: 25565,
            level_name: "world".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLists {
    pub whitelist: Vec<PlayerListEntry>,
    pub ops: Vec<OpListEntry>,
    pub banned_players: Vec<BanListEntry>,
    pub banned_ips: Vec<BanIpEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerListEntry {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpListEntry {
    pub uuid: String,
    pub name: String,
    #[serde(default = "default_op_level")]
    pub level: u32,
    #[serde(default = "default_true")]
    pub bypasses_player_limit: bool,
}

fn default_op_level() -> u32 {
    4
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BanListEntry {
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub expires: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BanIpEntry {
    pub ip: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub expires: String,
    #[serde(default)]
    pub reason: String,
}

const NOT_FOUND: &str = "Server not found (folder may have been deleted)";

fn host_json_path(id: &str) -> Result<PathBuf, String> {
    Ok(dedicated_dir(id)?.join("host.json"))
}

pub fn get_dedicated(id: &str) -> Result<HostServer, String> {
    let path = host_json_path(id)?;
    if !path.exists() {
        return Err(NOT_FOUND.into());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("{NOT_FOUND}: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Corrupt host.json: {e}"))
}

pub fn save_dedicated(server: &HostServer) -> Result<(), String> {
    let dir = dedicated_dir(&server.id)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("host.json");
    let raw = serde_json::to_string_pretty(server).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

pub fn list_dedicated() -> Result<Vec<HostServer>, String> {
    let root = dedicated_root()?;
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(&root) else {
        return Ok(out);
    };
    for entry in entries.flatten() {
        let path = entry.path().join("host.json");
        if !path.is_file() {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(s) = serde_json::from_str::<HostServer>(&raw) {
                out.push(s);
            }
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

pub fn create_dedicated(
    name: String,
    game_version: String,
    loader: String,
    memory_mb: Option<u32>,
    port: Option<u16>,
) -> Result<HostServer, String> {
    let loader = match loader.to_ascii_lowercase().as_str() {
        "fabric" => "fabric",
        "quilt" => "quilt",
        "forge" => "forge",
        "neoforge" => "neoforge",
        "paper" => "paper",
        "purpur" => "purpur",
        _ => "vanilla",
    }
    .to_string();
    let game_version = crate::models::normalize_game_version(&game_version);
    if game_version.is_empty() {
        return Err("Invalid game version".into());
    }
    let id = Uuid::new_v4().to_string();
    let port = port.unwrap_or(25565);
    let server = HostServer {
        id: id.clone(),
        name: if name.trim().is_empty() {
            format!("{game_version}-{loader}")
        } else {
            name.trim().to_string()
        },
        game_version,
        loader,
        loader_version: None,
        memory_mb: memory_mb.unwrap_or(2048).clamp(512, 65536),
        java_path: None,
        port,
        cpu_affinity_mask: None,
        created_at: Utc::now().to_rfc3339(),
        last_started: None,
        running_pid: None,
        installed: false,
        eula_accepted: false,
    };
    ensure_runtime_dirs(&id)?;
    // Persist host.json first — property/list writers require it to exist.
    save_dedicated(&server)?;
    write_default_properties(&id, port)?;
    write_empty_player_lists(&id)?;
    write_eula(&id, false)?;
    Ok(server)
}

pub fn update_dedicated(server: HostServer) -> Result<HostServer, String> {
    let mut existing = get_dedicated(&server.id)?;
    existing.name = server.name;
    existing.memory_mb = server.memory_mb.clamp(512, 65536);
    existing.java_path = server.java_path;
    existing.cpu_affinity_mask = server.cpu_affinity_mask;
    // running_pid is owned by the process manager — don't clobber from UI saves
    if server.port > 0 {
        existing.port = server.port;
        // keep properties port in sync when idle
        if let Ok(mut props) = get_dedicated_properties(&existing.id) {
            props.server_port = existing.port;
            let _ = set_dedicated_properties(&existing.id, props);
        }
    }
    save_dedicated(&existing)?;
    Ok(existing)
}

pub fn set_running_pid(id: &str, pid: Option<u32>) -> Result<(), String> {
    let mut server = get_dedicated(id)?;
    server.running_pid = pid;
    save_dedicated(&server)
}

pub fn delete_dedicated(id: String) -> Result<(), String> {
    let dir = dedicated_dir(&id)?;
    if !dir.exists() {
        return Ok(()); // already gone — not an error
    }
    // Only delete under dedicated root
    let root = dedicated_root()?;
    let canonical = dir.canonicalize().unwrap_or(dir.clone());
    let root_c = root.canonicalize().unwrap_or(root);
    if !canonical.starts_with(&root_c) {
        return Err("Refusing to delete path outside dedicated root".into());
    }
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn open_dedicated_folder(id: String) -> Result<(), String> {
    let dir = dedicated_dir(&id)?;
    if !dir.exists() {
        return Err(NOT_FOUND.into());
    }
    open::that(dir).map_err(|e| e.to_string())
}

fn ensure_runtime_dirs(id: &str) -> Result<(), String> {
    let runtime = dedicated_runtime(id)?;
    for sub in ["logs", "mods", "config", "world"] {
        fs::create_dir_all(runtime.join(sub)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn write_eula(id: &str, accepted: bool) -> Result<(), String> {
    let path = dedicated_runtime(id)?.join("eula.txt");
    let body = format!(
        "# Generated by EUML\neula={}\n",
        if accepted { "true" } else { "false" }
    );
    fs::write(path, body).map_err(|e| e.to_string())
}

pub fn accept_dedicated_eula(id: String) -> Result<HostServer, String> {
    let mut server = get_dedicated(&id)?;
    write_eula(&id, true)?;
    server.eula_accepted = true;
    save_dedicated(&server)?;
    Ok(server)
}

fn write_default_properties(id: &str, port: u16) -> Result<(), String> {
    let mut props = DedicatedProperties::default();
    props.server_port = port;
    set_dedicated_properties(id, props)
}

fn write_empty_player_lists(id: &str) -> Result<(), String> {
    set_dedicated_player_lists(id, PlayerLists::default())
}

fn properties_path(id: &str) -> Result<PathBuf, String> {
    Ok(dedicated_runtime(id)?.join("server.properties"))
}

pub fn get_dedicated_properties(id: &str) -> Result<DedicatedProperties, String> {
    let _ = get_dedicated(id)?;
    let path = properties_path(id)?;
    if !path.exists() {
        return Ok(DedicatedProperties::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let map = parse_properties(&raw);
    Ok(DedicatedProperties {
        motd: map
            .get("motd")
            .cloned()
            .unwrap_or_else(|| "EUML Dedicated Server".into()),
        max_players: map
            .get("max-players")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        difficulty: map
            .get("difficulty")
            .cloned()
            .unwrap_or_else(|| "easy".into()),
        gamemode: map
            .get("gamemode")
            .cloned()
            .unwrap_or_else(|| "survival".into()),
        online_mode: map
            .get("online-mode")
            .map(|v| v == "true")
            .unwrap_or(true),
        white_list: map
            .get("white-list")
            .or_else(|| map.get("whitelist"))
            .map(|v| v == "true")
            .unwrap_or(false),
        spawn_monsters: map
            .get("spawn-monsters")
            .map(|v| v == "true")
            .unwrap_or(true),
        view_distance: map
            .get("view-distance")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
        server_port: map
            .get("server-port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(25565),
        level_name: map
            .get("level-name")
            .cloned()
            .unwrap_or_else(|| "world".into()),
    })
}

pub fn set_dedicated_properties(id: &str, props: DedicatedProperties) -> Result<(), String> {
    let _ = get_dedicated(id)?;
    ensure_runtime_dirs(id)?;
    let path = properties_path(id)?;
    let mut map = if path.exists() {
        parse_properties(&fs::read_to_string(&path).unwrap_or_default())
    } else {
        HashMap::new()
    };
    map.insert("motd".into(), props.motd);
    map.insert("max-players".into(), props.max_players.to_string());
    map.insert("difficulty".into(), props.difficulty);
    map.insert("gamemode".into(), props.gamemode);
    map.insert(
        "online-mode".into(),
        if props.online_mode { "true" } else { "false" }.into(),
    );
    map.insert(
        "white-list".into(),
        if props.white_list { "true" } else { "false" }.into(),
    );
    map.insert(
        "spawn-monsters".into(),
        if props.spawn_monsters {
            "true"
        } else {
            "false"
        }
        .into(),
    );
    map.insert("view-distance".into(), props.view_distance.to_string());
    map.insert("server-port".into(), props.server_port.to_string());
    map.insert("level-name".into(), props.level_name);
    let body = serialize_properties(&map);
    fs::write(path, body).map_err(|e| e.to_string())?;

    // keep host.json port synced
    if let Ok(mut s) = get_dedicated(id) {
        s.port = props.server_port;
        let _ = save_dedicated(&s);
    }
    Ok(())
}

fn parse_properties(raw: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

fn serialize_properties(map: &HashMap<String, String>) -> String {
    let mut keys: Vec<_> = map.keys().cloned().collect();
    keys.sort();
    let mut out = String::from("# Minecraft server properties — managed by EUML\n");
    for k in keys {
        if let Some(v) = map.get(&k) {
            out.push_str(&format!("{k}={v}\n"));
        }
    }
    out
}

fn list_file(id: &str, name: &str) -> Result<PathBuf, String> {
    Ok(dedicated_runtime(id)?.join(name))
}

pub fn get_dedicated_player_lists(id: &str) -> Result<PlayerLists, String> {
    let _ = get_dedicated(id)?;
    Ok(PlayerLists {
        whitelist: read_json_list(&list_file(id, "whitelist.json")?)?,
        ops: read_json_list(&list_file(id, "ops.json")?)?,
        banned_players: read_json_list(&list_file(id, "banned-players.json")?)?,
        banned_ips: read_json_list(&list_file(id, "banned-ips.json")?)?,
    })
}

pub fn set_dedicated_player_lists(id: &str, lists: PlayerLists) -> Result<(), String> {
    let _ = get_dedicated(id)?;
    ensure_runtime_dirs(id)?;
    write_json_list(&list_file(id, "whitelist.json")?, &lists.whitelist)?;
    write_json_list(&list_file(id, "ops.json")?, &lists.ops)?;
    write_json_list(&list_file(id, "banned-players.json")?, &lists.banned_players)?;
    write_json_list(&list_file(id, "banned-ips.json")?, &lists.banned_ips)?;
    Ok(())
}

fn read_json_list<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn write_json_list<T: Serialize>(path: &Path, list: &[T]) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(list).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent("EUML/0.1.0")
        .build()
        .map_err(|e| e.to_string())
}

fn version_meta_url(game_version: &str) -> Result<String, String> {
    let client = http_client()?;
    let manifest: Value = client
        .get(rewrite_url(
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
        ))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    let versions = manifest
        .get("versions")
        .and_then(|v| v.as_array())
        .ok_or("Bad version manifest")?;
    for v in versions {
        if v.get("id").and_then(|i| i.as_str()) == Some(game_version) {
            return v
                .get("url")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| "Missing version URL".into());
        }
    }
    Err(format!("Version {game_version} not found"))
}

/// Install vanilla / fabric / quilt / forge / neoforge / paper / purpur server files into runtime/.
pub fn install_dedicated(id: String) -> Result<HostServer, String> {
    let mut server = get_dedicated(&id)?;
    ensure_runtime_dirs(&id)?;
    let runtime = dedicated_runtime(&id)?;

    match server.loader.as_str() {
        "paper" => install_paper_server(&mut server, &runtime)?,
        "purpur" => install_purpur_server(&mut server, &runtime)?,
        "forge" => {
            ensure_vanilla_server_jar(&server, &runtime)?;
            install_forge_server(&mut server, &runtime)?;
        }
        "neoforge" => {
            ensure_vanilla_server_jar(&server, &runtime)?;
            install_neoforge_server(&mut server, &runtime)?;
        }
        "fabric" => {
            ensure_vanilla_server_jar(&server, &runtime)?;
            install_fabric_server(&mut server, &runtime)?;
        }
        "quilt" => {
            ensure_vanilla_server_jar(&server, &runtime)?;
            install_quilt_server(&mut server, &runtime)?;
        }
        _ => {
            ensure_vanilla_server_jar(&server, &runtime)?;
            server.loader_version = None;
            write_launch_marker(
                &runtime,
                json!({ "kind": "jar", "jar": "server.jar" }),
            )?;
        }
    }

    if !properties_path(&id)?.exists() {
        write_default_properties(&id, server.port)?;
    }
    if !runtime.join("eula.txt").exists() {
        write_eula(&id, server.eula_accepted)?;
    }
    write_empty_player_lists_if_missing(&id)?;

    server.installed = true;
    save_dedicated(&server)?;
    Ok(server)
}

fn ensure_vanilla_server_jar(server: &HostServer, runtime: &Path) -> Result<(), String> {
    let meta = meta_dir()?;
    let versions_dir = meta.join("versions").join(&server.game_version);
    fs::create_dir_all(&versions_dir).map_err(|e| e.to_string())?;
    let version_json = versions_dir.join(format!("{}.json", server.game_version));
    if !version_json.exists() {
        let url = version_meta_url(&server.game_version)?;
        download_file(&url, &version_json)?;
    }
    let version: Value =
        serde_json::from_str(&fs::read_to_string(&version_json).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    let server_dl = version
        .pointer("/downloads/server")
        .ok_or("This Minecraft version has no official server download")?;
    let url = server_dl
        .get("url")
        .and_then(|u| u.as_str())
        .ok_or("Missing server jar URL")?;
    let vanilla_jar = runtime.join("server.jar");
    download_file(url, &vanilla_jar)?;
    Ok(())
}

fn write_launch_marker(runtime: &Path, value: Value) -> Result<(), String> {
    fs::write(
        runtime.join("euml-launch.json"),
        serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn write_empty_player_lists_if_missing(id: &str) -> Result<(), String> {
    for name in [
        "whitelist.json",
        "ops.json",
        "banned-players.json",
        "banned-ips.json",
    ] {
        let p = list_file(id, name)?;
        if !p.exists() {
            fs::write(&p, "[]\n").map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn install_fabric_server(server: &mut HostServer, runtime: &Path) -> Result<(), String> {
    let client = http_client()?;
    let loaders: Value = client
        .get(rewrite_url(&format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}",
            server.game_version
        )))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let loader_ver = loaders
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.pointer("/loader/version"))
        .and_then(|v| v.as_str())
        .ok_or("No Fabric loader for this Minecraft version")?
        .to_string();

    // Official Fabric server jar (includes launcher)
    let fabric_jar_url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/1.0.1/server/jar",
        server.game_version, loader_ver
    );
    let fabric_jar = runtime.join("fabric-server-launch.jar");
    match download_file(&fabric_jar_url, &fabric_jar) {
        Ok(()) => {}
        Err(_) => {
            // Fallback: server profile + libraries
            let profile: Value = client
                .get(rewrite_url(&format!(
                    "https://meta.fabricmc.net/v2/versions/loader/{}/{}/server/json",
                    server.game_version, loader_ver
                )))
                .send()
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| format!("Fabric server profile: {e}"))?
                .json()
                .map_err(|e| e.to_string())?;
            fs::write(
                runtime.join("fabric-server.json"),
                serde_json::to_string_pretty(&profile).unwrap_or_default(),
            )
            .ok();
            // Download maven libraries from profile
            download_profile_libraries(&profile)?;
            // Create a thin launcher note — still need fabric-server-launch
            // Try installer maven
            let alt = format!(
                "https://maven.fabricmc.net/net/fabricmc/fabric-installer/1.0.1/fabric-installer-1.0.1.jar"
            );
            let _ = download_file(&alt, &runtime.join("fabric-installer.jar"));
            return Err(
                "Could not download Fabric server jar. Check network / BMCLAPI and try again."
                    .into(),
            );
        }
    }

    // Fabric needs server.jar in same folder (already written as server.jar)
    server.loader_version = Some(loader_ver);
    write_launch_marker(
        runtime,
        json!({
            "kind": "jar",
            "jar": "fabric-server-launch.jar"
        }),
    )?;
    Ok(())
}

fn install_paper_server(server: &mut HostServer, runtime: &Path) -> Result<(), String> {
    let client = http_client()?;
    let version = &server.game_version;

    // Prefer Fill v3 API (current PaperMC downloads service)
    let builds_url = format!("https://fill.papermc.io/v3/projects/paper/versions/{version}/builds");
    let (build_id, download_url) = match client.get(&builds_url).send() {
        Ok(resp) if resp.status().is_success() => {
            let builds: Value = resp.json().map_err(|e| e.to_string())?;
            let builds_arr = builds
                .as_array()
                .ok_or("Unexpected Paper builds response")?;
            let pick = builds_arr
                .iter()
                .find(|b| b.get("channel").and_then(|c| c.as_str()) == Some("STABLE"))
                .or_else(|| builds_arr.first())
                .ok_or_else(|| format!("No Paper builds for {version}"))?;
            let build_id = pick
                .get("id")
                .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
                .map(|n| n.to_string())
                .or_else(|| pick.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| "latest".into());
            let download_url = pick
                .pointer("/downloads/server:default/url")
                .and_then(|u| u.as_str())
                .ok_or("Paper build missing download URL")?
                .to_string();
            (build_id, download_url)
        }
        _ => {
            // Fallback: legacy Bibliothek / api.papermc.io v2
            let builds: Value = client
                .get(format!(
                    "https://api.papermc.io/v2/projects/paper/versions/{version}"
                ))
                .send()
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| format!("Paper API: {e}"))?
                .json()
                .map_err(|e| e.to_string())?;
            let build_id = builds
                .pointer("/builds")
                .and_then(|b| b.as_array())
                .and_then(|a| a.last())
                .and_then(|n| n.as_i64().or_else(|| n.as_u64().map(|u| u as i64)))
                .ok_or_else(|| format!("No Paper builds for {version}"))?
                .to_string();
            let download_url = format!(
                "https://api.papermc.io/v2/projects/paper/versions/{version}/builds/{build_id}/downloads/paper-{version}-{build_id}.jar"
            );
            (build_id, download_url)
        }
    };

    let jar_name = format!("paper-{version}-{build_id}.jar");
    let dest = runtime.join(&jar_name);
    download_file(&download_url, &dest)?;
    let alias = runtime.join("paper.jar");
    let _ = fs::copy(&dest, &alias);

    server.loader_version = Some(build_id);
    write_launch_marker(runtime, json!({ "kind": "jar", "jar": "paper.jar" }))?;
    Ok(())
}

fn install_purpur_server(server: &mut HostServer, runtime: &Path) -> Result<(), String> {
    let client = http_client()?;
    let version = &server.game_version;
    let meta: Value = client
        .get(format!("https://api.purpurmc.org/v2/purpur/{version}"))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("Purpur API: {e}"))?
        .json()
        .map_err(|e| e.to_string())?;

    let build = meta
        .pointer("/builds/latest")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("No Purpur builds for {version}"))?
        .to_string();

    let url = format!("https://api.purpurmc.org/v2/purpur/{version}/{build}/download");
    let jar_name = format!("purpur-{version}-{build}.jar");
    let dest = runtime.join(&jar_name);
    download_file(&url, &dest)?;
    let alias = runtime.join("purpur.jar");
    let _ = fs::copy(&dest, &alias);

    server.loader_version = Some(build);
    write_launch_marker(runtime, json!({ "kind": "jar", "jar": "purpur.jar" }))?;
    Ok(())
}

fn install_forge_server(server: &mut HostServer, runtime: &Path) -> Result<(), String> {
    let client = http_client()?;
    let mc = &server.game_version;
    let promo: Value = client
        .get(rewrite_url(
            "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
        ))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("Forge promotions: {e}"))?
        .json()
        .map_err(|e| e.to_string())?;

    let forge_ver = promo
        .pointer(&format!("/promos/{mc}-recommended"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            promo
                .pointer(&format!("/promos/{mc}-latest"))
                .and_then(|v| v.as_str())
        })
        .ok_or_else(|| format!("No Forge promotion for {mc}"))?
        .to_string();

    let full = format!("{mc}-{forge_ver}");
    let installer_name = format!("forge-{full}-installer.jar");
    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{full}/{installer_name}"
    );
    let installer_path = runtime.join(&installer_name);
    download_file(&installer_url, &installer_path)?;

    let java = crate::java::resolve_java_path(mc, None)?;
    let status = std::process::Command::new(&java)
        .current_dir(runtime)
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installServer")
        .status()
        .map_err(|e| format!("Failed to run Forge installer: {e}"))?;
    if !status.success() {
        return Err(format!(
            "Forge installer failed (exit {})",
            status.code().unwrap_or(-1)
        ));
    }

    // Prefer modern args-file launch (1.17+)
    let forge_lib_dir = runtime
        .join("libraries")
        .join("net")
        .join("minecraftforge")
        .join("forge")
        .join(&full);

    #[cfg(windows)]
    let platform_name = "win_args.txt";
    #[cfg(not(windows))]
    let platform_name = "unix_args.txt";

    let platform_args = forge_lib_dir.join(platform_name);
    if platform_args.exists() {
        let jvm_args = runtime.join("user_jvm_args.txt");
        if !jvm_args.exists() {
            fs::write(
                &jvm_args,
                "# JVM args managed by EUML on start\n-Xmx2G\n",
            )
            .map_err(|e| e.to_string())?;
        }
        write_launch_marker(
            runtime,
            json!({
                "kind": "forge_args",
                "jvmArgs": "user_jvm_args.txt",
                "forgeArgs": format!("libraries/net/minecraftforge/forge/{full}/{platform_name}"),
            }),
        )?;
    } else {
        // Legacy: look for forge jar (not installer)
        let forge_jar = find_forge_launch_jar(runtime, &full)?;
        let rel = forge_jar
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("forge.jar")
            .to_string();
        write_launch_marker(runtime, json!({ "kind": "jar", "jar": rel }))?;
    }

    server.loader_version = Some(forge_ver);
    Ok(())
}

fn find_forge_launch_jar(runtime: &Path, full: &str) -> Result<PathBuf, String> {
    let candidates = [
        runtime.join(format!("forge-{full}.jar")),
        runtime.join(format!("forge-{full}-shim.jar")),
        runtime.join(format!("forge-{full}-universal.jar")),
    ];
    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }
    // Scan runtime for forge-*.jar excluding installer
    if let Ok(entries) = fs::read_dir(runtime) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("forge-") && name.ends_with(".jar") && !name.contains("installer")
            {
                return Ok(entry.path());
            }
        }
    }
    Err("Forge install finished but no launch jar/args file was found".into())
}

fn install_quilt_server(server: &mut HostServer, runtime: &Path) -> Result<(), String> {
    let client = http_client()?;
    let loaders: Value = client
        .get(rewrite_url(&format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}",
            server.game_version
        )))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let loader_ver = loaders
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.pointer("/loader/version"))
        .and_then(|v| v.as_str())
        .ok_or("No Quilt loader for this Minecraft version")?
        .to_string();

    let quilt_jar_url = format!(
        "https://meta.quiltmc.org/v3/versions/loader/{}/{}/server/jar",
        server.game_version, loader_ver
    );
    let quilt_jar = runtime.join("quilt-server-launch.jar");
    download_file(&quilt_jar_url, &quilt_jar)
        .map_err(|e| format!("Quilt server jar: {e}"))?;

    server.loader_version = Some(loader_ver);
    write_launch_marker(
        runtime,
        json!({ "kind": "jar", "jar": "quilt-server-launch.jar" }),
    )?;
    Ok(())
}

fn install_neoforge_server(server: &mut HostServer, runtime: &Path) -> Result<(), String> {
    let client = http_client()?;
    let mc = &server.game_version;
    // NeoForge versions look like 21.1.x for MC 1.21.1
    let parts: Vec<_> = mc.split('.').collect();
    let filter_prefix = if parts.len() >= 2 {
        format!("{}.{}", parts[1], parts.get(2).unwrap_or(&"0"))
    } else {
        mc.clone()
    };

    let versions: Value = client
        .get("https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge")
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("NeoForge versions API: {e}"))?
        .json()
        .map_err(|e| e.to_string())?;

    let neo_ver = versions
        .get("versions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .filter(|v| v.starts_with(&filter_prefix) || v.starts_with(&format!("{filter_prefix}.")))
        .last()
        .or_else(|| {
            versions
                .get("versions")
                .and_then(|v| v.as_array())
                .and_then(|a| a.last())
                .and_then(|v| v.as_str())
        })
        .ok_or_else(|| format!("No NeoForge version for {mc}"))?
        .to_string();

    let installer_name = format!("neoforge-{neo_ver}-installer.jar");
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{neo_ver}/{installer_name}"
    );
    let installer_path = runtime.join(&installer_name);
    download_file(&installer_url, &installer_path)?;

    let java = crate::java::resolve_java_path(mc, None)?;
    let status = std::process::Command::new(&java)
        .current_dir(runtime)
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installServer")
        .status()
        .map_err(|e| format!("Failed to run NeoForge installer: {e}"))?;
    if !status.success() {
        return Err(format!(
            "NeoForge installer failed (exit {})",
            status.code().unwrap_or(-1)
        ));
    }

    let neo_lib_dir = runtime
        .join("libraries")
        .join("net")
        .join("neoforged")
        .join("neoforge")
        .join(&neo_ver);

    #[cfg(windows)]
    let platform_name = "win_args.txt";
    #[cfg(not(windows))]
    let platform_name = "unix_args.txt";

    let platform_args = neo_lib_dir.join(platform_name);
    if platform_args.exists() {
        let jvm_args = runtime.join("user_jvm_args.txt");
        if !jvm_args.exists() {
            fs::write(&jvm_args, "# JVM args managed by EUML on start\n-Xmx2G\n")
                .map_err(|e| e.to_string())?;
        }
        write_launch_marker(
            runtime,
            json!({
                "kind": "forge_args",
                "jvmArgs": "user_jvm_args.txt",
                "forgeArgs": format!("libraries/net/neoforged/neoforge/{neo_ver}/{platform_name}"),
            }),
        )?;
    } else {
        // Fallback: find neoforge jar
        let neo_jar = runtime.join(format!("neoforge-{neo_ver}.jar"));
        if neo_jar.exists() {
            write_launch_marker(
                runtime,
                json!({ "kind": "jar", "jar": format!("neoforge-{neo_ver}.jar") }),
            )?;
        } else {
            return Err("NeoForge install finished but no launch args/jar found".into());
        }
    }

    server.loader_version = Some(neo_ver);
    Ok(())
}

/// Import a Modrinth `.mrpack` into an existing dedicated server (server-side files only).
pub fn import_dedicated_mrpack(id: String, path: String) -> Result<HostServer, String> {
    use std::io::{Read, Write};
    use zip::ZipArchive;

    let mut server = get_dedicated(&id)?;
    let runtime = dedicated_runtime(&id)?;
    ensure_runtime_dirs(&id)?;

    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut index_raw = String::new();
    {
        let mut index = archive
            .by_name("modrinth.index.json")
            .map_err(|e| format!("Not an mrpack: {e}"))?;
        index.read_to_string(&mut index_raw).map_err(|e| e.to_string())?;
    }
    let index: Value = serde_json::from_str(&index_raw).map_err(|e| e.to_string())?;
    let deps = &index["dependencies"];
    let game_version = deps["minecraft"]
        .as_str()
        .unwrap_or(&server.game_version)
        .to_string();
    let loader = if deps.get("fabric-loader").is_some() {
        "fabric"
    } else if deps.get("quilt-loader").is_some() {
        "quilt"
    } else if deps.get("neoforge").is_some() {
        "neoforge"
    } else if deps.get("forge").is_some() {
        "forge"
    } else {
        server.loader.as_str()
    }
    .to_string();

    server.game_version = crate::models::normalize_game_version(&game_version);
    server.loader = loader;
    server.loader_version = deps["fabric-loader"]
        .as_str()
        .or_else(|| deps["quilt-loader"].as_str())
        .or_else(|| deps["forge"].as_str())
        .or_else(|| deps["neoforge"].as_str())
        .map(|s| s.to_string());
    save_dedicated(&server)?;

    // Install loader jars if needed
    if !server.installed {
        let _ = install_dedicated(id.clone());
        server = get_dedicated(&id)?;
    }

    let client = http_client()?;
    let mods_dir = runtime.join("mods");
    fs::create_dir_all(&mods_dir).ok();

    for file in index["files"].as_array().into_iter().flatten() {
        // Skip client-only entries
        let env = file.get("env");
        let server_env = env
            .and_then(|e| e.get("server"))
            .and_then(|s| s.as_str())
            .unwrap_or("required");
        if server_env == "unsupported" {
            continue;
        }
        let path_in = file["path"].as_str().unwrap_or("");
        let downloads = file["downloads"].as_array();
        let url = downloads
            .and_then(|a| a.first())
            .and_then(|u| u.as_str())
            .unwrap_or("");
        if url.is_empty() {
            continue;
        }
        let dest = runtime.join(path_in);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(bytes) = client.get(url).send().and_then(|r| r.bytes()) {
            let _ = fs::write(&dest, bytes);
        }
    }

    // Extract overrides/ and server-overrides/
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        let rest = name
            .strip_prefix("server-overrides/")
            .or_else(|| name.strip_prefix("overrides/"));
        let Some(rest) = rest else { continue };
        if rest.is_empty() {
            continue;
        }
        let dest = runtime.join(rest);
        if file.is_dir() {
            fs::create_dir_all(&dest).ok();
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut out = fs::File::create(&dest).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
        }
    }

    server.installed = true;
    save_dedicated(&server)?;
    Ok(server)
}

fn download_profile_libraries(profile: &Value) -> Result<(), String> {
    let libraries = meta_dir()?.join("libraries");
    fs::create_dir_all(&libraries).ok();
    let Some(libs) = profile.get("libraries").and_then(|l| l.as_array()) else {
        return Ok(());
    };
    for lib in libs {
        let name = lib.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let url = lib
            .pointer("/downloads/artifact/url")
            .and_then(|u| u.as_str())
            .or_else(|| lib.pointer("/url").and_then(|u| u.as_str()));
        let path = lib
            .pointer("/downloads/artifact/path")
            .and_then(|p| p.as_str())
            .map(|p| p.to_string())
            .or_else(|| maven_path(name));
        let (Some(url), Some(rel)) = (url, path) else {
            continue;
        };
        let dest = libraries.join(&rel);
        let _ = download_file(url, &dest);
    }
    Ok(())
}

fn maven_path(name: &str) -> Option<String> {
    // group:artifact:version
    let parts: Vec<_> = name.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    Some(format!(
        "{group}/{artifact}/{version}/{artifact}-{version}.jar"
    ))
}

/// How to start a dedicated server process.
#[derive(Debug, Clone)]
pub enum LaunchSpec {
    Jar(PathBuf),
    /// Modern Forge: `java @user_jvm_args.txt @libraries/.../win_args.txt nogui`
    ForgeArgs {
        jvm_args: PathBuf,
        forge_args: PathBuf,
    },
}

/// Resolve launch instructions for a dedicated server.
pub fn resolve_launch(id: &str) -> Result<LaunchSpec, String> {
    let server = get_dedicated(id)?;
    let runtime = dedicated_runtime(id)?;
    if !runtime.exists() {
        return Err(NOT_FOUND.into());
    }

    let marker = runtime.join("euml-launch.json");
    if marker.exists() {
        if let Ok(raw) = fs::read_to_string(&marker) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                match kind {
                    "forge_args" => {
                        let jvm = v
                            .get("jvmArgs")
                            .and_then(|s| s.as_str())
                            .unwrap_or("user_jvm_args.txt");
                        let forge = v.get("forgeArgs").and_then(|s| s.as_str());
                        if let Some(forge) = forge {
                            let jvm_path = runtime.join(jvm);
                            let forge_path = runtime.join(forge);
                            if jvm_path.exists() && forge_path.exists() {
                                return Ok(LaunchSpec::ForgeArgs {
                                    jvm_args: jvm_path,
                                    forge_args: forge_path,
                                });
                            }
                        }
                    }
                    "jar" | "fabric" => {
                        if let Some(jar) = v.get("jar").and_then(|s| s.as_str()) {
                            let path = runtime.join(jar);
                            if path.exists() {
                                return Ok(LaunchSpec::Jar(path));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Fallbacks by loader
    match server.loader.as_str() {
        "fabric" => {
            let fabric = runtime.join("fabric-server-launch.jar");
            if fabric.exists() {
                return Ok(LaunchSpec::Jar(fabric));
            }
        }
        "quilt" => {
            let quilt = runtime.join("quilt-server-launch.jar");
            if quilt.exists() {
                return Ok(LaunchSpec::Jar(quilt));
            }
        }
        "paper" => {
            let paper = runtime.join("paper.jar");
            if paper.exists() {
                return Ok(LaunchSpec::Jar(paper));
            }
        }
        "purpur" => {
            let purpur = runtime.join("purpur.jar");
            if purpur.exists() {
                return Ok(LaunchSpec::Jar(purpur));
            }
        }
        _ => {}
    }

    let vanilla = runtime.join("server.jar");
    if vanilla.exists() {
        return Ok(LaunchSpec::Jar(vanilla));
    }
    Err("Server not installed — run Install first".into())
}

/// Resolve which jar to launch (jar-only loaders). Prefer [`resolve_launch`].
pub fn launch_jar(id: &str) -> Result<PathBuf, String> {
    match resolve_launch(id)? {
        LaunchSpec::Jar(p) => Ok(p),
        LaunchSpec::ForgeArgs { .. } => Err(
            "This Forge server uses args-file launch; use resolve_launch".into(),
        ),
    }
}

pub fn not_found_msg() -> &'static str {
    NOT_FOUND
}
