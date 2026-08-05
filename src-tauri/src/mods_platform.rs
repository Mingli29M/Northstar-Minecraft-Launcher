use crate::instances::{create_instance, get_instance};
use crate::models::{LoaderKind, ModEntry};
use crate::paths::minecraft_dir;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
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

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub version_number: String,
    pub files: Vec<ModrinthFile>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
}

pub fn search_mods(
    query: String,
    game_version: String,
    loader: String,
    categories: Option<Vec<String>>,
) -> Result<Vec<ModrinthHit>, String> {
    search_modrinth_projects(&query, &game_version, &loader, "mod", categories.as_deref())
}

/// Search Modrinth for mods / resource packs / shaders / datapacks.
pub fn search_content(
    query: String,
    game_version: String,
    loader: String,
    project_type: String,
    categories: Option<Vec<String>>,
) -> Result<Vec<ModrinthHit>, String> {
    let pt = match project_type.as_str() {
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
    // Only filter by loader for mods — packs/shaders are loader-agnostic.
    if project_type == "mod" {
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
        .header("User-Agent", "Northstar/1.1.2")
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
            let versions = fetch_compatible_versions(project_id, &game_version, &loader, project_type)
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
    if project_type == "mod" && loader_l != "vanilla" {
        let loaders_json = if loader_l == "quilt" {
            "[\"quilt\",\"fabric\"]".to_string()
        } else {
            format!("[\"{loader_l}\"]")
        };
        url.push_str(&format!("&loaders={}", urlencoding::encode(&loaders_json)));
    }
    let data: Value = client
        .get(&url)
        .header("User-Agent", "Northstar/1.1.2")
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
        if project_type == "mod"
            && loader_l != "vanilla"
            && !loaders.iter().any(|l| l.eq_ignore_ascii_case(&loader_l))
        {
            if !(loader_l == "quilt" && loaders.iter().any(|l| l == "fabric")) {
                continue;
            }
        }
        let files = v["files"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|f| ModrinthFile {
                url: f["url"].as_str().unwrap_or("").to_string(),
                filename: f["filename"].as_str().unwrap_or("file.zip").to_string(),
                primary: f["primary"].as_bool().unwrap_or(false),
            })
            .collect::<Vec<_>>();
        if files.is_empty() {
            continue;
        }
        out.push(ModrinthVersion {
            id: v["id"].as_str().unwrap_or("").to_string(),
            version_number: v["version_number"].as_str().unwrap_or("").to_string(),
            files,
        });
        if out.len() >= 3 {
            break;
        }
    }
    Ok(out)
}

fn download_modrinth_version_file(
    version_id: &str,
    dest_dir: &PathBuf,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let data: Value = client
        .get(format!("https://api.modrinth.com/v2/version/{version_id}"))
        .header("User-Agent", "Northstar/1.1.2")
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let file = data["files"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|f| f["primary"].as_bool().unwrap_or(false))
        .or_else(|| data["files"].as_array().and_then(|a| a.first()))
        .ok_or("No file on version")?;

    let url = file["url"].as_str().ok_or("Missing url")?;
    let filename = file["filename"].as_str().unwrap_or("download.bin");
    fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    let dest = dest_dir.join(filename);
    let bytes = client
        .get(url)
        .header("User-Agent", "Northstar/1.1.2")
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;
    fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
    Ok(filename.to_string())
}

pub fn install_mod(instance_id: String, _project_id: String, version_id: String) -> Result<ModEntry, String> {
    let dest_dir = minecraft_dir(&instance_id)?.join("mods");
    let filename = download_modrinth_version_file(&version_id, &dest_dir)?;
    let path = dest_dir.join(&filename);
    Ok(ModEntry {
        file_name: filename,
        enabled: true,
        path: path.to_string_lossy().to_string(),
        icon_path: crate::icons::icon_for_mod_jar(&path),
    })
}

/// Install a Modrinth file into resourcepacks / shaderpacks / datapacks (world).
pub fn install_content_from_modrinth(
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

    if kind_norm == "datapacks" {
        let world = world_name.ok_or("Select a world for datapack install")?;
        let dest_dir = minecraft_dir(&instance_id)?
            .join("saves")
            .join(&world)
            .join("datapacks");
        if !minecraft_dir(&instance_id)?.join("saves").join(&world).exists() {
            return Err("World not found".into());
        }
        let filename = download_modrinth_version_file(&version_id, &dest_dir)?;
        let path = dest_dir.join(&filename);
        return Ok(crate::models::ContentItem {
            name: filename,
            path: path.to_string_lossy().to_string(),
            kind: "datapacks".into(),
            icon_path: crate::icons::icon_for_pack(&path),
        });
    }

    let dest_dir = minecraft_dir(&instance_id)?.join(kind_norm);
    let filename = download_modrinth_version_file(&version_id, &dest_dir)?;
    let path = dest_dir.join(&filename);
    Ok(crate::models::ContentItem {
        name: filename,
        path: path.to_string_lossy().to_string(),
        kind: kind_norm.into(),
        icon_path: crate::icons::icon_for_pack(&path),
    })
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

pub fn import_mrpack(path: String) -> Result<crate::models::Instance, String> {
    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut index_raw = String::new();
    {
        let mut index = archive
            .by_name("modrinth.index.json")
            .map_err(|e| format!("Not an mrpack: {e}"))?;
        std::io::Read::read_to_string(&mut index, &mut index_raw).map_err(|e| e.to_string())?;
    }
    let index: Value = serde_json::from_str(&index_raw).map_err(|e| e.to_string())?;
    let name = index["name"].as_str().unwrap_or("Imported Pack").to_string();
    let deps = &index["dependencies"];
    let game_version = deps["minecraft"].as_str().unwrap_or("1.21.1").to_string();
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

    let inst = create_instance(name, game_version.clone(), loader.into(), loader_version, 4096, None)?;
    if loader != "vanilla" {
        let _ = crate::loaders::install_loader(inst.id.clone());
    }

    let client = reqwest::blocking::Client::new();
    let mods_dir = minecraft_dir(&inst.id)?.join("mods");
    for file in index["files"].as_array().into_iter().flatten() {
        let path_in = file["path"].as_str().unwrap_or("");
        let downloads = file["downloads"].as_array();
        let url = downloads
            .and_then(|a| a.first())
            .and_then(|u| u.as_str())
            .unwrap_or("");
        if url.is_empty() {
            continue;
        }
        let dest = minecraft_dir(&inst.id)?.join(path_in);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(bytes) = client.get(url).send().and_then(|r| r.bytes()) {
            let _ = fs::write(dest, bytes);
        } else {
            // fallback filename into mods
            let fname = PathBuf::from(path_in)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "mod.jar".into());
            if let Ok(bytes) = client.get(url).send().and_then(|r| r.bytes()) {
                let _ = fs::write(mods_dir.join(fname), bytes);
            }
        }
    }

    // Extract overrides/
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        if let Some(rest) = name.strip_prefix("overrides/") {
            if rest.is_empty() {
                continue;
            }
            let dest = minecraft_dir(&inst.id)?.join(rest);
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
    }

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
        .user_agent("Northstar/1.1.2")
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
    let versions =
        fetch_compatible_versions(&project_id, &game_version, &loader_s, &project_type).unwrap_or_default();

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
                match install_mod(instance_id.clone(), hit.project_id.clone(), ver.id.clone()) {
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
