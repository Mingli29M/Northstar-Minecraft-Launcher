use crate::paths::minecraft_dir;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSettings {
    pub world_name: String,
    pub seed: String,
    pub difficulty: i32,
    pub game_type: i32,
    pub hardcore: bool,
    pub allow_commands: bool,
    pub do_daylight_cycle: bool,
    pub keep_inventory: bool,
    pub mob_griefing: bool,
    pub do_mob_spawning: bool,
}

fn level_dat_path(instance_id: &str, world: &str) -> Result<PathBuf, String> {
    let p = minecraft_dir(instance_id)?
        .join("saves")
        .join(world)
        .join("level.dat");
    if !p.exists() {
        return Err(format!("level.dat not found for world '{world}'"));
    }
    Ok(p)
}

fn read_nbt(path: &PathBuf) -> Result<fastnbt::Value, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    fastnbt::from_bytes(&raw).map_err(|e| e.to_string())
}

fn write_nbt(path: &PathBuf, value: &fastnbt::Value) -> Result<(), String> {
    let raw = fastnbt::to_bytes(value).map_err(|e| e.to_string())?;
    let file = fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&raw).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn compound_mut<'a>(v: &'a mut fastnbt::Value) -> Result<&'a mut HashMap<String, fastnbt::Value>, String> {
    match v {
        fastnbt::Value::Compound(m) => Ok(m),
        _ => Err("Expected NBT compound".into()),
    }
}

fn get_i32(map: &HashMap<String, fastnbt::Value>, key: &str, default: i32) -> i32 {
    match map.get(key) {
        Some(fastnbt::Value::Byte(b)) => *b as i32,
        Some(fastnbt::Value::Int(i)) => *i,
        Some(fastnbt::Value::Short(s)) => *s as i32,
        Some(fastnbt::Value::Long(l)) => *l as i32,
        _ => default,
    }
}

fn get_bool(map: &HashMap<String, fastnbt::Value>, key: &str, default: bool) -> bool {
    match map.get(key) {
        Some(fastnbt::Value::Byte(b)) => *b != 0,
        Some(fastnbt::Value::Int(i)) => *i != 0,
        _ => default,
    }
}

fn get_seed(map: &HashMap<String, fastnbt::Value>) -> String {
    if let Some(fastnbt::Value::Long(s)) = map.get("RandomSeed") {
        return s.to_string();
    }
    if let Some(fastnbt::Value::Compound(wd)) = map.get("WorldGenSettings") {
        if let Some(fastnbt::Value::Long(s)) = wd.get("seed") {
            return s.to_string();
        }
    }
    if let Some(fastnbt::Value::Long(s)) = map.get("seed") {
        return s.to_string();
    }
    "0".into()
}

fn gamerule_bool(data: &HashMap<String, fastnbt::Value>, key: &str, default: bool) -> bool {
    let Some(fastnbt::Value::Compound(rules)) = data.get("GameRules") else {
        return default;
    };
    match rules.get(key) {
        Some(fastnbt::Value::String(s)) => s == "true",
        Some(fastnbt::Value::Byte(b)) => *b != 0,
        _ => default,
    }
}

fn set_gamerule(data: &mut HashMap<String, fastnbt::Value>, key: &str, value: bool) {
    let entry = data
        .entry("GameRules".into())
        .or_insert_with(|| fastnbt::Value::Compound(HashMap::new()));
    if let fastnbt::Value::Compound(rules) = entry {
        rules.insert(
            key.into(),
            fastnbt::Value::String(if value { "true" } else { "false" }.into()),
        );
    }
}

pub fn get_world_settings(instance_id: String, world_name: String) -> Result<WorldSettings, String> {
    let path = level_dat_path(&instance_id, &world_name)?;
    let root = read_nbt(&path)?;
    let root_map = match &root {
        fastnbt::Value::Compound(m) => m,
        _ => return Err("Invalid level.dat".into()),
    };
    let data = match root_map.get("Data") {
        Some(fastnbt::Value::Compound(m)) => m,
        _ => root_map,
    };

    Ok(WorldSettings {
        world_name,
        seed: get_seed(data),
        difficulty: get_i32(data, "Difficulty", 2),
        game_type: get_i32(data, "GameType", 0),
        hardcore: get_bool(data, "hardcore", false),
        allow_commands: get_bool(data, "allowCommands", false),
        do_daylight_cycle: gamerule_bool(data, "doDaylightCycle", true),
        keep_inventory: gamerule_bool(data, "keepInventory", false),
        mob_griefing: gamerule_bool(data, "mobGriefing", true),
        do_mob_spawning: gamerule_bool(data, "doMobSpawning", true),
    })
}

