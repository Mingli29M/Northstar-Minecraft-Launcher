use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LoaderKind {
    Vanilla,
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}

impl LoaderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoaderKind::Vanilla => "vanilla",
            LoaderKind::Fabric => "fabric",
            LoaderKind::Quilt => "quilt",
            LoaderKind::Forge => "forge",
            LoaderKind::NeoForge => "neoforge",
        }
    }

    /// Parse a short label (instance name, version id, folder name).
    /// Fabric/Quilt beat bare "forge" so jars like `forgified-fabric-api` never win.
    pub fn from_label(s: &str) -> Option<Self> {
        let lower = s.to_lowercase();
        let neo = contains_token(&lower, &["neoforge", "neo_forge", "neo-forge"]);
        let fabric = contains_token(&lower, &["fabric"]);
        let quilt = contains_token(&lower, &["quilt"]);
        let forge = is_forge_token(&lower);

        if neo {
            Some(LoaderKind::NeoForge)
        } else if fabric {
            Some(LoaderKind::Fabric)
        } else if quilt {
            Some(LoaderKind::Quilt)
        } else if forge {
            Some(LoaderKind::Forge)
        } else {
            None
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        Self::from_label(s).unwrap_or(LoaderKind::Vanilla)
    }
}

/// Match loader tokens on alphanumeric boundaries (splits `1.21-fabric-loader` → fabric, loader).
fn contains_token(hay: &str, needles: &[&str]) -> bool {
    let parts: Vec<&str> = hay
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|p| !p.is_empty())
        .collect();
    needles.iter().any(|n| {
        parts.iter().any(|p| {
            *p == *n
                || p.starts_with(n)
                    && (p[n.len()..].is_empty()
                        || p[n.len()..].starts_with("loader")
                        || p[n.len()..].starts_with("mod"))
        })
    })
}

fn is_forge_token(hay: &str) -> bool {
    // Strip known non-Forge lookalikes before matching "forge".
    let cleaned = hay
        .replace("neoforge", " ")
        .replace("neo_forge", " ")
        .replace("neo-forge", " ")
        .replace("forgified", " ")
        .replace("forge-like", " ");
    contains_token(&cleaned, &["minecraftforge"])
        || contains_token(&cleaned, &["forge"])
        || cleaned.contains("lexforgy")
}

/// Strong signals from version profile / folder names (never from random mod jars).
pub fn loader_from_profile(profile_raw: &str, version_folder_names: &[String]) -> Option<LoaderKind> {
    for id in version_folder_names {
        if let Some(l) = LoaderKind::from_label(id) {
            return Some(l);
        }
    }
    if profile_raw.trim().is_empty() {
        return None;
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(profile_raw) {
        if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
            if let Some(l) = LoaderKind::from_label(id) {
                return Some(l);
            }
        }
        if let Some(l) = loader_from_profile_value(&v) {
            return Some(l);
        }
    }
    loader_from_profile_text(profile_raw)
}

fn loader_from_profile_value(v: &serde_json::Value) -> Option<LoaderKind> {
    let main = v
        .get("mainClass")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_lowercase();
    if main.contains("neoforged") {
        return Some(LoaderKind::NeoForge);
    }
    if main.contains("minecraftforge") || main.contains("cpw.mods") {
        return Some(LoaderKind::Forge);
    }
    if main.contains("quiltmc") {
        return Some(LoaderKind::Quilt);
    }
    if main.contains("fabricmc") {
        return Some(LoaderKind::Fabric);
    }

    let blob = v.to_string().to_lowercase();
    if blob.contains("neoforged") || blob.contains("net.neoforge") {
        return Some(LoaderKind::NeoForge);
    }
    if blob.contains("fabric-loader") || blob.contains("net.fabricmc") {
        return Some(LoaderKind::Fabric);
    }
    if blob.contains("quilt-loader") || blob.contains("org.quiltmc") {
        return Some(LoaderKind::Quilt);
    }
    if blob.contains("net.minecraftforge") || blob.contains("minecraftforge") {
        return Some(LoaderKind::Forge);
    }
    None
}

fn loader_from_profile_text(raw: &str) -> Option<LoaderKind> {
    let blob = raw.to_lowercase();
    if blob.contains("neoforged") || blob.contains("net.neoforge") {
        Some(LoaderKind::NeoForge)
    } else if blob.contains("fabric-loader") || blob.contains("net.fabricmc") {
        Some(LoaderKind::Fabric)
    } else if blob.contains("quilt-loader") || blob.contains("org.quiltmc") {
        Some(LoaderKind::Quilt)
    } else if blob.contains("net.minecraftforge") || blob.contains("minecraftforge") {
        Some(LoaderKind::Forge)
    } else {
        None
    }
}

