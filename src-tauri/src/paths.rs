use crate::models::LauncherSettings;
use std::fs;
use std::path::{Path, PathBuf};

pub fn app_root() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "Could not resolve data directory".to_string())?;
    let root = base.join("euml");
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root)
}

pub fn settings_path() -> Result<PathBuf, String> {
    Ok(app_root()?.join("settings.json"))
}

pub fn accounts_path() -> Result<PathBuf, String> {
    Ok(app_root()?.join("accounts.json"))
}

pub fn favorites_path() -> Result<PathBuf, String> {
    Ok(app_root()?.join("favorites.json"))
}

pub fn dedicated_root() -> Result<PathBuf, String> {
    let settings = load_settings()?;
    let root = if let Some(custom) = settings.dedicated_path.filter(|s| !s.trim().is_empty()) {
        PathBuf::from(custom)
    } else {
        app_root()?.join("dedicated")
    };
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root)
}

/// `{dedicated}/{id}` — returns Ok even if missing (caller checks host.json).
pub fn dedicated_dir(id: &str) -> Result<PathBuf, String> {
    if id.trim().is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
    {
        return Err("Invalid dedicated server id".into());
    }
    Ok(dedicated_root()?.join(id))
}

pub fn dedicated_runtime(id: &str) -> Result<PathBuf, String> {
    Ok(dedicated_dir(id)?.join("runtime"))
}

pub fn meta_dir() -> Result<PathBuf, String> {
    let p = app_root()?.join("meta");
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

pub fn load_settings() -> Result<LauncherSettings, String> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(LauncherSettings::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub fn save_settings(settings: &LauncherSettings) -> Result<(), String> {
    let path = settings_path()?;
    let raw = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

pub fn instances_root() -> Result<PathBuf, String> {
    let settings = load_settings()?;
    let root = if let Some(custom) = settings.instances_path {
        PathBuf::from(custom)
    } else {
        app_root()?.join("instances")
    };
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root)
}

/// Resolve instance directory by searching root + group folders.
pub fn instance_dir(id: &str) -> Result<PathBuf, String> {
    crate::instances::resolve_instance_dir(id)
}

pub fn minecraft_dir(id: &str) -> Result<PathBuf, String> {
    let p = instance_dir(id)?.join("minecraft");
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

pub fn ensure_instance_dirs(id: &str) -> Result<PathBuf, String> {
    let root = instance_dir(id)?;
    ensure_instance_dirs_at(&root)?;
    Ok(root)
}

pub fn ensure_instance_dirs_at(root: &Path) -> Result<(), String> {
    let mc = root.join("minecraft");
    for sub in [
        "mods",
        "config",
        "saves",
        "resourcepacks",
        "shaderpacks",
        "datapacks",
        "screenshots",
        "logs",
    ] {
        fs::create_dir_all(mc.join(sub)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
