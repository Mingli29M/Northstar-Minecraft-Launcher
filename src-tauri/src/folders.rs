use crate::models::InstanceFolder;
use crate::paths::instances_root;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

fn sanitize_folder_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Folder name is required".into());
    }
    if name == "." || name == ".." || name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("Invalid folder name".into());
    }
    if name.eq_ignore_ascii_case("folders.json") || name.starts_with('.') {
        return Err("Reserved folder name".into());
    }
    Ok(name.to_string())
}

/// List real directories under the instances root (dirs that are not themselves instances).
pub fn list_folders() -> Result<Vec<InstanceFolder>, String> {
    migrate_virtual_folders()?;
    let root = instances_root()?;
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Skip instance directories (contain instance.json)
        if path.join("instance.json").exists() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let created = entry
            .metadata()
            .ok()
            .and_then(|m| m.created().ok())
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| {
                        chrono::DateTime::<Utc>::from_timestamp(d.as_secs() as i64, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_else(|| Utc::now().to_rfc3339())
                    })
            })
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        out.push(InstanceFolder {
            id: name.clone(),
            name,
            created_at: created,
            path: path.to_string_lossy().to_string(),
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

/// One-time: turn old folders.json virtual folders into real directories.
fn migrate_virtual_folders() -> Result<(), String> {
    let root = instances_root()?;
    let legacy = root.join("folders.json");
    if !legacy.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&legacy).unwrap_or_default();
    if let Ok(folders) = serde_json::from_str::<Vec<serde_json::Value>>(&raw) {
        for f in folders {
            let name = f
                .get("name")
                .and_then(|n| n.as_str())
                .or_else(|| f.get("id").and_then(|n| n.as_str()))
                .unwrap_or("");
            if let Ok(safe) = sanitize_folder_name(name) {
                let dir = root.join(&safe);
                let _ = fs::create_dir_all(&dir);
                // Move instances that referenced this folder id/name into the dir
                let old_id = f.get("id").and_then(|n| n.as_str()).unwrap_or(name);
                if let Ok(instances) = crate::instances::list_instances_raw() {
                    for mut inst in instances {
                        let matches = inst.folder.as_deref() == Some(old_id)
                            || inst.folder.as_deref() == Some(name);
                        if matches {
                            let _ = crate::instances::move_instance_to_folder(&inst.id, Some(safe.clone()));
                            inst.folder = Some(safe.clone());
                        }
                    }
                }
            }
        }
    }
    let _ = fs::rename(&legacy, root.join("folders.json.bak"));
    Ok(())
}

pub fn create_folder(name: String) -> Result<InstanceFolder, String> {
    let name = sanitize_folder_name(&name)?;
    let path = instances_root()?.join(&name);
    if path.exists() {
        return Err(format!("Folder already exists on disk: {name}"));
    }
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(InstanceFolder {
        id: name.clone(),
        name: name.clone(),
        created_at: Utc::now().to_rfc3339(),
        path: path.to_string_lossy().to_string(),
    })
}

pub fn rename_folder(id: String, name: String) -> Result<Vec<InstanceFolder>, String> {
    let new_name = sanitize_folder_name(&name)?;
    let root = instances_root()?;
    let from = root.join(&id);
    let to = root.join(&new_name);
    if !from.is_dir() {
        return Err("Folder not found on disk".into());
    }
    if from.join("instance.json").exists() {
        return Err("Not a group folder".into());
    }
    if to.exists() {
        return Err(format!("Folder already exists: {new_name}"));
    }
    fs::rename(&from, &to).map_err(|e| e.to_string())?;
    // Update instance.json folder fields inside
    if let Ok(rd) = fs::read_dir(&to) {
        for entry in rd.flatten() {
            let meta = entry.path().join("instance.json");
            if !meta.exists() {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(&meta) {
                if let Ok(mut inst) = serde_json::from_str::<crate::models::Instance>(&raw) {
                    inst.folder = Some(new_name.clone());
                    let _ = fs::write(meta, serde_json::to_string_pretty(&inst).unwrap_or_default());
                }
            }
        }
    }
    list_folders()
}

pub fn delete_folder(id: String) -> Result<Vec<InstanceFolder>, String> {
    let root = instances_root()?;
    let dir = root.join(&id);
    if !dir.is_dir() || dir.join("instance.json").exists() {
        return Err("Folder not found".into());
    }
    // Move contained instances up to root first
    let entries: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    for child in entries {
        if child.join("instance.json").exists() {
            let name = child.file_name().map(|n| n.to_os_string()).ok_or("bad path")?;
            let dest = root.join(&name);
            if dest.exists() {
                return Err(format!(
                    "Cannot delete folder: conflict moving {}",
                    name.to_string_lossy()
                ));
            }
            fs::rename(&child, &dest).map_err(|e| e.to_string())?;
            let meta = dest.join("instance.json");
            if let Ok(raw) = fs::read_to_string(&meta) {
                if let Ok(mut inst) = serde_json::from_str::<crate::models::Instance>(&raw) {
                    inst.folder = None;
                    let _ = fs::write(meta, serde_json::to_string_pretty(&inst).unwrap_or_default());
                }
            }
        }
    }
    // Remove leftover empty-ish folder
    let _ = fs::remove_dir_all(&dir);
    list_folders()
}

pub fn folder_path(name: &str) -> Result<PathBuf, String> {
    let name = sanitize_folder_name(name)?;
    Ok(instances_root()?.join(name))
}

pub fn open_folder(name: String) -> Result<(), String> {
    let path = folder_path(&name)?;
    if !path.is_dir() {
        return Err("Folder not found".into());
    }
    open::that(path).map_err(|e| e.to_string())
}
