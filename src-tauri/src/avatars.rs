//! Resolve and cache Minecraft player head icons locally.
//!
//! External avatar CDNs (Crafatar, etc.) are often blocked or flaky from the
//! WebView. Fetching in Rust with multiple fallbacks and returning a data URL
//! makes heads load with or without VPN.

use crate::paths::meta_dir;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const STEVE_UUID: &str = "8667ba71b85a4004af54457a9734eed7";

#[derive(Debug, Deserialize)]
struct ProfileResponse {
    #[serde(default)]
    properties: Vec<ProfileProperty>,
}

#[derive(Debug, Deserialize)]
struct ProfileProperty {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct TexturePayload {
    textures: TextureMap,
}

#[derive(Debug, Deserialize)]
struct TextureMap {
    #[serde(rename = "SKIN")]
    skin: Option<SkinTexture>,
}

#[derive(Debug, Deserialize)]
struct SkinTexture {
    url: String,
}

fn avatars_dir() -> Result<PathBuf, String> {
    let p = meta_dir()?.join("avatars");
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

fn cache_path(kind: &str, uuid: &str, username: &str) -> Result<PathBuf, String> {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b"|");
    hasher.update(uuid.replace('-', "").to_lowercase().as_bytes());
    hasher.update(b"|");
    hasher.update(username.to_lowercase().as_bytes());
    let key = hex::encode(hasher.finalize());
    Ok(avatars_dir()?.join(format!("{key}.png")))
}

fn cache_fresh(path: &PathBuf) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    SystemTime::now()
        .duration_since(modified)
        .map(|d| d < CACHE_TTL)
        .unwrap_or(false)
}

fn to_data_url(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", {
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

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("NorthstarLauncher/1.1.2")
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())
}

fn looks_like_png(bytes: &[u8]) -> bool {
    bytes.len() > 8 && bytes.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'])
}

fn fetch_bytes(client: &reqwest::blocking::Client, url: &str) -> Option<Vec<u8>> {
    let resp = client.get(url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().ok()?.to_vec();
    if looks_like_png(&bytes) {
        Some(bytes)
    } else {
        None
    }
}

fn normalize_uuid(uuid: &str) -> String {
    uuid.replace('-', "").to_lowercase()
}

fn candidate_urls(kind: &str, uuid: &str, username: &str) -> Vec<String> {
    let id = normalize_uuid(uuid);
    let name = username.trim();
    let mut urls = Vec::new();

    match kind {
        "littleskin" => {
            if !name.is_empty() {
                urls.push(format!(
                    "https://littleskin.cn/avatar/player/{}",
                    urlencoding::encode(name)
                ));
                urls.push(format!(
                    "https://littleskin.cn/avatar/{}",
                    urlencoding::encode(name)
                ));
            }
            if !id.is_empty() && id != "0".repeat(32) {
                urls.push(format!("https://littleskin.cn/avatar/{id}"));
            }
        }
        "offline" => {
            // Offline accounts have no Mojang skin — use Steve.
            urls.push(format!("https://crafthead.net/avatar/{STEVE_UUID}/64"));
            urls.push(format!("https://mc-heads.net/avatar/{STEVE_UUID}/64"));
            urls.push(format!(
                "https://crafatar.com/avatars/{STEVE_UUID}?overlay=true&size=64"
            ));
        }
        _ => {
            if !id.is_empty() && id.len() >= 32 && !id.chars().all(|c| c == '0') {
                urls.push(format!("https://crafthead.net/avatar/{id}/64"));
                urls.push(format!("https://mc-heads.net/avatar/{id}/64"));
                urls.push(format!("https://mineatar.io/face/{id}?scale=8&overlay=true"));
                urls.push(format!(
                    "https://crafatar.com/avatars/{id}?overlay=true&size=64"
                ));
            }
            if !name.is_empty() {
                urls.push(format!(
                    "https://crafthead.net/avatar/{}/64",
                    urlencoding::encode(name)
                ));
                urls.push(format!(
                    "https://mc-heads.net/avatar/{}/64",
                    urlencoding::encode(name)
                ));
            }
        }
    }
    urls
}

fn profile_urls(kind: &str, uuid: &str) -> Vec<String> {
    let id = normalize_uuid(uuid);
    if id.is_empty() || id.len() < 32 {
        return Vec::new();
    }
    match kind {
        "littleskin" => vec![format!(
            "https://littleskin.cn/api/yggdrasil/sessionserver/session/minecraft/profile/{id}"
        )],
        "offline" => Vec::new(),
        _ => vec![
            format!("https://sessionserver.mojang.com/session/minecraft/profile/{id}"),
            format!("https://bmclapi2.bangbang93.com/session/minecraft/profile/{id}"),
        ],
    }
}

fn skin_url_from_profile(client: &reqwest::blocking::Client, profile_url: &str) -> Option<String> {
    let resp = client.get(profile_url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let profile: ProfileResponse = resp.json().ok()?;
    let prop = profile.properties.into_iter().find(|p| p.name == "textures")?;
    let decoded = base64_decode(&prop.value)?;
    let payload: TexturePayload = serde_json::from_slice(&decoded).ok()?;
    payload.textures.skin.map(|s| s.url)
}

/// Minimal base64 decode (std-only) for Mojang texture payloads.
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' | b'-' => Some(62),
            b'/' | b'_' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'=')
        .collect();
    if bytes.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let a = val(chunk[0])?;
        let b = val(*chunk.get(1)?)?;
        let c = chunk.get(2).and_then(|x| val(*x));
        let d = chunk.get(3).and_then(|x| val(*x));
        out.push((a << 2) | (b >> 4));
        if let Some(c) = c {
            out.push(((b & 0x0f) << 4) | (c >> 2));
            if let Some(d) = d {
                out.push(((c & 0x03) << 6) | d);
            }
        }
    }
    Some(out)
}

