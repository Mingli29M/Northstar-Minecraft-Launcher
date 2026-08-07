use crate::models::{Instance, LoaderKind};
use crate::paths::{ensure_instance_dirs_at, instances_root};
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zip::ZipArchive;

pub fn list_instances() -> Result<Vec<Instance>, String> {
    list_instances_raw()
}

/// Scan instances root and real subfolders for instance.json files.
pub fn list_instances_raw() -> Result<Vec<Instance>, String> {
    let root = instances_root()?;
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    scan_dir_for_instances(&root, None, &mut out)?;
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() || path.join("instance.json").exists() {
            continue;
        }
        let folder_name = entry.file_name().to_string_lossy().to_string();
        if folder_name.starts_with('.') {
            continue;
        }
        scan_dir_for_instances(&path, Some(folder_name), &mut out)?;
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

fn scan_dir_for_instances(
    dir: &Path,
    folder: Option<String>,
    out: &mut Vec<Instance>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let meta = path.join("instance.json");
        if !meta.exists() {
            continue;
        }
        let raw = fs::read_to_string(&meta).map_err(|e| e.to_string())?;
        let mut inst: Instance = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        let mut dirty = false;
        if inst.folder != folder {
            inst.folder = folder.clone();
            dirty = true;
        }
        // Strong loader signals from profile / version folders only — never from mod jar names
        // (e.g. forgified-fabric-api must not flip Fabric → Forge).
        let mut profile_raw = String::new();
        let patch = path.join("patches").join("version.json");
        if let Ok(raw) = fs::read_to_string(&patch) {
            profile_raw.push_str(&raw);
        }
        let mut version_folders: Vec<String> = Vec::new();
        let versions_dir = path.join("minecraft").join("versions");
        if versions_dir.is_dir() {
            if let Ok(rd) = fs::read_dir(&versions_dir) {
                for e in rd.flatten().take(40) {
                    let name = e.file_name().to_string_lossy().to_string();
                    let vjson = e.path().join(format!("{name}.json"));
                    if let Ok(raw) = fs::read_to_string(&vjson) {
                        if profile_raw.is_empty() {
                            profile_raw = raw;
                        } else {
                            profile_raw.push('\n');
                            profile_raw.push_str(&raw);
                        }
                    }
                    version_folders.push(name);
                }
            }
        }
        let inferred = crate::models::resolve_loader(
            &inst.name,
            &inst.game_version,
            inst.loader.clone(),
            &profile_raw,
            &version_folders,
        );
        if inferred != inst.loader {
            inst.loader = inferred;
            dirty = true;
        }
        let normalized = crate::models::normalize_game_version(&inst.game_version);
        // Only rewrite when we extracted a real Mojang id. Pack names like
        // "Create：Complete" must not be "normalized" into a still-invalid token.
        if !normalized.is_empty() && normalized != inst.game_version {
            inst.game_version = normalized;
            dirty = true;
        } else if !crate::models::is_plausible_game_version(&inst.game_version) {
            // Auto-heal broken pack-name versions from files already on disk.
            if let Some(hit) = detect_from_instance_dir(&path) {
                inst.game_version = hit.game_version;
                if let Some(loader) = hit.loader {
                    inst.loader = loader;
                }
                if hit.loader_version.is_some() {
                    inst.loader_version = hit.loader_version;
                }
                dirty = true;
            }
        }
        if dirty {
            let _ = fs::write(&meta, serde_json::to_string_pretty(&inst).unwrap_or_default());
        }
        out.push(inst);
    }
    Ok(())
}

pub fn resolve_instance_dir(id: &str) -> Result<PathBuf, String> {
    let root = instances_root()?;
    let at_root = root.join(id);
    if at_root.join("instance.json").exists() {
        return Ok(at_root);
    }
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() || path.join("instance.json").exists() {
            continue;
        }
        let candidate = path.join(id);
        if candidate.join("instance.json").exists() {
            return Ok(candidate);
        }
    }
    // Fallback for brand-new creates before json is written
    Ok(at_root)
}

pub fn instance_dir_for(folder: Option<&str>, id: &str) -> Result<PathBuf, String> {
    let root = instances_root()?;
    Ok(match folder {
        Some(f) if !f.is_empty() && f != "root" => {
            let group = root.join(f);
            fs::create_dir_all(&group).map_err(|e| e.to_string())?;
            group.join(id)
        }
        _ => root.join(id),
    })
}

