use crate::paths::favorites_path;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteEntry {
    /// Composite id: `instance:{uuid}`, `modrinth:{project_id}`, `server:{ip}`, `mcversion:{id}`
    pub id: String,
    pub kind: String,
    pub label: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub added_at: String,
}

fn load() -> Result<Vec<FavoriteEntry>, String> {
    let path = favorites_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save(list: &[FavoriteEntry]) -> Result<(), String> {
    let path = favorites_path()?;
    let raw = serde_json::to_string_pretty(list).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

pub fn list_favorites() -> Result<Vec<FavoriteEntry>, String> {
    load()
}

pub fn toggle_favorite(
    id: String,
    kind: String,
    label: String,
    subtitle: Option<String>,
    icon_url: Option<String>,
) -> Result<Vec<FavoriteEntry>, String> {
    let mut list = load()?;
    if let Some(pos) = list.iter().position(|f| f.id == id) {
        list.remove(pos);
    } else {
        list.insert(
            0,
            FavoriteEntry {
                id,
                kind,
                label,
                subtitle,
                icon_url,
                added_at: Utc::now().to_rfc3339(),
            },
        );
    }
    save(&list)?;
    Ok(list)
}

pub fn remove_favorite(id: String) -> Result<Vec<FavoriteEntry>, String> {
    let mut list = load()?;
    list.retain(|f| f.id != id);
    save(&list)?;
    Ok(list)
}
