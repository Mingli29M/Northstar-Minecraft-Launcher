use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFormat {
    Toml,
    Json,
    Properties,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub key: String,
    /// Human-readable label (Mod Menu style), not the raw key.
    #[serde(default)]
    pub label: String,
    /// Section / category name for grouping.
    #[serde(default)]
    pub section: String,
    pub value: String,
    pub value_type: String, // string | number | bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedConfig {
    pub format: ConfigFormat,
    pub fields: Vec<ConfigField>,
    pub raw: String,
}

pub fn humanize_key(key: &str) -> String {
    let leaf = key.rsplit('.').next().unwrap_or(key);
    let mut out = String::new();
    for (i, ch) in leaf.chars().enumerate() {
        if i > 0 && (ch.is_uppercase() || ch == '_' || ch == '-') {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            if ch == '_' || ch == '-' {
                continue;
            }
        }
        if i == 0 {
            out.extend(ch.to_uppercase());
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() {
        key.to_string()
    } else {
        out
    }
}

pub fn section_of(key: &str) -> String {
    if let Some((section, _)) = key.split_once('.') {
        humanize_key(section)
    } else {
        "General".into()
    }
}

fn make_field(key: String, value: String) -> ConfigField {
    let label = humanize_key(&key);
    let section = section_of(&key);
    let value_type = infer_type(&value);
    ConfigField {
        key,
        label,
        section,
        value,
        value_type,
    }
}

pub fn detect_format(relative: &str, contents: &str) -> ConfigFormat {
    let lower = relative.to_lowercase();
    if lower.ends_with(".toml") {
        return ConfigFormat::Toml;
    }
    if lower.ends_with(".json") || lower.ends_with(".hjson") {
        return ConfigFormat::Json;
    }
    if lower.ends_with(".properties") || lower.ends_with(".cfg") || lower.ends_with("options.txt") {
        return ConfigFormat::Properties;
    }
    let t = contents.trim_start();
    if t.starts_with('{') || t.starts_with('[') {
        return ConfigFormat::Json;
    }
    if t.contains('=') && !t.contains(" = ") && t.lines().all(|l| {
        let l = l.trim();
        l.is_empty() || l.starts_with('#') || l.contains('=')
    }) {
        return ConfigFormat::Properties;
    }
    if t.contains('=') || t.contains('[') {
        return ConfigFormat::Toml;
    }
    ConfigFormat::Text
}

pub fn parse_config(relative: String, contents: String) -> ParsedConfig {
    let format = detect_format(&relative, &contents);
    let fields = match format {
        ConfigFormat::Json => parse_json_flat(&contents),
        ConfigFormat::Toml => parse_toml_flat(&contents),
        ConfigFormat::Properties => parse_properties(&contents),
        ConfigFormat::Text => Vec::new(),
    };
    ParsedConfig {
        format,
        fields,
        raw: contents,
    }
}

fn parse_properties(contents: &str) -> Vec<ConfigField> {
    let mut out = Vec::new();
    let mut section = "General".to_string();
    for line in contents.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('#') || t.starts_with('!') {
            let comment = t.trim_start_matches(['#', '!', ' ', '\t']);
            // Mod Menu–style section headers are often bare comments.
            if !comment.is_empty()
                && !comment.contains('=')
                && comment.len() < 64
                && !comment.to_lowercase().starts_with("http")
            {
                section = humanize_key(comment);
            }
            continue;
        }
        if let Some((k, v)) = t.split_once('=') {
            let key = k.trim().to_string();
            let mut field = make_field(key.clone(), v.trim().to_string());
            if !key.contains('.') {
                field.section = section.clone();
            }
            out.push(field);
        }
    }
    out
}

fn parse_json_flat(contents: &str) -> Vec<ConfigField> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(contents) else {
        return Vec::new();
    };
    let mut map = BTreeMap::new();
    flatten_json("", &v, &mut map);
    map.into_iter()
        .map(|(key, value)| make_field(key, value))
        .collect()
}

fn flatten_json(prefix: &str, v: &serde_json::Value, out: &mut BTreeMap<String, String>) {
    match v {
        serde_json::Value::Object(obj) => {
            for (k, child) in obj {
                let next = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_json(&next, child, out);
            }
        }
        serde_json::Value::Bool(b) => {
            out.insert(prefix.into(), b.to_string());
        }
        serde_json::Value::Number(n) => {
            out.insert(prefix.into(), n.to_string());
        }
        serde_json::Value::String(s) => {
            out.insert(prefix.into(), s.clone());
        }
        serde_json::Value::Null => {
            out.insert(prefix.into(), "null".into());
        }
        serde_json::Value::Array(_) => {
            out.insert(prefix.into(), v.to_string());
        }
    }
}

fn parse_toml_flat(contents: &str) -> Vec<ConfigField> {
    let Ok(table) = contents.parse::<toml::Table>() else {
        return Vec::new();
    };
    let mut map = BTreeMap::new();
    flatten_toml("", &toml::Value::Table(table), &mut map);
    map.into_iter()
        .map(|(key, value)| make_field(key, value))
        .collect()
}

