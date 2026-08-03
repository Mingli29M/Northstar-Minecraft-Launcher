use crate::models::{Instance, LoaderKind};
use crate::paths::{ensure_instance_dirs_at, instances_root};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

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
        if normalized != inst.game_version {
            inst.game_version = normalized;
            dirty = true;
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
    if game_version.is_empty() || !game_version.chars().next().unwrap_or('x').is_ascii_digit() {
        return Err(format!(
            "Invalid game version '{game_version}'. Use a Minecraft version like 1.21.1, not the instance name."
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
