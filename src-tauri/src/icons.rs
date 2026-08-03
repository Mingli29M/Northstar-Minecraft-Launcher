use crate::paths::meta_dir;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub fn icons_cache_dir() -> Result<PathBuf, String> {
    let p = meta_dir()?.join("icons");
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

fn cache_key(source: &Path, hint: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.to_string_lossy().as_bytes());
    hasher.update(hint.as_bytes());
    if let Ok(meta) = fs::metadata(source) {
        if let Ok(modified) = meta.modified() {
            if let Ok(d) = modified.duration_since(std::time::UNIX_EPOCH) {
                hasher.update(d.as_secs().to_le_bytes());
            }
        }
        hasher.update(meta.len().to_le_bytes());
    }
    hex::encode(hasher.finalize())
}

fn to_data_url(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", {
        // Minimal base64 (std-only)
        const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
            let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(T[((n >> 18) & 63) as usize] as char);
            out.push(T[((n >> 12) & 63) as usize] as char);
            out.push(if chunk.len() > 1 {
                T[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                T[(n & 63) as usize] as char
            } else {
                '='
            });
        }
        out
    })
}

fn write_cached(bytes: &[u8], key: &str) -> Option<String> {
    let dir = icons_cache_dir().ok()?;
    let dest = dir.join(format!("{key}.png"));
    if !dest.exists() {
        fs::write(&dest, bytes).ok()?;
    }
    Some(to_data_url(bytes))
}

fn load_cached(key: &str) -> Option<String> {
    let dest = icons_cache_dir().ok()?.join(format!("{key}.png"));
    if dest.exists() {
        let bytes = fs::read(&dest).ok()?;
        return Some(to_data_url(&bytes));
    }
    None
}

/// Extract an icon for a mod jar (fabric.mod.json / quilt.mod.json).
pub fn icon_for_mod_jar(jar_path: &Path) -> Option<String> {
    let key = cache_key(jar_path, "mod");
    if let Some(cached) = load_cached(&key) {
        return Some(cached);
    }
    let file = fs::File::open(jar_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;
    let icon_entry = resolve_jar_icon_path(&mut archive)?;
    let mut entry = archive.by_name(&icon_entry).ok()?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).ok()?;
    if buf.is_empty() {
        return None;
    }
    write_cached(&buf, &key)
}

fn resolve_jar_icon_path<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<String> {
    for meta_name in ["fabric.mod.json", "quilt.mod.json"] {
        if let Ok(mut f) = archive.by_name(meta_name) {
            let mut raw = String::new();
            if f.read_to_string(&mut raw).is_ok() {
                if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                    if let Some(icon) = v.get("icon").and_then(|i| i.as_str()) {
                        return Some(icon.to_string());
                    }
                    if let Some(icon) = v
                        .get("icon")
                        .and_then(|i| i.as_object())
                        .and_then(|o| o.values().next())
                        .and_then(|i| i.as_str())
                    {
                        return Some(icon.to_string());
                    }
                }
            }
        }
    }
    for candidate in ["icon.png", "logo.png", "pack.png"] {
        if archive.by_name(candidate).is_ok() {
            return Some(candidate.into());
        }
    }
    for i in 0..archive.len().min(200) {
        if let Ok(f) = archive.by_index(i) {
            let name = f.name().to_string();
            let lower = name.to_lowercase();
            if lower.ends_with("icon.png") || lower.ends_with("logo.png") {
                return Some(name);
            }
        }
    }
    None
}

pub fn icon_for_pack(path: &Path) -> Option<String> {
    let key = cache_key(path, "pack");
    if let Some(cached) = load_cached(&key) {
        return Some(cached);
    }
    if path.is_dir() {
        let pack = path.join("pack.png");
        if pack.exists() {
            let bytes = fs::read(&pack).ok()?;
            return write_cached(&bytes, &key);
        }
        return None;
    }
    let lower = path.extension()?.to_string_lossy().to_lowercase();
    if lower == "zip" || lower == "jar" {
        let file = fs::File::open(path).ok()?;
        let mut archive = ZipArchive::new(file).ok()?;
        let mut entry = archive.by_name("pack.png").ok()?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).ok()?;
        return write_cached(&buf, &key);
    }
    None
}

pub fn icon_for_world(path: &Path) -> Option<String> {
    let icon = path.join("icon.png");
    if !icon.exists() {
        return None;
    }
    let key = cache_key(&icon, "world");
    if let Some(cached) = load_cached(&key) {
        return Some(cached);
    }
    let bytes = fs::read(&icon).ok()?;
    write_cached(&bytes, &key)
}