fn flatten_toml(prefix: &str, v: &toml::Value, out: &mut BTreeMap<String, String>) {
    match v {
        toml::Value::Table(t) => {
            for (k, child) in t {
                let next = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_toml(&next, child, out);
            }
        }
        toml::Value::Boolean(b) => {
            out.insert(prefix.into(), b.to_string());
        }
        toml::Value::Integer(i) => {
            out.insert(prefix.into(), i.to_string());
        }
        toml::Value::Float(f) => {
            out.insert(prefix.into(), f.to_string());
        }
        toml::Value::String(s) => {
            out.insert(prefix.into(), s.clone());
        }
        other => {
            out.insert(prefix.into(), other.to_string());
        }
    }
}

fn infer_type(value: &str) -> String {
    if value == "true" || value == "false" {
        "bool".into()
    } else if value.parse::<f64>().is_ok() {
        "number".into()
    } else {
        "string".into()
    }
}

/// Apply flat field edits back into a serialized document (best-effort).
pub fn apply_config_fields(relative: String, original: String, fields: Vec<ConfigField>) -> Result<String, String> {
    let format = detect_format(&relative, &original);
    match format {
        ConfigFormat::Properties | ConfigFormat::Text => {
            let mut map: BTreeMap<String, String> = fields.into_iter().map(|f| (f.key, f.value)).collect();
            let mut out = String::new();
            for line in original.lines() {
                let t = line.trim();
                if t.is_empty() || t.starts_with('#') || t.starts_with('!') || !t.contains('=') {
                    out.push_str(line);
                    out.push('\n');
                    continue;
                }
                let key = t.split_once('=').map(|(k, _)| k.trim()).unwrap_or("");
                if let Some(v) = map.remove(key) {
                    out.push_str(key);
                    out.push('=');
                    out.push_str(&v);
                    out.push('\n');
                } else {
                    out.push_str(line);
                    out.push('\n');
                }
            }
            for (k, v) in map {
                out.push_str(&k);
                out.push('=');
                out.push_str(&v);
                out.push('\n');
            }
            Ok(out)
        }
        ConfigFormat::Json => {
            let mut root: serde_json::Value =
                serde_json::from_str(&original).unwrap_or(serde_json::json!({}));
            for f in fields {
                set_json_path(&mut root, &f.key, &f.value, &f.value_type);
            }
            serde_json::to_string_pretty(&root).map_err(|e| e.to_string())
        }
        ConfigFormat::Toml => {
            // Rewrite as simple key = value for flat keys; nested best-effort via re-parse
            let mut table = original.parse::<toml::Table>().unwrap_or_default();
            for f in fields {
                set_toml_path(&mut table, &f.key, &f.value, &f.value_type);
            }
            Ok(toml::to_string_pretty(&table).unwrap_or(original))
        }
    }
}

fn set_json_path(root: &mut serde_json::Value, path: &str, value: &str, value_type: &str) {
    let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return;
    }
    let mut cur = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(obj) = cur.as_object_mut() {
                obj.insert((*part).to_string(), typed_json(value, value_type));
            }
            return;
        }
        let needs_obj = !cur.get(*part).map(|v| v.is_object()).unwrap_or(false);
        if needs_obj {
            if let Some(obj) = cur.as_object_mut() {
                obj.insert((*part).into(), serde_json::json!({}));
            }
        }
        match cur {
            serde_json::Value::Object(map) => {
                cur = map.get_mut(*part).unwrap();
            }
            _ => return,
        }
    }
}

fn typed_json(value: &str, value_type: &str) -> serde_json::Value {
    match value_type {
        "bool" => serde_json::Value::Bool(value == "true"),
        "number" => value
            .parse::<f64>()
            .map(|n| serde_json::json!(n))
            .unwrap_or(serde_json::Value::String(value.into())),
        _ => serde_json::Value::String(value.into()),
    }
}

fn set_toml_path(table: &mut toml::Table, path: &str, value: &str, value_type: &str) {
    let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();
    if parts.len() == 1 {
        table.insert(parts[0].to_string(), typed_toml(value, value_type));
        return;
    }
    if parts.len() == 2 {
        let key0 = parts[0].to_string();
        let key1 = parts[1].to_string();
        let entry = table
            .entry(key0)
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        if let toml::Value::Table(t) = entry {
            t.insert(key1, typed_toml(value, value_type));
        }
    } else {
        table.insert(path.to_string(), typed_toml(value, value_type));
    }
}

fn typed_toml(value: &str, value_type: &str) -> toml::Value {
    match value_type {
        "bool" => toml::Value::Boolean(value == "true"),
        "number" => {
            if let Ok(i) = value.parse::<i64>() {
                toml::Value::Integer(i)
            } else if let Ok(f) = value.parse::<f64>() {
                toml::Value::Float(f)
            } else {
                toml::Value::String(value.into())
            }
        }
        _ => toml::Value::String(value.into()),
    }
}

/// Guess config files related to a mod jar name.
pub fn configs_for_mod(instance_id: String, mod_file_name: String) -> Result<Vec<String>, String> {
    let stem = PathBuf::from(&mod_file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(mod_file_name)
        .to_lowercase();
    let all = crate::world_settings::list_instance_configs(instance_id)?;
    let tokens: Vec<String> = stem
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() >= 3)
        .map(|s| s.to_string())
        .collect();
    Ok(all
        .into_iter()
        .filter(|p| {
            let pl = p.to_lowercase();
            tokens.iter().any(|t| pl.contains(t))
        })
        .collect())
}
