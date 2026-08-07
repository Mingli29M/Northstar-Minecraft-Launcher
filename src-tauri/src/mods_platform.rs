use crate::download::{download_file, download_many_progress, emit_idle, emit_progress, DownloadProgress};
use crate::instances::{create_instance, get_instance};
use crate::models::{LoaderKind, ModEntry};
use crate::paths::{app_root, minecraft_dir};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use zip::ZipArchive;

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct ModrinthHit {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub slug: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub versions: Vec<ModrinthVersion>,
}

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct ModrinthVersion {
    pub id: String,
    pub version_number: String,
    #[serde(default)]
    pub name: String,
    /// release | beta | alpha
    #[serde(default)]
    pub version_type: String,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub date_published: String,
    pub files: Vec<ModrinthFile>,
    #[serde(default)]
    pub dependencies: Vec<ModrinthDependency>,
    #[serde(default)]
    pub project_id: String,
}

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct ModrinthDependency {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
    /// required | optional | incompatible | embedded
    #[serde(default)]
    pub dependency_type: String,
    #[serde(default)]
    pub project_title: Option<String>,
    #[serde(default)]
    pub project_slug: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    #[serde(default)]
    pub sha1: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct ModrinthGalleryImage {
    pub url: String,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub title: Option<String>,
}

pub fn search_mods(
    query: String,
    game_version: String,
    loader: String,
    categories: Option<Vec<String>>,
) -> Result<Vec<ModrinthHit>, String> {
    search_modrinth_projects(&query, &game_version, &loader, "mod", categories.as_deref())
}

/// Search Modrinth for mods / modpacks / resource packs / shaders / datapacks.
pub fn search_content(
    query: String,
    game_version: String,
    loader: String,
    project_type: String,
    categories: Option<Vec<String>>,
) -> Result<Vec<ModrinthHit>, String> {
    let pt = match project_type.as_str() {
        "modpack" | "modpacks" | "pack" | "packs" => "modpack",
        "resourcepack" | "resourcepacks" => "resourcepack",
        "shader" | "shaders" | "shaderpack" | "shaderpacks" => "shader",
        "datapack" | "datapacks" => "datapack",
        _ => "mod",
    };
    search_modrinth_projects(&query, &game_version, &loader, pt, categories.as_deref())
}

fn search_modrinth_projects(
    query: &str,
    game_version: &str,
    loader: &str,
    project_type: &str,
    categories: Option<&[String]>,
) -> Result<Vec<ModrinthHit>, String> {
    let game_version = crate::models::normalize_game_version(game_version);
    if game_version.is_empty() || !game_version.chars().next().unwrap_or('x').is_ascii_digit() {
        return Err(format!(
            "Invalid Minecraft version '{game_version}'. Pick a real version (e.g. 1.21.1), not an instance name."
        ));
    }
    let loader = LoaderKind::from_str_loose(loader).as_str().to_string();

    let client = reqwest::blocking::Client::new();
    let mut facet_parts = vec![
        format!("[\"versions:{game_version}\"]"),
        format!("[\"project_type:{project_type}\"]"),
    ];
    // Mods and modpacks are loader-scoped; resource/shader/datapacks are not.
    if project_type == "mod" || project_type == "modpack" {
        match loader.as_str() {
            "quilt" => facet_parts.push("[\"categories:quilt\",\"categories:fabric\"]".into()),
            "forge" => facet_parts.push("[\"categories:forge\"]".into()),
            "neoforge" => facet_parts.push("[\"categories:neoforge\"]".into()),
            "fabric" => facet_parts.push("[\"categories:fabric\"]".into()),
            _ => {}
        }
    }
    if let Some(cats) = categories {
        for c in cats {
            let c = c.trim();
            if c.is_empty() {
                continue;
            }
            facet_parts.push(format!("[\"categories:{c}\"]"));
        }
    }
    let facets = format!("[{}]", facet_parts.join(","));
    let q = query.trim();
    let index = if q.is_empty() { "downloads" } else { "relevance" };
    let mut url = format!(
        "https://api.modrinth.com/v2/search?limit=24&index={index}&facets={}",
        urlencoding::encode(&facets)
    );
    if !q.is_empty() {
        url.push_str(&format!("&query={}", urlencoding::encode(q)));
    }
    let data: Value = client
        .get(&url)
        .header("User-Agent", "Northstar/1.2.3")
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let hit_meta: Vec<(String, String, String, String, Option<String>, Vec<String>)> = data["hits"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|hit| {
            let project_id = hit["project_id"].as_str()?.to_string();
            if project_id.is_empty() {
                return None;
            }
            let categories = hit["categories"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|c| c.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();
            Some((
                project_id,
                hit["title"].as_str().unwrap_or("").to_string(),
                hit["description"].as_str().unwrap_or("").to_string(),
                hit["slug"].as_str().unwrap_or("").to_string(),
                hit["icon_url"].as_str().map(|s| s.to_string()),
                categories,
            ))
        })
        .collect();

    use rayon::prelude::*;
    let versions_map: Vec<(String, Vec<ModrinthVersion>)> = hit_meta
        .par_iter()
        .map(|(project_id, _, _, _, _, _)| {
            let versions =
                fetch_compatible_versions(project_id, &game_version, &loader, project_type, 3)
                    .unwrap_or_default();
            (project_id.clone(), versions)
        })
        .collect();

    let mut hits = Vec::with_capacity(hit_meta.len());
    for ((project_id, title, description, slug, icon_url, categories), (_, versions)) in
        hit_meta.into_iter().zip(versions_map.into_iter())
    {
        hits.push(ModrinthHit {
            project_id,
            title,
            description,
            slug,
            icon_url,
            categories,
            versions,
        });
    }
    Ok(hits)
}

fn fetch_compatible_versions(
    project_id: &str,
    game_version: &str,
    loader: &str,
    project_type: &str,
    limit: usize,
) -> Result<Vec<ModrinthVersion>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let mut url = format!(
        "https://api.modrinth.com/v2/project/{project_id}/version?game_versions={}",
        urlencoding::encode(&format!("[\"{game_version}\"]"))
    );
    let loader_l = loader.to_lowercase();
    let filter_loaders = matches!(project_type, "mod" | "modpack") && loader_l != "vanilla";
    if filter_loaders {
        let loaders_json = if loader_l == "quilt" {
            "[\"quilt\",\"fabric\"]".to_string()
        } else {
            format!("[\"{loader_l}\"]")
        };
        url.push_str(&format!("&loaders={}", urlencoding::encode(&loaders_json)));
    }
    let data: Value = client
        .get(&url)
        .header("User-Agent", "Northstar/1.2.3")
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for v in data.as_array().into_iter().flatten() {
        let loaders = v["loaders"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let versions = v["game_versions"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !versions.iter().any(|g| g == game_version) {
            continue;
        }
        if filter_loaders && !loaders.iter().any(|l| l.eq_ignore_ascii_case(&loader_l)) {
            if !(loader_l == "quilt" && loaders.iter().any(|l| l == "fabric")) {
                continue;
            }
        }
        let parsed = parse_modrinth_version(v, project_id);
        if parsed.files.is_empty() {
            continue;
        }
        out.push(parsed);
        if out.len() >= limit {
            break;
        }
    }
    Ok(out)
}

fn parse_modrinth_deps(v: &Value) -> Vec<ModrinthDependency> {
    v["dependencies"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|d| ModrinthDependency {
            project_id: d["project_id"].as_str().map(|s| s.to_string()),
            version_id: d["version_id"].as_str().map(|s| s.to_string()),
            dependency_type: d["dependency_type"].as_str().unwrap_or("required").to_string(),
            project_title: None,
            project_slug: None,
        })
        .collect()
}

fn parse_modrinth_file(f: &Value) -> ModrinthFile {
    ModrinthFile {
        url: f["url"].as_str().unwrap_or("").to_string(),
        filename: f["filename"].as_str().unwrap_or("file.zip").to_string(),
        primary: f["primary"].as_bool().unwrap_or(false),
        sha1: f["hashes"]["sha1"].as_str().map(|s| s.to_string()),
    }
}

fn parse_modrinth_version(v: &Value, fallback_project: &str) -> ModrinthVersion {
    let files = v["files"]
        .as_array()
        .into_iter()
        .flatten()
        .map(parse_modrinth_file)
        .collect::<Vec<_>>();
    let loaders = v["loaders"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();
    let game_versions = v["game_versions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();
    ModrinthVersion {
        id: v["id"].as_str().unwrap_or("").to_string(),
        version_number: v["version_number"].as_str().unwrap_or("").to_string(),
        name: v["name"].as_str().unwrap_or("").to_string(),
        version_type: v["version_type"].as_str().unwrap_or("release").to_string(),
        loaders,
        game_versions,
        date_published: v["date_published"].as_str().unwrap_or("").to_string(),
        files,
        dependencies: parse_modrinth_deps(v),
        project_id: v["project_id"]
            .as_str()
            .unwrap_or(fallback_project)
            .to_string(),
    }
}

/// Batch-resolve project id → (slug, title) for dependency labels.
fn fetch_project_titles(project_ids: &[String]) -> HashMap<String, (String, String)> {
    let mut unique = Vec::new();
    for id in project_ids {
        if !id.is_empty() && !unique.contains(id) {
            unique.push(id.clone());
        }
    }
    if unique.is_empty() {
        return HashMap::new();
    }
    let Ok(ids) = serde_json::to_string(&unique) else {
        return HashMap::new();
    };
    let url = format!(
        "https://api.modrinth.com/v2/projects?ids={}",
        urlencoding::encode(&ids)
    );
    let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
    else {
        return HashMap::new();
    };
    let Ok(data) = client
        .get(&url)
        .header("User-Agent", "Northstar/1.2.3")
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.json::<Value>())
    else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for project in data.as_array().into_iter().flatten() {
        if let (Some(id), Some(slug)) = (project["id"].as_str(), project["slug"].as_str()) {
            let title = project["title"].as_str().unwrap_or(slug).to_string();
            out.insert(id.to_string(), (slug.to_string(), title));
        }
    }
    out
}

fn enrich_version_deps(versions: &mut [ModrinthVersion]) {
    let mut ids = Vec::new();
    for v in versions.iter() {
        for d in &v.dependencies {
            if let Some(pid) = &d.project_id {
                ids.push(pid.clone());
            }
        }
    }
    let titles = fetch_project_titles(&ids);
    for v in versions.iter_mut() {
        for d in &mut v.dependencies {
            if let Some(pid) = &d.project_id {
                if let Some((slug, title)) = titles.get(pid) {
                    d.project_slug = Some(slug.clone());
                    d.project_title = Some(title.clone());
                }
            }
        }
    }
}

/// Version ids (and filenames) already present in the instance mods folder.
pub fn installed_modrinth_markers(instance_id: String) -> Result<InstalledModMarkers, String> {
    let mods = minecraft_dir(&instance_id)?.join("mods");
    let mut version_ids = Vec::new();
    let mut filenames = Vec::new();
    if !mods.is_dir() {
        return Ok(InstalledModMarkers {
            version_ids,
            filenames,
        });
    }
    let mut hashes = Vec::new();
    for entry in fs::read_dir(&mods).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let base = name.strip_suffix(".disabled").unwrap_or(&name);
        if !(base.ends_with(".jar") || base.ends_with(".zip")) {
            continue;
        }
        filenames.push(base.to_string());
        if let Ok(hash) = file_sha1_hex(&path) {
            hashes.push(hash);
        }
    }
    if let Ok(map) = lookup_versions_by_hashes(&hashes) {
        for v in map.values() {
            if !v.id.is_empty() && !version_ids.contains(&v.id) {
                version_ids.push(v.id.clone());
            }
        }
    }
    Ok(InstalledModMarkers {
        version_ids,
        filenames,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct InstalledModMarkers {
    pub version_ids: Vec<String>,
    pub filenames: Vec<String>,
}

/// Resolve many SHA1 hashes in one Modrinth request.
pub fn lookup_versions_by_hashes(
    hashes: &[String],
) -> Result<HashMap<String, ModrinthVersion>, String> {
    if hashes.is_empty() {
        return Ok(HashMap::new());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let data: Value = client
        .post("https://api.modrinth.com/v2/version_files")
        .header("User-Agent", "Northstar/1.2.3")
        .json(&serde_json::json!({
            "hashes": hashes,
            "algorithm": "sha1"
        }))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    let mut versions = HashMap::new();
    for (hash, version) in data.as_object().into_iter().flatten() {
        versions.insert(hash.clone(), parse_modrinth_version(version, ""));
    }
    Ok(versions)
}

/// Map Modrinth project ids to public slugs for local mod-id reconciliation.
pub fn fetch_project_slugs(project_ids: &[String]) -> Result<HashMap<String, String>, String> {
    if project_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let ids = serde_json::to_string(project_ids).map_err(|e| e.to_string())?;
    let url = format!(
        "https://api.modrinth.com/v2/projects?ids={}",
        urlencoding::encode(&ids)
    );
    let data: Value = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?
        .get(&url)
        .header("User-Agent", "Northstar/1.2.3")
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    let mut slugs = HashMap::new();
    for project in data.as_array().into_iter().flatten() {
        if let (Some(id), Some(slug)) = (project["id"].as_str(), project["slug"].as_str()) {
            slugs.insert(id.to_string(), slug.to_string());
        }
    }
    Ok(slugs)
}

/// Fetch a Modrinth version (includes dependencies[]).
pub fn fetch_version(version_id: &str) -> Result<ModrinthVersion, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let data: Value = client
        .get(format!("https://api.modrinth.com/v2/version/{version_id}"))
        .header("User-Agent", "Northstar/1.2.3")
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    Ok(parse_modrinth_version(&data, ""))
}

/// Install a Modrinth project by id (latest compatible version) and required deps.
pub fn install_project_with_deps(
    app: Option<&AppHandle>,
    instance_id: &str,
    project_id: &str,
    game_version: &str,
    loader: &str,
    depth: u8,
) -> Result<Vec<String>, String> {
    if depth > 6 {
        return Ok(Vec::new());
    }
    let versions = fetch_compatible_versions(project_id, game_version, loader, "mod", 3)?;
    let version = versions
        .first()
        .ok_or_else(|| format!("No compatible Modrinth version for `{project_id}`"))?;
    // Re-fetch for full dependency list
    let full = fetch_version(&version.id).unwrap_or_else(|_| version.clone());
    let mut installed = Vec::new();
    install_mod_jar(app, instance_id, &full.id)?;
    installed.push(project_id.to_string());

    for dep in full.dependencies {
        if !dep.dependency_type.eq_ignore_ascii_case("required") {
            continue;
        }
        let Some(dep_project) = dep.project_id.filter(|s| !s.is_empty()) else {
            continue;
        };
        match install_project_with_deps(
            app,
            instance_id,
            &dep_project,
            game_version,
            loader,
            depth + 1,
        ) {
            Ok(more) => installed.extend(more),
            Err(e) => {
                // Soft-fail nested deps; caller can re-scan
                let _ = e;
            }
        }
    }
    Ok(installed)
}

/// SHA1 hex of a file (for Modrinth version_file lookup).
pub fn file_sha1_hex(path: &std::path::Path) -> Result<String, String> {
    use sha1::{Digest, Sha1};
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn download_modrinth_version_file(
    app: Option<&AppHandle>,
    version_id: &str,
    dest_dir: &PathBuf,
    phase: &str,
) -> Result<String, String> {
    let full = fetch_version(version_id)?;
    let file = full
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| full.files.first())
        .ok_or("No file on version")?;
    if file.url.is_empty() {
        return Err("Missing download URL".into());
    }
    fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    let dest = dest_dir.join(&file.filename);

    emit_progress(
        app,
        DownloadProgress {
            phase: phase.into(),
            done: 0,
            total: 1,
            failed: 0,
            current_file: Some(file.filename.clone()),
            bytes_per_sec: None,
            message: format!("Downloading {}…", file.filename),
            active: true,
            ..Default::default()
        },
    );
    // Uses process AppHandle for byte-level ticks when Content-Length is known.
    download_file(&file.url, &dest)?;
    Ok(file.filename.clone())
}

/// Download a single mod jar (no dependency walk, no idle emit).
fn install_mod_jar(
    app: Option<&AppHandle>,
    instance_id: &str,
    version_id: &str,
) -> Result<ModEntry, String> {
    let dest_dir = minecraft_dir(instance_id)?.join("mods");
    let filename = download_modrinth_version_file(app, version_id, &dest_dir, "mod")?;
    let path = dest_dir.join(&filename);
    Ok(ModEntry {
        file_name: filename,
        enabled: true,
        path: path.to_string_lossy().to_string(),
        icon_path: crate::icons::icon_for_mod_jar(&path),
    })
}

pub fn install_mod(
    app: Option<&AppHandle>,
    instance_id: String,
    project_id: String,
    version_id: String,
) -> Result<ModEntry, String> {
    let entry = match install_mod_jar(app, &instance_id, &version_id) {
        Ok(e) => e,
        Err(e) => {
            emit_idle(app, format!("Mod install failed: {e}"));
            return Err(e);
        }
    };

    // Auto-install required Modrinth dependency chain.
    if let Ok(inst) = get_instance(&instance_id) {
        if let Ok(full) = fetch_version(&version_id) {
            let required: Vec<_> = full
                .dependencies
                .iter()
                .filter(|d| d.dependency_type.eq_ignore_ascii_case("required"))
                .filter_map(|d| d.project_id.clone().filter(|s| !s.is_empty()))
                .filter(|dep_project| dep_project != &project_id)
                .collect();
            let total = required.len().saturating_add(1);
            for (i, dep_project) in required.iter().enumerate() {
                let label = dep_project.clone();
                emit_progress(
                    app,
                    DownloadProgress {
                        phase: "mod-deps".into(),
                        done: i + 1,
                        total,
                        failed: 0,
                        current_file: Some(label.clone()),
                        bytes_per_sec: None,
                        message: format!(
                            "Installing dependency {label} ({}/{})…",
                            i + 1,
                            required.len()
                        ),
                        active: true,
                        ..Default::default()
                    },
                );
                let _ = install_project_with_deps(
                    app,
                    &instance_id,
                    dep_project,
                    &inst.game_version,
                    inst.loader.as_str(),
                    1,
                );
            }
        }
    }

    emit_idle(app, format!("Installed {}", entry.file_name));
    Ok(entry)
}

/// Install a Modrinth file into resourcepacks / shaderpacks / datapacks (world).
pub fn install_content_from_modrinth(
    app: Option<&AppHandle>,
    instance_id: String,
    version_id: String,
    kind: String,
    world_name: Option<String>,
) -> Result<crate::models::ContentItem, String> {
    let kind_norm = match kind.as_str() {
        "resourcepack" | "resourcepacks" => "resourcepacks",
        "shader" | "shaders" | "shaderpack" | "shaderpacks" => "shaderpacks",
        "datapack" | "datapacks" => "datapacks",
        _ => "mods",
    };

    let result = (|| -> Result<crate::models::ContentItem, String> {
        if kind_norm == "datapacks" {
            let world = world_name.ok_or("Select a world for datapack install")?;
            let dest_dir = minecraft_dir(&instance_id)?
                .join("saves")
                .join(&world)
                .join("datapacks");
            if !minecraft_dir(&instance_id)?.join("saves").join(&world).exists() {
                return Err("World not found".into());
            }
            let filename = download_modrinth_version_file(app, &version_id, &dest_dir, kind_norm)?;
            let path = dest_dir.join(&filename);
            return Ok(crate::models::ContentItem {
                name: filename,
                path: path.to_string_lossy().to_string(),
                kind: "datapacks".into(),
                icon_path: crate::icons::icon_for_pack(&path),
            });
        }

        let dest_dir = minecraft_dir(&instance_id)?.join(kind_norm);
        let filename = download_modrinth_version_file(app, &version_id, &dest_dir, kind_norm)?;
        let path = dest_dir.join(&filename);
        Ok(crate::models::ContentItem {
            name: filename,
            path: path.to_string_lossy().to_string(),
            kind: kind_norm.into(),
            icon_path: crate::icons::icon_for_pack(&path),
        })
    })();

    match &result {
        Ok(item) => emit_idle(app, format!("Installed {}", item.name)),
        Err(e) => emit_idle(app, format!("Install failed: {e}")),
    }
    result
}

pub fn list_instance_mods(instance_id: String) -> Result<Vec<ModEntry>, String> {
    let mods = minecraft_dir(&instance_id)?.join("mods");
    fs::create_dir_all(&mods).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for entry in fs::read_dir(mods).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".jar") || name.ends_with(".jar.disabled") {
            let enabled = name.ends_with(".jar") && !name.ends_with(".jar.disabled");
            out.push(ModEntry {
                file_name: name,
                enabled,
                path: entry.path().to_string_lossy().to_string(),
                icon_path: crate::icons::icon_for_mod_jar(&entry.path()),
            });
        }
    }
    out.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(out)
}

pub fn set_mod_enabled(instance_id: String, file_name: String, enabled: bool) -> Result<Vec<ModEntry>, String> {
    let mods = minecraft_dir(&instance_id)?.join("mods");
    let current = mods.join(&file_name);
    if !current.exists() {
        return Err("Mod file not found".into());
    }
    let target_name = if enabled {
        file_name.trim_end_matches(".disabled").to_string()
    } else if file_name.ends_with(".disabled") {
        file_name.clone()
    } else {
        format!("{file_name}.disabled")
    };
    let target = mods.join(&target_name);
    if current != target {
        fs::rename(current, target).map_err(|e| e.to_string())?;
    }
    list_instance_mods(instance_id)
}

pub fn uninstall_mod(instance_id: String, file_name: String) -> Result<Vec<ModEntry>, String> {
    let path = minecraft_dir(&instance_id)?.join("mods").join(&file_name);
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    list_instance_mods(instance_id)
}

/// Download a Modrinth modpack version (`.mrpack`) and import it as a new instance.
pub fn install_modpack_from_modrinth(
    app: Option<&AppHandle>,
    version_id: String,
) -> Result<crate::models::Instance, String> {
    crate::console_log::append(app, "Fetching Modrinth modpack version…", "info");
    let version = fetch_version(&version_id)?;
    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or("Modrinth modpack version has no files")?;
    if file.url.is_empty() {
        return Err("Modrinth modpack file has no download URL".into());
    }
    let cache = app_root()?.join("cache").join("mrpacks");
    fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let dest = cache.join(if file.filename.to_ascii_lowercase().ends_with(".mrpack") {
        file.filename.clone()
    } else {
        format!("{version_id}.mrpack")
    });
    emit_progress(
        app,
        DownloadProgress {
            phase: "modpack".into(),
            done: 0,
            total: 1,
            failed: 0,
            current_file: Some(file.filename.clone()),
            bytes_per_sec: None,
            message: format!("Downloading {}", file.filename),
            active: true,
            ..Default::default()
        },
    );
    download_file(&file.url, &dest)?;
    let result = import_mrpack(app, dest.to_string_lossy().to_string());
    let _ = fs::remove_file(&dest);
    if result.is_err() {
        emit_idle(app, "Modpack install failed");
    }
    result
}

fn mrpack_file_for_client(file: &Value) -> bool {
    match file.pointer("/env/client").and_then(|v| v.as_str()) {
        Some("unsupported") => false,
        _ => true,
    }
}

pub fn import_mrpack(
    app: Option<&AppHandle>,
    path: String,
) -> Result<crate::models::Instance, String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.is_file() {
        return Err(format!("Modpack file not found: {path}"));
    }

    crate::console_log::append(app, format!("Reading modpack {path}…"), "info");
    let index_raw = {
        let file = fs::File::open(&path_buf).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
        let mut index = archive
            .by_name("modrinth.index.json")
            .map_err(|e| format!("Not an mrpack: {e}"))?;
        let mut raw = String::new();
        std::io::Read::read_to_string(&mut index, &mut raw).map_err(|e| e.to_string())?;
        raw
    };
    let index: Value = serde_json::from_str(&index_raw).map_err(|e| e.to_string())?;
    let name = index["name"].as_str().unwrap_or("Imported Pack").to_string();
    let deps = &index["dependencies"];
    let raw_gv = deps["minecraft"].as_str().unwrap_or("1.21.1");
    let game_version = crate::models::normalize_game_version(raw_gv);
    if !crate::models::is_plausible_game_version(&game_version) {
        return Err(format!(
            "Modpack has invalid Minecraft version '{raw_gv}'. Expected something like 1.21.1."
        ));
    }
    let loader = if deps.get("fabric-loader").is_some() {
        "fabric"
    } else if deps.get("quilt-loader").is_some() {
        "quilt"
    } else if deps.get("neoforge").is_some() {
        "neoforge"
    } else if deps.get("forge").is_some() {
        "forge"
    } else {
        "vanilla"
    };
    let loader_version = deps["fabric-loader"]
        .as_str()
        .or_else(|| deps["quilt-loader"].as_str())
        .or_else(|| deps["forge"].as_str())
        .or_else(|| deps["neoforge"].as_str())
        .map(|s| s.to_string());

    crate::console_log::append(
        app,
        format!("Creating instance `{name}` ({game_version} · {loader})…"),
        "info",
    );
    let inst = create_instance(
        name.clone(),
        game_version.clone(),
        loader.into(),
        loader_version,
        4096,
        None,
    )?;

    if loader != "vanilla" {
        crate::console_log::append(
            app,
            format!("Installing {loader} loader (this can take a few minutes)…"),
            "progress",
        );
        emit_progress(
            app,
            DownloadProgress {
                phase: "loader".into(),
                done: 0,
                total: 1,
                failed: 0,
                current_file: Some(loader.into()),
                bytes_per_sec: None,
                message: format!("Installing {loader}…"),
                active: true,
                ..Default::default()
            },
        );
        match crate::loaders::install_loader(inst.id.clone()) {
            Ok(_) => {
                crate::console_log::append(app, format!("{loader} loader ready."), "info");
            }
            Err(e) => {
                emit_idle(app, format!("Loader install failed: {e}"));
                return Err(format!("Loader install failed: {e}"));
            }
        }
    }

    let root = minecraft_dir(&inst.id)?;
    let mut jobs: Vec<(String, PathBuf)> = Vec::new();
    for file in index["files"].as_array().into_iter().flatten() {
        if !mrpack_file_for_client(file) {
            continue;
        }
        let path_in = file["path"].as_str().unwrap_or("").replace('\\', "/");
        if path_in.is_empty() || path_in.contains("..") {
            continue;
        }
        let url = file["downloads"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|u| u.as_str())
            .find(|u| !u.is_empty())
            .unwrap_or("");
        if url.is_empty() {
            continue;
        }
        let dest = root.join(Path::new(&path_in));
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).ok();
        }
        jobs.push((url.to_string(), dest));
    }

    crate::console_log::append(
        app,
        format!("Downloading {} modpack files…", jobs.len()),
        "progress",
    );
    let (ok, fail) = download_many_progress(jobs, app, "modpack-files", 0, None)?;
    crate::console_log::append(
        app,
        format!("Modpack files done — ok {ok}, fail {fail}"),
        if fail > 0 { "warn" } else { "info" },
    );
    if fail > 0 && ok == 0 {
        emit_idle(app, "Modpack file downloads all failed");
        return Err(format!(
            "Failed to download modpack files ({fail} failed). Check network / download mirror."
        ));
    }

    crate::console_log::append(app, "Extracting modpack overrides…", "progress");
    let mut overrides = 0usize;
    {
        let file = fs::File::open(&path_buf).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            let Some(rest) = name
                .strip_prefix("overrides/")
                .or_else(|| name.strip_prefix("client-overrides/"))
            else {
                continue;
            };
            if rest.is_empty() || rest.contains("..") {
                continue;
            }
            let dest = root.join(rest);
            if entry.is_dir() {
                fs::create_dir_all(&dest).ok();
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).ok();
                }
                let mut out = fs::File::create(&dest).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                overrides += 1;
            }
        }
    }

    emit_idle(
        app,
        format!("Installed modpack `{name}` — {ok} files, {overrides} overrides"),
    );
    crate::console_log::append(
        app,
        format!("Modpack `{name}` ready ({ok} files, {overrides} overrides)."),
        "info",
    );
    get_instance(&inst.id)
}

