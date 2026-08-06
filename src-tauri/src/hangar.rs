use crate::dedicated;
use crate::paths::dedicated_runtime;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const HANGAR_API: &str = "https://hangar.papermc.io/api/v1";
const USER_AGENT: &str = "Northstar/1.2.1";

fn hangar_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(25))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| e.to_string())
}

fn hangar_get(url: &str) -> Result<Value, String> {
    let client = hangar_client()?;
    let resp = client.get(url).send().map_err(|e| format!("Hangar request failed: {e}"))?;
    let status = resp.status();
    if status.as_u16() == 429 {
        return Err("Hangar rate limit reached — wait a moment and try again".into());
    }
    if !status.is_success() {
        return Err(format!("Hangar API error ({status}): {url}"));
    }
    resp.json().map_err(|e| format!("Hangar response parse error: {e}"))
}

fn normalize_platform(platform: &str) -> String {
    match platform.trim().to_uppercase().as_str() {
        "PURPUR" => "PAPER".into(),
        "" => "PAPER".into(),
        other => other.to_string(),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HangarProject {
    pub slug: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub author: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub downloads: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HangarVersion {
    pub name: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub platform_versions: Vec<String>,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub external_url: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HostPluginEntry {
    pub name: String,
    pub enabled: bool,
    pub path: String,
}

fn parse_project(hit: &Value) -> Option<HangarProject> {
    let ns = hit.get("namespace")?;
    let author = ns["owner"].as_str()?.to_string();
    let slug = ns["slug"].as_str()?.to_string();
    let name = hit["name"].as_str().unwrap_or(&slug).to_string();
    let description = hit["description"].as_str().unwrap_or("").to_string();
    let icon_url = hit["avatarUrl"].as_str().map(|s| s.to_string());
    let category = hit["category"].as_str().map(|s| s.to_string());
    let downloads = hit["stats"]["downloads"].as_u64();
    Some(HangarProject {
        slug,
        name,
        description,
        icon_url,
        author,
        category,
        downloads,
    })
}

fn platform_supported(hit: &Value, platform: &str) -> bool {
    hit.get("supportedPlatforms")
        .and_then(|p| p.get(platform))
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(true)
}

pub fn search_plugins(
    query: String,
    platform: String,
    limit: u32,
) -> Result<Vec<HangarProject>, String> {
    let platform = normalize_platform(&platform);
    let limit = limit.clamp(1, 48);
    let q = query.trim();
    let mut url = format!("{HANGAR_API}/projects?limit={limit}");
    if !q.is_empty() {
        url.push_str(&format!("&q={}", urlencoding::encode(q)));
    }
    let data = hangar_get(&url)?;
    let hits = data["result"].as_array().cloned().unwrap_or_default();
    let mut out = Vec::new();
    for hit in hits {
        if !platform_supported(&hit, &platform) {
            continue;
        }
        if let Some(p) = parse_project(&hit) {
            out.push(p);
        }
        if out.len() >= limit as usize {
            break;
        }
    }
    Ok(out)
}

fn parse_version_entry(v: &Value, platform: &str) -> Option<HangarVersion> {
    let name = v["name"].as_str()?.to_string();
    let created_at = v["createdAt"].as_str().map(|s| s.to_string());
    let platform_versions = v["platformDependencies"][platform]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();
    let dl = v["downloads"][platform].as_object()?;
    let download_url = dl.get("downloadUrl").and_then(|u| u.as_str()).map(|s| s.to_string());
    let external_url = dl.get("externalUrl").and_then(|u| u.as_str()).map(|s| s.to_string());
    let file_name = dl
        .get("fileInfo")
        .and_then(|f| f.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());
    Some(HangarVersion {
        name,
        created_at,
        platform_versions,
        download_url,
        external_url,
        file_name,
    })
}

pub fn list_plugin_versions(
    author: String,
    slug: String,
    platform: String,
) -> Result<Vec<HangarVersion>, String> {
    let platform = normalize_platform(&platform);
    let author = author.trim();
    let slug = slug.trim();
    if author.is_empty() || slug.is_empty() {
        return Err("Author and slug are required".into());
    }
    let url = format!("{HANGAR_API}/projects/{author}/{slug}/versions?limit=25");
    let data = hangar_get(&url)?;
    let versions = data["result"].as_array().cloned().unwrap_or_default();
    Ok(versions
        .iter()
        .filter_map(|v| parse_version_entry(v, &platform))
        .collect())
}

fn fetch_version_detail(author: &str, slug: &str, version: &str, platform: &str) -> Result<HangarVersion, String> {
    let url = format!("{HANGAR_API}/projects/{author}/{slug}/versions/{version}");
    let data = hangar_get(&url)?;
    parse_version_entry(&data, platform).ok_or_else(|| format!("Version '{version}' not found for {platform}"))
}

fn resolve_version(
    author: &str,
    slug: &str,
    version_or_latest: &str,
    platform: &str,
) -> Result<HangarVersion, String> {
    let want = version_or_latest.trim();
    if want.is_empty() || want.eq_ignore_ascii_case("latest") {
        let versions = list_plugin_versions(author.to_string(), slug.to_string(), platform.to_string())?;
        versions
            .into_iter()
            .next()
            .ok_or_else(|| "No versions available for this plugin".into())
    } else {
        fetch_version_detail(author, slug, want, platform)
    }
}

fn download_jar(url: &str, dest: &PathBuf) -> Result<(), String> {
    let client = hangar_client()?;
    let mut resp = client
        .get(url)
        .send()
        .map_err(|e| format!("Download failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Download failed (HTTP {})", resp.status()));
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = fs::File::create(dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut resp, &mut file).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn install_plugin(
    dedicated_id: String,
    author: String,
    slug: String,
    version_or_latest: String,
    platform: String,
) -> Result<HostPluginEntry, String> {
    let _ = dedicated::get_dedicated(&dedicated_id)?;
    let platform = normalize_platform(&platform);
    let author = author.trim().to_string();
    let slug = slug.trim().to_string();
    if author.is_empty() || slug.is_empty() {
        return Err("Author and slug are required".into());
    }

    let version = resolve_version(&author, &slug, &version_or_latest, &platform)?;
    let download_url = version
        .download_url
        .as_deref()
        .filter(|u| !u.is_empty())
        .ok_or_else(|| {
            if let Some(ext) = &version.external_url {
                format!(
                    "Plugin '{}' is hosted externally — download manually from {ext}",
                    version.name
                )
            } else {
                format!("No direct download available for '{}' on {platform}", version.name)
            }
        })?;

    let filename = version
        .file_name
        .clone()
        .filter(|n| n.ends_with(".jar"))
        .unwrap_or_else(|| format!("{slug}-{}.jar", version.name));

    let plugins_dir = dedicated_runtime(&dedicated_id)?.join("plugins");
    fs::create_dir_all(&plugins_dir).map_err(|e| e.to_string())?;
    let dest = plugins_dir.join(&filename);
    download_jar(download_url, &dest)?;

    Ok(HostPluginEntry {
        name: filename.clone(),
        enabled: true,
        path: dest.to_string_lossy().to_string(),
    })
}