pub fn get_instance(id: &str) -> Result<Instance, String> {
    let path = resolve_instance_dir(id)?.join("instance.json");
    if !path.exists() {
        return Err(format!("Instance not found: {id}"));
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("Instance not found: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub fn save_instance(inst: &Instance) -> Result<(), String> {
    let dir = instance_dir_for(inst.folder.as_deref(), &inst.id)?;
    ensure_instance_dirs_at(&dir)?;
    let path = dir.join("instance.json");
    let raw = serde_json::to_string_pretty(inst).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

pub fn create_instance(
    name: String,
    game_version: String,
    loader: String,
    loader_version: Option<String>,
    memory_mb: u32,
    folder: Option<String>,
) -> Result<Instance, String> {
    let folder = normalize_folder(folder)?;
    let id = Uuid::new_v4().to_string();
    let mut loader_kind = LoaderKind::from_str_loose(&loader);
    loader_kind = crate::models::infer_loader(&name, &game_version, loader_kind);
    let game_version = crate::models::normalize_game_version(&game_version);
    if !crate::models::is_plausible_game_version(&game_version) {
        return Err(format!(
            "Invalid game version '{game_version}'. Use a Minecraft version like 1.21.1, not a pack or instance name."
        ));
    }
    let inst = Instance {
        id: id.clone(),
        name,
        game_version,
        loader: loader_kind,
        loader_version,
        java_path: None,
        memory_mb,
        jvm_args: String::new(),
        env_vars: String::new(),
        pre_command: String::new(),
        post_command: String::new(),
        created_at: Utc::now().to_rfc3339(),
        last_played: None,
        folder,
        icon_path: None,
    };
    save_instance(&inst)?;
    Ok(inst)
}

fn normalize_folder(folder: Option<String>) -> Result<Option<String>, String> {
    match folder {
        None => Ok(None),
        Some(f) if f.is_empty() || f == "root" => Ok(None),
        Some(f) => {
            let name = f.trim().to_string();
            if name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
                return Err("Invalid folder name".into());
            }
            let path = instances_root()?.join(&name);
            if !path.is_dir() {
                fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            }
            if path.join("instance.json").exists() {
                return Err("That path is an instance, not a group folder".into());
            }
            Ok(Some(name))
        }
    }
}

pub fn update_instance(instance: Instance) -> Result<Instance, String> {
    let existing = get_instance(&instance.id)?;
    let mut instance = instance;
    instance.folder = normalize_folder(instance.folder)?;
    let gv = crate::models::normalize_game_version(&instance.game_version);
    if !crate::models::is_plausible_game_version(&gv) {
        return Err(format!(
            "Invalid game version '{}'. Use a Minecraft version like 1.21.1, not a pack or instance name.",
            instance.game_version
        ));
    }
    instance.game_version = gv;
    if existing.folder != instance.folder {
        move_instance_to_folder(&instance.id, instance.folder.clone())?;
        // Re-save other fields after move
        let mut moved = get_instance(&instance.id)?;
        moved.name = instance.name;
        moved.game_version = instance.game_version;
        moved.loader = instance.loader;
        moved.loader_version = instance.loader_version;
        moved.java_path = instance.java_path;
        moved.memory_mb = instance.memory_mb;
        moved.jvm_args = instance.jvm_args;
        moved.env_vars = instance.env_vars;
        moved.pre_command = instance.pre_command;
        moved.post_command = instance.post_command;
        moved.last_played = instance.last_played;
        save_instance(&moved)?;
        return Ok(moved);
    }
    save_instance(&instance)?;
    Ok(instance)
}

pub fn move_instance(id: String, folder: Option<String>) -> Result<Instance, String> {
    move_instance_to_folder(&id, folder)?;
    get_instance(&id)
}

pub fn move_instance_to_folder(id: &str, folder: Option<String>) -> Result<(), String> {
    let folder = normalize_folder(folder)?;
    let from = resolve_instance_dir(id)?;
    if !from.join("instance.json").exists() {
        return Err(format!("Instance not found: {id}"));
    }
    let to = instance_dir_for(folder.as_deref(), id)?;
    if same_path(&from, &to) {
        let mut inst = get_instance(id)?;
        inst.folder = folder;
        return save_instance(&inst);
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if to.exists() {
        return Err("Destination already exists".into());
    }
    fs::rename(&from, &to).map_err(|e| e.to_string())?;
    let meta = to.join("instance.json");
    let raw = fs::read_to_string(&meta).map_err(|e| e.to_string())?;
    let mut inst: Instance = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    inst.folder = folder;
    fs::write(
        meta,
        serde_json::to_string_pretty(&inst).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn same_path(a: &Path, b: &Path) -> bool {
    let ca = fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let cb = fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    ca == cb
}

pub fn delete_instance(id: &str) -> Result<(), String> {
    let dir = resolve_instance_dir(id)?;
    if dir.join("instance.json").exists() {
        fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn open_instance_folder(id: &str) -> Result<(), String> {
    let dir = resolve_instance_dir(id)?;
    open::that(&dir).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
struct DetectHit {
    game_version: String,
    loader: Option<LoaderKind>,
    loader_version: Option<String>,
    source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedGameVersion {
    pub game_version: String,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
    pub source: String,
    pub applied: bool,
}

/// If mods/indexes say a patch refinement of the stored version (e.g. `1.21` → `1.21.1`),
/// update the instance and clear a mismatched Forge/NeoForge profile so it reinstalls.
pub fn refine_game_version_from_files(id: &str) -> Result<Option<String>, String> {
    let dir = resolve_instance_dir(id)?;
    let mut inst = get_instance(id)?;
    let Some(hit) = detect_from_instance_dir(&dir) else {
        return Ok(None);
    };
    if hit.game_version == inst.game_version {
        return Ok(None);
    }
    let refinement = hit
        .game_version
        .starts_with(&format!("{}.", inst.game_version));
    let current_bad = !crate::models::is_plausible_game_version(&inst.game_version);
    if !refinement && !current_bad {
        return Ok(None);
    }
    inst.game_version = hit.game_version.clone();
    if let Some(loader) = hit.loader.clone() {
        inst.loader = loader;
    }
    if matches!(inst.loader, LoaderKind::Forge | LoaderKind::NeoForge) {
        inst.loader_version = None;
        let _ = fs::remove_file(dir.join("patches").join("version.json"));
    }
    save_instance(&inst)?;
    Ok(Some(hit.game_version))
}

/// Probe instance files for a real Mojang version id (and loader hints).
pub fn detect_instance_game_version(
    id: &str,
    apply: bool,
) -> Result<DetectedGameVersion, String> {
    let dir = resolve_instance_dir(id)?;
    if !dir.join("instance.json").exists() {
        return Err(format!("Instance not found: {id}"));
    }
    let hit = detect_from_instance_dir(&dir).ok_or_else(|| {
        "Could not detect a Minecraft version from this instance's files. Set it manually (e.g. 1.21.1).".to_string()
    })?;

    let mut applied = false;
    if apply {
        let mut inst = get_instance(id)?;
        inst.game_version = hit.game_version.clone();
        if let Some(loader) = hit.loader.clone() {
            inst.loader = loader;
        }
        if hit.loader_version.is_some() {
            inst.loader_version = hit.loader_version.clone();
        }
        // Clear poisoned "recommended" stub versions so the installer resolves a real build.
        if matches!(inst.loader, LoaderKind::Forge | LoaderKind::NeoForge) {
            let bad = inst
                .loader_version
                .as_deref()
                .is_some_and(|v| v.eq_ignore_ascii_case("recommended") || v.eq_ignore_ascii_case("latest"));
            if bad {
                inst.loader_version = None;
            }
        }
        save_instance(&inst)?;
        if inst.loader != LoaderKind::Vanilla {
            crate::loaders::install_loader(inst.id.clone())?;
        }
        applied = true;
    }

    Ok(DetectedGameVersion {
        game_version: hit.game_version,
        loader: hit.loader.map(|l| l.as_str().to_string()),
        loader_version: hit.loader_version,
        source: hit.source,
        applied,
    })
}

fn detect_from_instance_dir(root: &Path) -> Option<DetectHit> {
    // Modpack indexes / mod jars first. Loader profiles we wrote ourselves often
    // carry a wrong inheritsFrom (e.g. 1.21 vs 1.21.1) and must not win.
    for rel in [
        "modrinth.index.json",
        "minecraft/modrinth.index.json",
        "mmc-pack.json",
    ] {
        let path = root.join(rel);
        if let Some(hit) = read_index_hit(&path, rel) {
            return Some(hit);
        }
    }

    if let Some(hit) = detect_from_mod_jars(&root.join("minecraft").join("mods")) {
        return Some(hit);
    }

    let versions = root.join("minecraft").join("versions");
    if versions.is_dir() {
        if let Ok(rd) = fs::read_dir(&versions) {
            let mut dirs: Vec<_> = rd.flatten().filter(|e| e.path().is_dir()).collect();
            dirs.sort_by_key(|e| e.file_name());
            for entry in dirs {
                let name = entry.file_name().to_string_lossy().to_string();
                let json = entry.path().join(format!("{name}.json"));
                if let Some(hit) = read_profile_hit(&json, &format!("minecraft/versions/{name}")) {
                    return Some(hit);
                }
                let from_name = crate::models::normalize_game_version(&name);
                if crate::models::is_plausible_game_version(&from_name) {
                    return Some(DetectHit {
                        game_version: from_name,
                        loader: loader_hint_from_folder(&name),
                        loader_version: None,
                        source: format!("version folder `{name}`"),
                    });
                }
            }
        }
    }

    if let Some(hit) =
        read_profile_hit(&root.join("patches").join("version.json"), "patches/version.json")
    {
        return Some(hit);
    }

    None
}

fn read_profile_hit(path: &Path, source: &str) -> Option<DetectHit> {
    // Our old Forge/NeoForge stub wrote inheritsFrom from a wrong game_version and
    // poisoned detection — never trust empty stub profiles.
    if crate::loaders::is_stub_profile(path) {
        return None;
    }
    let raw = fs::read_to_string(path).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    let inherits = v
        .get("inheritsFrom")
        .and_then(|x| x.as_str())
        .map(|s| crate::models::normalize_game_version(s))
        .filter(|s| crate::models::is_plausible_game_version(s));
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .map(|s| crate::models::normalize_game_version(s))
        .filter(|s| crate::models::is_plausible_game_version(s));
    let game_version = inherits.or(id)?;
    let loader = crate::models::loader_from_profile(&raw, &[]);
    let loader_version = v
        .pointer("/arguments/game")
        .and_then(|g| g.as_array())
        .and_then(|args| {
            let mut it = args.iter().filter_map(|a| a.as_str());
            while let Some(a) = it.next() {
                if a == "--fml.neoForgeVersion" || a == "--fml.forgeVersion" {
                    return it.next().map(|s| s.to_string());
                }
            }
            None
        });
    Some(DetectHit {
        game_version,
        loader,
        loader_version,
        source: source.to_string(),
    })
}

fn read_index_hit(path: &Path, source: &str) -> Option<DetectHit> {
    let raw = fs::read_to_string(path).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;

    // Modrinth index
    if let Some(mc) = v
        .pointer("/dependencies/minecraft")
        .and_then(|x| x.as_str())
        .map(|s| crate::models::normalize_game_version(s))
        .filter(|s| crate::models::is_plausible_game_version(s))
    {
        let loader = if v.pointer("/dependencies/fabric-loader").is_some() {
            Some(LoaderKind::Fabric)
        } else if v.pointer("/dependencies/quilt-loader").is_some() {
            Some(LoaderKind::Quilt)
        } else if v.pointer("/dependencies/neoforge").is_some() {
            Some(LoaderKind::NeoForge)
        } else if v.pointer("/dependencies/forge").is_some() {
            Some(LoaderKind::Forge)
        } else {
            None
        };
        let loader_version = v
            .pointer("/dependencies/fabric-loader")
            .or_else(|| v.pointer("/dependencies/quilt-loader"))
            .or_else(|| v.pointer("/dependencies/neoforge"))
            .or_else(|| v.pointer("/dependencies/forge"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());
        return Some(DetectHit {
            game_version: mc,
            loader,
            loader_version,
            source: source.to_string(),
        });
    }

    // MultiMC / Prism mmc-pack.json
    if let Some(components) = v.get("components").and_then(|c| c.as_array()) {
        let mut game_version = None;
        let mut loader = None;
        let mut loader_version = None;
        for c in components {
            let uid = c.get("uid").and_then(|u| u.as_str()).unwrap_or("");
            let ver = c
                .get("version")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());
            match uid {
                "net.minecraft" => {
                    if let Some(ver) = ver {
                        let n = crate::models::normalize_game_version(&ver);
                        if crate::models::is_plausible_game_version(&n) {
                            game_version = Some(n);
                        }
                    }
                }
                "net.fabricmc.fabric-loader" => {
                    loader = Some(LoaderKind::Fabric);
                    loader_version = ver;
                }
                "org.quiltmc.quilt-loader" => {
                    loader = Some(LoaderKind::Quilt);
                    loader_version = ver;
                }
                "net.minecraftforge" => {
                    loader = Some(LoaderKind::Forge);
                    loader_version = ver;
                }
                "net.neoforged" => {
                    loader = Some(LoaderKind::NeoForge);
                    loader_version = ver;
                }
                _ => {}
            }
        }
        if let Some(game_version) = game_version {
            return Some(DetectHit {
                game_version,
                loader,
                loader_version,
                source: source.to_string(),
            });
        }
    }

    None
}

fn loader_hint_from_folder(name: &str) -> Option<LoaderKind> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("neoforge") {
        Some(LoaderKind::NeoForge)
    } else if lower.contains("forge") {
        Some(LoaderKind::Forge)
    } else if lower.contains("quilt") {
        Some(LoaderKind::Quilt)
    } else if lower.contains("fabric") {
        Some(LoaderKind::Fabric)
    } else {
        None
    }
}

/// Lightweight consensus from a sample of mod jars (Fabric/Quilt/Forge/NeoForge metadata).
fn detect_from_mod_jars(mods_dir: &Path) -> Option<DetectHit> {
    if !mods_dir.is_dir() {
        return None;
    }
    let mut votes: HashMap<String, usize> = HashMap::new();
    let mut jars: Vec<PathBuf> = fs::read_dir(mods_dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.ends_with(".jar") && !name.ends_with(".disabled")
        })
        .collect();
    // Prefer jars whose names already look versioned (e.g. create-1.21.1-…).
    jars.sort_by(|a, b| {
        let an = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let bn = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let ascore = if an.contains("1.") { 0 } else { 1 };
        let bscore = if bn.contains("1.") { 0 } else { 1 };
        ascore.cmp(&bscore).then_with(|| an.cmp(bn))
    });
    for path in jars.into_iter().take(48) {
        if let Some(ver) = minecraft_dep_from_jar(&path) {
            *votes.entry(ver).or_default() += 2;
        } else if let Some(ver) = minecraft_from_jar_name(
            path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
        ) {
            *votes.entry(ver).or_default() += 1;
        }
    }
    // Prefer higher vote count, then more specific ids (1.21.1 over 1.21).
    let (game_version, count) = votes.into_iter().max_by(|(va, ca), (vb, cb)| {
        ca.cmp(cb).then_with(|| version_specificity(va).cmp(&version_specificity(vb)))
    })?;
    if count < 1 || !crate::models::is_plausible_game_version(&game_version) {
        return None;
    }
    Some(DetectHit {
        game_version,
        loader: None,
        loader_version: None,
        source: format!("mod jar metadata ({count} vote(s))"),
    })
}

fn version_specificity(v: &str) -> usize {
    v.bytes().filter(|b| *b == b'.').count().saturating_mul(10) + v.len()
}

fn minecraft_dep_from_jar(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;
    for entry_name in ["fabric.mod.json", "quilt.mod.json"] {
        if let Ok(mut f) = archive.by_name(entry_name) {
            let mut raw = String::new();
            if f.read_to_string(&mut raw).is_ok() {
                if let Some(v) = minecraft_from_fabric_json(&raw) {
                    return Some(v);
                }
            }
        }
    }
    for entry_name in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Ok(mut f) = archive.by_name(entry_name) {
            let mut raw = String::new();
            if f.read_to_string(&mut raw).is_ok() {
                if let Some(v) = minecraft_from_mods_toml(&raw) {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn minecraft_from_fabric_json(raw: &str) -> Option<String> {
    let v: Value = serde_json::from_str(raw).ok()?;
    let dep = v
        .pointer("/depends/minecraft")
        .or_else(|| v.pointer("/depends/Minecraft"))?;
    let s = match dep {
        Value::String(s) => s.as_str(),
        Value::Array(arr) => arr.first()?.as_str()?,
        _ => return None,
    };
    best_version_in_text(s)
}

fn minecraft_from_mods_toml(raw: &str) -> Option<String> {
    // Prefer the minecraft dependency block's versionRange.
    let lower = raw.to_ascii_lowercase();
    if let Some(idx) = lower.find("modid=\"minecraft\"").or_else(|| lower.find("modid = \"minecraft\""))
    {
        let window = &raw[idx..raw.len().min(idx + 240)];
        if let Some(v) = best_version_in_text(window) {
            return Some(v);
        }
    }
    best_version_in_text(raw)
}

fn minecraft_from_jar_name(name: &str) -> Option<String> {
    best_version_in_text(name)
}

/// Pick the most specific Minecraft release id in a string.
/// Only accepts modern `1.x` ids so mod versions like `6.0.10` are ignored.
fn best_version_in_text(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?i)\b1\.\d+(?:\.\d+)?(?:-(?:pre|rc)\.?\d*)?\b").ok()?;
    let mut best: Option<String> = None;
    for m in re.find_iter(s) {
        let n = crate::models::normalize_game_version(m.as_str());
        if !crate::models::is_plausible_game_version(&n) || !n.starts_with("1.") {
            continue;
        }
        if best
            .as_ref()
            .map(|b| version_specificity(&n) > version_specificity(b))
            .unwrap_or(true)
        {
            best = Some(n);
        }
    }
    best
}