pub fn export_mrpack(instance_id: String, dest_path: String) -> Result<String, String> {
    let inst = get_instance(&instance_id)?;
    let mods = list_instance_mods(instance_id.clone())?;
    let mut files = Vec::new();
    for m in &mods {
        if !m.enabled {
            continue;
        }
        files.push(serde_json::json!({
            "path": format!("mods/{}", m.file_name),
            "downloads": [],
            "env": { "client": "required", "server": "optional" }
        }));
    }

    let mut deps = serde_json::Map::new();
    deps.insert("minecraft".into(), Value::String(inst.game_version.clone()));
    let loader_ver = inst.loader_version.clone().unwrap_or_else(|| "*".into());
    match inst.loader {
        LoaderKind::Fabric => {
            deps.insert("fabric-loader".into(), Value::String(loader_ver));
        }
        LoaderKind::Quilt => {
            deps.insert("quilt-loader".into(), Value::String(loader_ver));
        }
        LoaderKind::Forge => {
            deps.insert("forge".into(), Value::String(loader_ver));
        }
        LoaderKind::NeoForge => {
            deps.insert("neoforge".into(), Value::String(loader_ver));
        }
        LoaderKind::Vanilla => {}
    }

    let index = serde_json::json!({
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": "euml-export",
        "name": inst.name,
        "files": files,
        "dependencies": deps
    });

    let dest = PathBuf::from(&dest_path);
    let file = fs::File::create(&dest).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("modrinth.index.json", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(serde_json::to_string_pretty(&index).unwrap().as_bytes())
        .map_err(|e| e.to_string())?;
    zip.finish().map_err(|e| e.to_string())?;
    Ok(dest_path)
}

#[derive(Debug, serde::Serialize)]
pub struct ModrinthProjectDetails {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
    pub downloads: u64,
    pub followers: u64,
    pub project_type: String,
    pub modrinth_url: String,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub mcmod_url: String,
    pub curseforge_url: String,
    pub gallery: Vec<ModrinthGalleryImage>,
    pub versions: Vec<ModrinthVersion>,
}

pub fn get_modrinth_project(
    project_id: String,
    game_version: String,
    loader: String,
) -> Result<ModrinthProjectDetails, String> {
    let game_version = crate::models::normalize_game_version(&game_version);
    let loader_s = LoaderKind::from_str_loose(&loader).as_str().to_string();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(15))
        .user_agent("Northstar/1.2.3")
        .build()
        .map_err(|e| e.to_string())?;
    let data: Value = client
        .get(format!("https://api.modrinth.com/v2/project/{project_id}"))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let slug = data["slug"].as_str().unwrap_or(&project_id).to_string();
    let title = data["title"].as_str().unwrap_or(&slug).to_string();
    let project_type = data["project_type"].as_str().unwrap_or("mod").to_string();
    let categories = data["categories"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let mut versions =
        fetch_compatible_versions(&project_id, &game_version, &loader_s, &project_type, 40)
            .unwrap_or_default();
    enrich_version_deps(&mut versions);

    let mut gallery = data["gallery"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|g| {
            let url = g["url"].as_str()?.to_string();
            if url.is_empty() {
                return None;
            }
            Some(ModrinthGalleryImage {
                url,
                featured: g["featured"].as_bool().unwrap_or(false),
                title: g["title"].as_str().map(|s| s.to_string()),
            })
        })
        .collect::<Vec<_>>();
    gallery.sort_by(|a, b| b.featured.cmp(&a.featured));

    let q = urlencoding::encode(&title);
    Ok(ModrinthProjectDetails {
        project_id: data["id"].as_str().unwrap_or(&project_id).to_string(),
        slug: slug.clone(),
        title: title.clone(),
        description: data["description"].as_str().unwrap_or("").to_string(),
        body: data["body"].as_str().unwrap_or("").to_string(),
        icon_url: data["icon_url"].as_str().map(|s| s.to_string()),
        categories,
        downloads: data["downloads"].as_u64().unwrap_or(0),
        followers: data["followers"].as_u64().unwrap_or(0),
        project_type: project_type.clone(),
        modrinth_url: format!("https://modrinth.com/{project_type}/{slug}"),
        source_url: data["source_url"].as_str().map(|s| s.to_string()),
        issues_url: data["issues_url"].as_str().map(|s| s.to_string()),
        wiki_url: data["wiki_url"].as_str().map(|s| s.to_string()),
        discord_url: data["discord_url"].as_str().map(|s| s.to_string()),
        mcmod_url: format!("https://search.mcmod.cn/s?key={q}"),
        curseforge_url: format!(
            "https://www.curseforge.com/minecraft/search?page=1&pageSize=20&sortBy=relevancy&class=mod&search={q}"
        ),
        gallery,
        versions,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct ModUpdateResult {
    pub file_name: String,
    pub updated: bool,
    pub message: String,
}

/// Best-effort: for each jar, search Modrinth by filename stem and install newer compatible version.
pub fn update_instance_mods(instance_id: String) -> Result<Vec<ModUpdateResult>, String> {
    let inst = get_instance(&instance_id)?;
    let mods = list_instance_mods(instance_id.clone())?;
    let mut results = Vec::new();
    for m in mods {
        if !m.enabled {
            continue;
        }
        let stem = PathBuf::from(&m.file_name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| m.file_name.clone());
        let query = stem
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
            .filter(|t| t.len() >= 3)
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        if query.is_empty() {
            results.push(ModUpdateResult {
                file_name: m.file_name,
                updated: false,
                message: "Skipped (no search tokens)".into(),
            });
            continue;
        }
        match search_mods(
            query,
            inst.game_version.clone(),
            inst.loader.as_str().into(),
            None,
        ) {
            Ok(hits) => {
                let Some(hit) = hits.first() else {
                    results.push(ModUpdateResult {
                        file_name: m.file_name,
                        updated: false,
                        message: "No Modrinth match".into(),
                    });
                    continue;
                };
                let Some(ver) = hit.versions.first() else {
                    results.push(ModUpdateResult {
                        file_name: m.file_name,
                        updated: false,
                        message: "No compatible version".into(),
                    });
                    continue;
                };
                let primary = ver
                    .files
                    .iter()
                    .find(|f| f.primary)
                    .or_else(|| ver.files.first());
                if let Some(f) = primary {
                    if f.filename == m.file_name {
                        results.push(ModUpdateResult {
                            file_name: m.file_name,
                            updated: false,
                            message: "Already latest".into(),
                        });
                        continue;
                    }
                }
                match install_mod(
                    crate::app_handle::get(),
                    instance_id.clone(),
                    hit.project_id.clone(),
                    ver.id.clone(),
                ) {
                    Ok(entry) => {
                        // Remove old jar if different
                        if entry.file_name != m.file_name {
                            let _ = uninstall_mod(instance_id.clone(), m.file_name.clone());
                        }
                        results.push(ModUpdateResult {
                            file_name: entry.file_name,
                            updated: true,
                            message: format!("Updated to {}", ver.version_number),
                        });
                    }
                    Err(e) => results.push(ModUpdateResult {
                        file_name: m.file_name,
                        updated: false,
                        message: e,
                    }),
                }
            }
            Err(e) => results.push(ModUpdateResult {
                file_name: m.file_name,
                updated: false,
                message: e,
            }),
        }
    }
    Ok(results)
}
