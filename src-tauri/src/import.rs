use crate::instances::{create_instance, get_instance};
use crate::models::LoaderKind;
use crate::paths::{instance_dir, minecraft_dir};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const SKIP_NAMES: &[&str] = &[
    "instance.json",
    "mmc-pack.json",
    "instance.cfg",
    "folders.json",
    ".cache",
];

pub fn import_foreign_instance(
    path: String,
    folder: Option<String>,
) -> Result<crate::models::Instance, String> {
    let root = PathBuf::from(&path);
    if !root.is_dir() {
        return Err("Path is not a directory".into());
    }
    import_one_instance(&root, folder)
}

/// Import one instance folder, or a parent directory containing many instances.
pub fn import_instance_folder(
    path: String,
    folder: Option<String>,
) -> Result<Vec<crate::models::Instance>, String> {
    let root = PathBuf::from(&path);
    if !root.is_dir() {
        return Err("Path is not a directory".into());
    }

    if looks_like_instance_root(&root) {
        let mut out = Vec::new();
        for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let child = entry.path();
            if child.is_dir() && looks_like_instance(&child) {
                match import_one_instance(&child, folder.clone()) {
                    Ok(inst) => out.push(inst),
                    Err(e) => return Err(format!("Failed importing {}: {e}", child.display())),
                }
            }
        }
        if out.is_empty() {
            return Err("No importable instances found in this folder".into());
        }
        return Ok(out);
    }

    if looks_like_instance(&root) {
        return Ok(vec![import_one_instance(&root, folder)?]);
    }

    Err(
        "Not a recognizable Minecraft instance folder. Pick a Prism/MultiMC instance, \
         a folder with .minecraft/minecraft, or a directory containing multiple instances."
            .into(),
    )
}

fn looks_like_instance(path: &Path) -> bool {
    path.join("instance.json").exists()
        || path.join("mmc-pack.json").exists()
        || path.join("instance.cfg").exists()
        || path.join(".minecraft").is_dir()
        || path.join("minecraft").is_dir()
        || path.join("mods").is_dir()
        || path.join("saves").is_dir()
        || path.join("versions").is_dir()
}

fn looks_like_instance_root(path: &Path) -> bool {
    if looks_like_instance(path) && !path.join("mmc-pack.json").exists() {
        // A single Prism instance has mmc-pack; a pack of instances usually doesn't look like one instance
        // If it has .minecraft at top, it's one instance.
        if path.join(".minecraft").is_dir() || path.join("minecraft").is_dir() || path.join("instance.json").exists()
        {
            return false;
        }
    }
    if looks_like_instance(path)
        && (path.join(".minecraft").is_dir()
            || path.join("minecraft").is_dir()
            || path.join("instance.json").exists()
            || path.join("mmc-pack.json").exists())
    {
        return false;
    }
    fs::read_dir(path)
        .ok()
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().is_dir() && looks_like_instance(&e.path()))
                .count()
                >= 1
        })
        .unwrap_or(false)
}

fn import_one_instance(
    root: &Path,
    folder: Option<String>,
) -> Result<crate::models::Instance, String> {
    let root_canon = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());

    // Already registered EUML instance inside our tree — just reassign folder
    if root.join("instance.json").exists() {
        if let Ok(raw) = fs::read_to_string(root.join("instance.json")) {
            if let Ok(existing) = serde_json::from_str::<crate::models::Instance>(&raw) {
                if let Ok(own) = instance_dir(&existing.id) {
                    let own_canon = fs::canonicalize(&own).unwrap_or(own);
                    if own_canon == root_canon {
                        return crate::instances::move_instance(existing.id, folder);
                    }
                }
            }
        }
    }

    let name = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Imported".into());

    let mut game_version = "1.21.1".to_string();
    let mut loader = LoaderKind::Vanilla;
    let mut loader_version = None;

    let mmc = root.join("mmc-pack.json");
    if mmc.exists() {
        if let Ok(v) =
            serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&mmc).unwrap_or_default())
        {
            if let Some(components) = v.get("components").and_then(|c| c.as_array()) {
                for c in components {
                    let uid = c.get("uid").and_then(|u| u.as_str()).unwrap_or("");
                    let ver = c.get("version").and_then(|u| u.as_str()).map(|s| s.to_string());
                    match uid {
                        "net.minecraft" => {
                            if let Some(ver) = ver {
                                game_version = ver;
                            }
                        }
                        "net.fabricmc.fabric-loader" => {
                            loader = LoaderKind::Fabric;
                            loader_version = ver;
                        }
                        "org.quiltmc.quilt-loader" => {
                            loader = LoaderKind::Quilt;
                            loader_version = ver;
                        }
                        "net.minecraftforge" => {
                            loader = LoaderKind::Forge;
                            loader_version = ver;
                        }
                        "net.neoforged" => {
                            loader = LoaderKind::NeoForge;
                            loader_version = ver;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let version_json = root.join(format!("{name}.json"));
    if version_json.exists() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(&version_json).unwrap_or_default(),
        ) {
            if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                game_version = id.to_string();
            }
        }
    }

    // EUML instance.json outside our tree
    if root.join("instance.json").exists() {
        if let Ok(existing) =
            serde_json::from_str::<crate::models::Instance>(&fs::read_to_string(root.join("instance.json")).unwrap_or_default())
        {
            game_version = existing.game_version;
            loader = existing.loader;
            loader_version = existing.loader_version;
        }
    }

    let display_name = {
        let cfg = root.join("instance.cfg");
        if cfg.exists() {
            fs::read_to_string(&cfg)
                .ok()
                .and_then(|raw| {
                    raw.lines()
                        .find_map(|l| l.strip_prefix("name=").map(|s| s.to_string()))
                })
                .unwrap_or_else(|| name.clone())
        } else if root.join("instance.json").exists() {
            serde_json::from_str::<crate::models::Instance>(
                &fs::read_to_string(root.join("instance.json")).unwrap_or_default(),
            )
            .map(|i| i.name)
            .unwrap_or(name)
        } else {
            name
        }
    };

    let inst = create_instance(
        display_name,
        game_version,
        loader.as_str().to_string(),
        loader_version,
        4096,
        folder,
    )?;

    let src_mc = if root.join(".minecraft").is_dir() {
        root.join(".minecraft")
    } else if root.join("minecraft").is_dir() {
        root.join("minecraft")
    } else {
        root.to_path_buf()
    };

    let dest_mc = minecraft_dir(&inst.id)?;
    copy_game_data(&src_mc, &dest_mc)?;

    // If source was an EUML tree, also copy non-minecraft bits we care about are already in minecraft
    get_instance(&inst.id)
}

fn copy_game_data(src: &Path, dest: &Path) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = match entry.path().strip_prefix(src) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        // Skip launcher metadata files at any depth's top name
        if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
            if SKIP_NAMES.iter().any(|s| s.eq_ignore_ascii_case(name)) {
                continue;
            }
        }
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
