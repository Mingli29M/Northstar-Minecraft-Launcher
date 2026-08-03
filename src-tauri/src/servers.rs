use crate::paths::minecraft_dir;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub ip: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "acceptTextures")]
    pub accept_textures: Option<i8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ServersRoot {
    #[serde(default)]
    servers: Vec<ServerEntry>,
}

fn servers_path(instance_id: &str) -> Result<PathBuf, String> {
    Ok(minecraft_dir(instance_id)?.join("servers.dat"))
}

fn read_servers_dat(path: &PathBuf) -> Result<ServersRoot, String> {
    if !path.exists() {
        return Ok(ServersRoot::default());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    // Try gzip first (vanilla), then raw NBT.
    let decoded = {
        let mut dec = GzDecoder::new(&bytes[..]);
        let mut out = Vec::new();
        match dec.read_to_end(&mut out) {
            Ok(_) if !out.is_empty() => out,
            _ => bytes,
        }
    };
    fastnbt::from_bytes::<ServersRoot>(&decoded).map_err(|e| format!("Invalid servers.dat: {e}"))
}

fn write_servers_dat(path: &PathBuf, root: &ServersRoot) -> Result<(), String> {
    let nbt = fastnbt::to_bytes(root).map_err(|e| e.to_string())?;
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(&nbt).map_err(|e| e.to_string())?;
    let gz = enc.finish().map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, gz).map_err(|e| e.to_string())
}

pub fn list_servers(instance_id: String) -> Result<Vec<ServerEntry>, String> {
    let path = servers_path(&instance_id)?;
    Ok(read_servers_dat(&path)?.servers)
}

pub fn add_server(instance_id: String, name: String, ip: String) -> Result<Vec<ServerEntry>, String> {
    let path = servers_path(&instance_id)?;
    let mut root = read_servers_dat(&path)?;
    let name = name.trim().to_string();
    let ip = ip.trim().to_string();
    if name.is_empty() || ip.is_empty() {
        return Err("Server name and address are required".into());
    }
    root.servers.push(ServerEntry {
        name,
        ip,
        icon: None,
        accept_textures: None,
    });
    write_servers_dat(&path, &root)?;
    Ok(root.servers)
}

pub fn update_server(
    instance_id: String,
    index: usize,
    name: String,
    ip: String,
) -> Result<Vec<ServerEntry>, String> {
    let path = servers_path(&instance_id)?;
    let mut root = read_servers_dat(&path)?;
    let entry = root
        .servers
        .get_mut(index)
        .ok_or_else(|| "Server index out of range".to_string())?;
    let name = name.trim().to_string();
    let ip = ip.trim().to_string();
    if name.is_empty() || ip.is_empty() {
        return Err("Server name and address are required".into());
    }
    entry.name = name;
    entry.ip = ip;
    write_servers_dat(&path, &root)?;
    Ok(root.servers)
}

pub fn remove_server(instance_id: String, index: usize) -> Result<Vec<ServerEntry>, String> {
    let path = servers_path(&instance_id)?;
    let mut root = read_servers_dat(&path)?;
    if index >= root.servers.len() {
        return Err("Server index out of range".into());
    }
    root.servers.remove(index);
    write_servers_dat(&path, &root)?;
    Ok(root.servers)
}