pub fn save_world_settings(instance_id: String, settings: WorldSettings) -> Result<WorldSettings, String> {
    let path = level_dat_path(&instance_id, &settings.world_name)?;
    // Backup
    let bak = path.with_extension("dat.euml.bak");
    let _ = fs::copy(&path, &bak);

    let mut root = read_nbt(&path)?;
    let root_map = compound_mut(&mut root)?;
    let data_val = root_map
        .entry("Data".into())
        .or_insert_with(|| fastnbt::Value::Compound(HashMap::new()));
    let data = compound_mut(data_val)?;

    data.insert("Difficulty".into(), fastnbt::Value::Byte(settings.difficulty.clamp(0, 3) as i8));
    data.insert("GameType".into(), fastnbt::Value::Int(settings.game_type));
    data.insert("hardcore".into(), fastnbt::Value::Byte(if settings.hardcore { 1 } else { 0 }));
    data.insert(
        "allowCommands".into(),
        fastnbt::Value::Byte(if settings.allow_commands { 1 } else { 0 }),
    );

    // Seed: update both legacy and modern locations when editable
    if let Ok(seed) = settings.seed.parse::<i64>() {
        data.insert("RandomSeed".into(), fastnbt::Value::Long(seed));
        if let Some(fastnbt::Value::Compound(wd)) = data.get_mut("WorldGenSettings") {
            wd.insert("seed".into(), fastnbt::Value::Long(seed));
        }
    }

    set_gamerule(data, "doDaylightCycle", settings.do_daylight_cycle);
    set_gamerule(data, "keepInventory", settings.keep_inventory);
    set_gamerule(data, "mobGriefing", settings.mob_griefing);
    set_gamerule(data, "doMobSpawning", settings.do_mob_spawning);

    write_nbt(&path, &root)?;
    get_world_settings(instance_id, settings.world_name)
}

/// List config files under instance config/ (+ options.txt at minecraft root).
pub fn list_instance_configs(instance_id: String) -> Result<Vec<String>, String> {
    let mc = minecraft_dir(&instance_id)?;
    let mut out = Vec::new();
    let options = mc.join("options.txt");
    if options.exists() {
        out.push("options.txt".into());
    }
    let config = mc.join("config");
    if config.exists() {
        for entry in walkdir::WalkDir::new(&config).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if matches!(
                ext.as_str(),
                "toml" | "json" | "properties" | "cfg" | "txt" | "yml" | "yaml" | "hjson"
            ) {
                if let Ok(rel) = path.strip_prefix(&mc) {
                    out.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }
    out.sort();
    Ok(out)
}

fn resolve_instance_relative(instance_id: &str, relative: &str) -> Result<PathBuf, String> {
    let mc = minecraft_dir(instance_id)?;
    let rel = relative.replace('\\', "/");
    if rel.contains("..") {
        return Err("Invalid path".into());
    }
    if !(rel == "options.txt" || rel.starts_with("config/")) {
        return Err("Only config/ and options.txt are editable".into());
    }
    let path = mc.join(&rel);
    if path.exists() {
        let canon_mc = fs::canonicalize(&mc).map_err(|e| e.to_string())?;
        let canon = fs::canonicalize(&path).map_err(|e| e.to_string())?;
        if !canon.starts_with(&canon_mc) {
            return Err("Path escapes instance directory".into());
        }
    } else {
        // Ensure parent stays under minecraft dir
        let parent = path.parent().unwrap_or(&mc);
        let canon_mc = fs::canonicalize(&mc).unwrap_or(mc.clone());
        let canon_parent = fs::canonicalize(parent).unwrap_or(parent.to_path_buf());
        if !canon_parent.starts_with(&canon_mc) {
            return Err("Path escapes instance directory".into());
        }
    }
    Ok(path)
}

pub fn read_instance_text_file(instance_id: String, relative: String) -> Result<String, String> {
    let path = resolve_instance_relative(&instance_id, &relative)?;
    if !path.exists() {
        return Err("File not found".into());
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn write_instance_text_file(
    instance_id: String,
    relative: String,
    contents: String,
) -> Result<(), String> {
    let path = resolve_instance_relative(&instance_id, &relative)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, contents).map_err(|e| e.to_string())
}