fn crop_head_from_skin(skin_png: &[u8]) -> Option<Vec<u8>> {
    use image::imageops::{overlay, FilterType};
    use image::{DynamicImage, ImageFormat, RgbaImage};

    let img = image::load_from_memory(skin_png).ok()?;
    // Classic / modern skins are 64x64 (or 64x32 legacy). Face is 8x8 at (8,8).
    if img.width() < 64 || img.height() < 32 {
        return None;
    }
    let face = img.crop_imm(8, 8, 8, 8);
    let mut composed = RgbaImage::new(8, 8);
    overlay(&mut composed, &face.to_rgba8(), 0, 0);
    // Helmet/overlay layer at (40, 8) on modern skins.
    if img.height() >= 64 {
        let helm = img.crop_imm(40, 8, 8, 8);
        overlay(&mut composed, &helm.to_rgba8(), 0, 0);
    }
    let scaled = DynamicImage::ImageRgba8(composed).resize_exact(64, 64, FilterType::Nearest);
    let mut out = Vec::new();
    scaled
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .ok()?;
    Some(out)
}

fn fetch_from_skin_profile(
    client: &reqwest::blocking::Client,
    kind: &str,
    uuid: &str,
) -> Option<Vec<u8>> {
    for profile_url in profile_urls(kind, uuid) {
        let Some(skin_url) = skin_url_from_profile(client, &profile_url) else {
            continue;
        };
        // Prefer BMCLAPI texture mirrors when Mojang textures are blocked.
        let mirrors = [
            skin_url.clone(),
            skin_url.replace(
                "https://textures.minecraft.net",
                "https://bmclapi2.bangbang93.com",
            ),
            skin_url.replace(
                "https://textures.minecraft.net/texture/",
                "https://bmclapi2.bangbang93.com/textures/",
            ),
        ];
        for url in &mirrors {
            if let Some(skin) = fetch_bytes(client, url) {
                if let Some(head) = crop_head_from_skin(&skin) {
                    return Some(head);
                }
            }
        }
    }
    None
}

/// Resolve a player head as a `data:image/png;base64,...` URL (cached on disk).
pub fn resolve_account_avatar(
    kind: String,
    uuid: String,
    username: String,
) -> Result<Option<String>, String> {
    let kind = kind.to_lowercase();
    let path = cache_path(&kind, &uuid, &username)?;
    if cache_fresh(&path) {
        if let Ok(bytes) = fs::read(&path) {
            if looks_like_png(&bytes) {
                return Ok(Some(to_data_url(&bytes)));
            }
        }
    }

    let client = http_client()?;
    let mut bytes: Option<Vec<u8>> = None;

    for url in candidate_urls(&kind, &uuid, &username) {
        if let Some(b) = fetch_bytes(&client, &url) {
            bytes = Some(b);
            break;
        }
    }

    if bytes.is_none() {
        bytes = fetch_from_skin_profile(&client, &kind, &uuid);
    }

    // Last resort for offline / failed online: Steve head from bundled crop if CDN fails.
    if bytes.is_none() && (kind == "offline" || kind == "microsoft") {
        bytes = fetch_from_skin_profile(&client, "microsoft", STEVE_UUID);
    }

    let Some(png) = bytes else {
        return Ok(None);
    };

    let _ = fs::write(&path, &png);
    Ok(Some(to_data_url(&png)))
}