/// Resolve loader: disk profile/folders first, then name, then keep current.
pub fn resolve_loader(
    name: &str,
    game_version: &str,
    current: LoaderKind,
    profile_raw: &str,
    version_folder_names: &[String],
) -> LoaderKind {
    if let Some(l) = loader_from_profile(profile_raw, version_folder_names) {
        return l;
    }
    if let Some(l) = LoaderKind::from_label(&format!("{name} {game_version}")) {
        return l;
    }
    current
}

pub fn infer_loader(name: &str, game_version: &str, current: LoaderKind) -> LoaderKind {
    if let Some(l) = LoaderKind::from_label(&format!("{name} {game_version}")) {
        return l;
    }
    current
}

/// Pull a real Minecraft version id out of messy strings like "1.21.1-Fabric NNEW".
/// Never keeps a bare trailing `-` (that broke Modrinth facets as `1.21.1-`).
pub fn normalize_game_version(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('-').trim();
    let re = regex::Regex::new(
        r"(?i)(\d+\.\d+(?:\.\d+)?(?:-(?:pre|rc|snapshot)\.?\d*)?)",
    )
    .ok();
    if let Some(re) = re {
        if let Some(caps) = re.captures(trimmed) {
            let mut v = caps[1].to_string();
            while v.ends_with('-') {
                v.pop();
            }
            return v;
        }
    }
    let mut base = trimmed
        .split(|c: char| c.is_whitespace())
        .next()
        .unwrap_or("1.21.1")
        .to_string();
    if let Some((head, tail)) = base.split_once('-') {
        let t = tail.to_lowercase();
        if !(t.starts_with("pre") || t.starts_with("rc") || t.starts_with("snapshot")) {
            base = head.to_string();
        }
    }
    base.trim_end_matches('-').to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: LoaderKind,
    pub loader_version: Option<String>,
    pub java_path: Option<String>,
    pub memory_mb: u32,
    #[serde(default)]
    pub jvm_args: String,
    #[serde(default)]
    pub env_vars: String,
    #[serde(default)]
    pub pre_command: String,
    #[serde(default)]
    pub post_command: String,
    pub created_at: String,
    pub last_played: Option<String>,
    /// Real on-disk folder name under instances root. None = instances root.
    #[serde(default, alias = "folder_id")]
    pub folder: Option<String>,
    /// Optional custom icon (data URL or absolute path cached as data URL).
    #[serde(default)]
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceFolder {
    /// Directory name on disk (also used as id).
    pub id: String,
    pub name: String,
    pub created_at: String,
    /// Absolute path to the folder on disk.
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub instances_path: Option<String>,
    pub curseforge_api_key: Option<String>,
    pub java_path: Option<String>,
    #[serde(default)]
    pub last_instance_id: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub download_threads: Option<u32>,
    /// `official` (default) or `bmclapi`
    #[serde(default)]
    pub download_source: Option<String>,
    /// Override path for dedicated Host servers root
    #[serde(default)]
    pub dedicated_path: Option<String>,
    /// Solid page background color (CSS)
    #[serde(default)]
    pub background_color: Option<String>,
    /// Optional local path or URL for body background image
    #[serde(default)]
    pub background_image: Option<String>,
    /// Curated font family key: system | noto | source | plex
    #[serde(default)]
    pub font_family: Option<String>,
    /// UI scale factor: 0.9 / 1 / 1.1 / 1.25
    #[serde(default)]
    pub ui_scale: Option<f64>,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            instances_path: None,
            curseforge_api_key: None,
            java_path: None,
            last_instance_id: None,
            accent: Some("#1370f0".into()),
            locale: Some("en".into()),
            download_threads: Some(16),
            download_source: Some("official".into()),
            dedicated_path: None,
            background_color: None,
            background_image: None,
            font_family: Some("system".into()),
            ui_scale: Some(1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Microsoft,
    Offline,
    LittleSkin,
}

impl Default for AccountKind {
    fn default() -> Self {
        AccountKind::Microsoft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub active: bool,
    #[serde(default)]
    pub kind: AccountKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModEntry {
    pub file_name: String,
    pub enabled: bool,
    pub path: String,
    #[serde(default)]
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqIssue {
    pub severity: IssueSeverity,
    pub mod_id: String,
    pub message: String,
    pub missing_mod_id: Option<String>,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqScanResult {
    pub issues: Vec<ReqIssue>,
    pub mod_count: usize,
    pub scanned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub name: String,
    pub path: String,
    pub kind: String,
    #[serde(default)]
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub text: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashHint {
    pub title: String,
    pub detail: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type_")]
    pub type_: String,
    pub release_time: String,
}
