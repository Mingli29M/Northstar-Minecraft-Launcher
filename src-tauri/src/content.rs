use crate::models::{ContentItem, CrashHint, LogLine, LitematicaInfo, WorldBackup, WorldInfo};
use crate::paths::minecraft_dir;
use chrono::{NaiveDateTime, Utc};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;
use zip::ZipArchive;

fn content_icon(kind: &str, path: &Path) -> Option<String> {
    let k = kind.to_lowercase();
    if k.contains("save") || k.contains("world") {
        crate::icons::icon_for_world(path)
    } else if k.contains("screenshot") {
        None
    } else {
        crate::icons::icon_for_pack(path)
    }
}

pub fn list_content(instance_id: String, kind: String) -> Result<Vec<ContentItem>, String> {
    let dir = content_dir(&instance_id, &kind)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        out.push(ContentItem {
            name,
            path: entry.path().to_string_lossy().to_string(),
            kind: kind.clone(),
            icon_path: content_icon(&kind, &entry.path()),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn content_dir(instance_id: &str, kind: &str) -> Result<PathBuf, String> {
    let kind = match kind {
        "worlds" | "saves" => "saves",
        "resourcepacks" | "resourcepack" => "resourcepacks",
        "shaderpacks" | "shader" | "shaders" => "shaderpacks",
        "datapacks" | "datapack" => "datapacks",
        "screenshots" => "screenshots",
        other => other,
    };
    Ok(minecraft_dir(instance_id)?.join(kind))
}

pub fn install_content_zip(
    instance_id: String,
    kind: String,
    zip_path: String,
) -> Result<Vec<ContentItem>, String> {
    let dest_dir = content_dir(&instance_id, &kind)?;
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let src = PathBuf::from(&zip_path);
    if !src.exists() {
        return Err("Source file not found".into());
    }
    let file_name = src
        .file_name()
        .ok_or("Invalid path")?
        .to_string_lossy()
        .to_string();

    // Folders (e.g. resource pack folder) — copy tree
    if src.is_dir() {
        let dest = unique_path(dest_dir.join(&file_name));
        copy_dir(&src, &dest)?;
        return list_content(instance_id, normalize_kind(&kind));
    }

    let lower = file_name.to_lowercase();
    // World / pack zips: extract into a folder named after the archive
    if lower.ends_with(".zip") || lower.ends_with(".mrpack") {
        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("pack");
        let dest = unique_path(dest_dir.join(stem));
        extract_zip(&src, &dest)?;
        return list_content(instance_id, normalize_kind(&kind));
    }

    // Loose files (zip resource packs kept as .zip are valid for Minecraft)
    let dest = unique_path(dest_dir.join(&file_name));
    fs::copy(&src, &dest).map_err(|e| e.to_string())?;
    list_content(instance_id, normalize_kind(&kind))
}

fn normalize_kind(kind: &str) -> String {
    match kind {
        "worlds" | "saves" => "saves".into(),
        "resourcepack" => "resourcepacks".into(),
        "shader" | "shaders" => "shaderpacks".into(),
        "datapack" => "datapacks".into(),
        other => other.to_string(),
    }
}

pub fn delete_content(instance_id: String, kind: String, name: String) -> Result<Vec<ContentItem>, String> {
    let path = content_dir(&instance_id, &kind)?.join(&name);
    if path.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
    } else if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    } else {
        return Err(format!("Not found: {name}"));
    }
    list_content(instance_id, normalize_kind(&kind))
}

pub fn open_content_item(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    let target = if p.is_file() {
        p.parent().map(|x| x.to_path_buf()).unwrap_or(p)
    } else {
        p
    };
    open::that(target).map_err(|e| e.to_string())
}

/// Import a Minecraft saves folder (or a single world folder) into the instance.
pub fn import_save(instance_id: String, src_path: String) -> Result<Vec<ContentItem>, String> {
    let src = PathBuf::from(&src_path);
    if !src.exists() {
        return Err("Save path not found".into());
    }
    let dest_root = content_dir(&instance_id, "saves")?;
    fs::create_dir_all(&dest_root).map_err(|e| e.to_string())?;

    if src.is_file() && src.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("zip")).unwrap_or(false)
    {
        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("world");
        let dest = unique_path(dest_root.join(stem));
        extract_zip(&src, &dest)?;
        return list_content(instance_id, "saves".into());
    }

    if !src.is_dir() {
        return Err("Select a world folder, a saves folder, or a .zip".into());
    }

    // If this looks like a saves root (multiple worlds), import each child world.
    if looks_like_saves_root(&src) {
        for entry in fs::read_dir(&src).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let dest = unique_path(dest_root.join(&name));
                copy_dir(&entry.path(), &dest)?;
            }
        }
    } else {
        // Single world folder
        let name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "World".into());
        let dest = unique_path(dest_root.join(name));
        copy_dir(&src, &dest)?;
    }
    list_content(instance_id, "saves".into())
}

fn looks_like_saves_root(dir: &Path) -> bool {
    // Heuristic: contains level.dat directly → single world; otherwise treat as saves root if any child has level.dat
    if dir.join("level.dat").exists() {
        return false;
    }
    fs::read_dir(dir)
        .ok()
        .map(|rd| {
            rd.flatten().any(|e| e.path().is_dir() && e.path().join("level.dat").exists())
        })
        .unwrap_or(false)
}

pub fn list_worlds(instance_id: String) -> Result<Vec<ContentItem>, String> {
    list_content(instance_id, "saves".into())
}

fn world_path(instance_id: &str, world_name: &str) -> Result<PathBuf, String> {
    let path = minecraft_dir(instance_id)?
        .join("saves")
        .join(world_name);
    if !path.is_dir() {
        return Err(format!("World not found: {world_name}"));
    }
    Ok(path)
}

fn backups_root(instance_id: &str, world_name: &str) -> Result<PathBuf, String> {
    Ok(world_path(instance_id, world_name)?.join("backups"))
}

fn backup_timestamp_name() -> String {
    Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string()
}

fn backup_created_at(name: &str, path: &Path) -> String {
    if let Ok(dt) = NaiveDateTime::parse_from_str(name, "%Y-%m-%d_%H-%M-%S") {
        return dt.and_utc().to_rfc3339();
    }
    if let Ok(m) = fs::metadata(path) {
        if let Ok(t) = m.modified() {
            if let Ok(dt) = t.duration_since(std::time::UNIX_EPOCH) {
                if let Some(utc) = chrono::DateTime::<Utc>::from_timestamp(dt.as_secs() as i64, 0) {
                    return utc.to_rfc3339();
                }
            }
        }
    }
    Utc::now().to_rfc3339()
}

fn copy_world_for_backup(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "backups" {
            continue;
        }
        let target = dest.join(&name);
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn clear_world_except_backups(world_dir: &Path) -> Result<(), String> {
    for entry in fs::read_dir(world_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "backups" {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        } else if path.is_file() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn merge_backup_into_world(backup_dir: &Path, world_dir: &Path) -> Result<(), String> {
    for entry in fs::read_dir(backup_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "backups" {
            continue;
        }
        let target = world_dir.join(&name);
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn list_world_backups(instance_id: String, world_name: String) -> Result<Vec<WorldBackup>, String> {
    let root = backups_root(&instance_id, &world_name)?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        out.push(WorldBackup {
            created_at: backup_created_at(&name, &entry.path()),
            path: entry.path().to_string_lossy().to_string(),
            name,
        });
    }
    out.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(out)
}

pub fn create_world_backup(instance_id: String, world_name: String) -> Result<WorldBackup, String> {
    let world_dir = world_path(&instance_id, &world_name)?;
    let root = backups_root(&instance_id, &world_name)?;
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let name = backup_timestamp_name();
    let dest = root.join(&name);
    copy_world_for_backup(&world_dir, &dest)?;
    Ok(WorldBackup {
        name: name.clone(),
        path: dest.to_string_lossy().to_string(),
        created_at: backup_created_at(&name, &dest),
    })
}

pub fn restore_world_backup(
    instance_id: String,
    world_name: String,
    backup_name: String,
) -> Result<(), String> {
    let world_dir = world_path(&instance_id, &world_name)?;
    let backup_dir = backups_root(&instance_id, &world_name)?.join(&backup_name);
    if !backup_dir.is_dir() {
        return Err(format!("Backup not found: {backup_name}"));
    }
    let _ = create_world_backup(instance_id.clone(), world_name.clone())?;
    clear_world_except_backups(&world_dir)?;
    merge_backup_into_world(&backup_dir, &world_dir)
}

pub fn delete_world_backup(
    instance_id: String,
    world_name: String,
    backup_name: String,
) -> Result<(), String> {
    let path = backups_root(&instance_id, &world_name)?.join(&backup_name);
    if !path.is_dir() {
        return Err(format!("Backup not found: {backup_name}"));
    }
    fs::remove_dir_all(&path).map_err(|e| e.to_string())
}

pub fn prune_world_backups(instance_id: &str, world_name: &str, keep: u32) -> Result<(), String> {
    let mut backups = list_world_backups(instance_id.to_string(), world_name.to_string())?;
    if backups.len() <= keep as usize {
        return Ok(());
    }
    backups.sort_by(|a, b| b.name.cmp(&a.name));
    for backup in backups.into_iter().skip(keep as usize) {
        delete_world_backup(
            instance_id.to_string(),
            world_name.to_string(),
            backup.name,
        )?;
    }
    Ok(())
}

pub fn auto_backup_all_worlds(instance_id: &str, keep: u32) -> Result<(), String> {
    let saves = minecraft_dir(instance_id)?.join("saves");
    if !saves.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&saves).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        create_world_backup(instance_id.to_string(), name.clone())?;
        prune_world_backups(instance_id, &name, keep)?;
    }
    Ok(())
}

pub fn list_worlds_detailed(instance_id: String) -> Result<Vec<WorldInfo>, String> {
    let saves = minecraft_dir(&instance_id)?.join("saves");
    fs::create_dir_all(&saves).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&saves).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let backup_count = list_world_backups(instance_id.clone(), name.clone())?
            .len() as u32;
        out.push(WorldInfo {
            name: name.clone(),
            path: entry.path().to_string_lossy().to_string(),
            backup_count,
            has_backups: backup_count > 0,
            icon_path: crate::icons::icon_for_world(&entry.path()),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn jar_is_litematica(path: &Path, file_name: &str) -> bool {
    if file_name.to_lowercase().contains("litematica") {
        return true;
    }
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    let Ok(mut archive) = ZipArchive::new(std::io::Cursor::new(bytes)) else {
        return false;
    };
    for meta_name in ["fabric.mod.json", "quilt.mod.json"] {
        let Ok(mut f) = archive.by_name(meta_name) else {
            continue;
        };
        let mut raw = String::new();
        if f.read_to_string(&mut raw).is_err() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                if id.to_lowercase().contains("litematica") {
                    return true;
                }
            }
        }
    }
    false
}

pub fn detect_litematica(instance_id: String) -> Result<LitematicaInfo, String> {
    let mc = minecraft_dir(&instance_id)?;
    let schematics_path = mc.join("schematics");
    let mods = mc.join("mods");
    let mut present = false;
    if mods.is_dir() {
        for entry in fs::read_dir(&mods).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".jar") {
                continue;
            }
            if jar_is_litematica(&entry.path(), &name) {
                present = true;
                break;
            }
        }
    }
    Ok(LitematicaInfo {
        present,
        schematics_path: schematics_path.to_string_lossy().to_string(),
    })
}

pub fn list_screenshots(instance_id: String) -> Result<Vec<ContentItem>, String> {
    list_content(instance_id, "screenshots".into())
}

pub fn list_datapacks(instance_id: String, world_name: String) -> Result<Vec<ContentItem>, String> {
    let dir = minecraft_dir(&instance_id)?
        .join("saves")
        .join(&world_name)
        .join("datapacks");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        out.push(ContentItem {
            name,
            path: entry.path().to_string_lossy().to_string(),
            kind: "datapacks".into(),
            icon_path: crate::icons::icon_for_pack(&entry.path()),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn install_datapack(
    instance_id: String,
    world_name: String,
    src_path: String,
) -> Result<Vec<ContentItem>, String> {
    let dest_dir = minecraft_dir(&instance_id)?
        .join("saves")
        .join(&world_name)
        .join("datapacks");
    if !minecraft_dir(&instance_id)?.join("saves").join(&world_name).exists() {
        return Err("World not found".into());
    }
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let src = PathBuf::from(&src_path);
    let file_name = src
        .file_name()
        .ok_or("Invalid path")?
        .to_string_lossy()
        .to_string();
    if src.is_dir() {
        copy_dir(&src, &unique_path(dest_dir.join(&file_name)))?;
    } else if file_name.to_lowercase().ends_with(".zip") {
        // Datapacks may stay as zip
        fs::copy(&src, unique_path(dest_dir.join(&file_name))).map_err(|e| e.to_string())?;
    } else {
        fs::copy(&src, unique_path(dest_dir.join(&file_name))).map_err(|e| e.to_string())?;
    }
    list_datapacks(instance_id, world_name)
}

pub fn delete_datapack(
    instance_id: String,
    world_name: String,
    name: String,
) -> Result<Vec<ContentItem>, String> {
    let path = minecraft_dir(&instance_id)?
        .join("saves")
        .join(&world_name)
        .join("datapacks")
        .join(&name);
    if path.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
    } else if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    list_datapacks(instance_id, world_name)
}

fn unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "item".into());
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let parent = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    for i in 2..1000 {
        let candidate = parent.join(format!("{stem}-{i}{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    path
}

fn copy_dir(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(src).map_err(|e| e.to_string())?;
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn extract_zip(zip_path: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(p) => dest.join(p),
            None => continue,
        };
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            outfile.write_all(&buf).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn read_logs(instance_id: String) -> Result<Vec<LogLine>, String> {
    let logs = minecraft_dir(&instance_id)?.join("logs");
    fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
    let latest = logs.join("latest.log");
    let euml = logs.join("euml-last-launch.txt");
    let path = if latest.exists() {
        latest
    } else if euml.exists() {
        euml
    } else {
        return Ok(Vec::new());
    };
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let level = if line.contains("/ERROR") || line.contains(" ERROR ") {
            "ERROR"
        } else if line.contains("/WARN") || line.contains(" WARN ") {
            "WARN"
        } else if line.contains("/INFO") || line.contains(" INFO ") {
            "INFO"
        } else {
            "INFO"
        };
        out.push(LogLine {
            text: line.to_string(),
            level: level.into(),
        });
    }
    Ok(out)
}

pub fn analyze_crash(instance_id: String) -> Result<Vec<CrashHint>, String> {
    analyze_crash_since(instance_id, None)
}

pub fn analyze_crash_since(
    instance_id: String,
    since: Option<SystemTime>,
) -> Result<Vec<CrashHint>, String> {
    let mc = minecraft_dir(&instance_id)?;
    let Some((_path, text)) = find_crash_source(&mc, since) else {
        return Ok(Vec::new());
    };

    let exception = extract_exception_type(&text);
    let frames = extract_stack_frames(&text, 5);
    let mut params = Vec::new();
    if let Some(ref ex) = exception {
        params.push(ex.clone());
    }
    params.extend(frames.clone());

    let lower = text.to_lowercase();
    let mut hints = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let rules: &[(&str, &str, &str, &str, fn(&str) -> bool)] = &[
        (
            "oom",
            "Out of memory",
            "Increase allocated memory in instance settings or close other applications.",
            "error",
            |s| {
                s.contains("outofmemoryerror")
                    || s.contains("java heap space")
                    || s.contains("gc overhead limit exceeded")
            },
        ),
        (
            "mixin_conflict",
            "Mixin conflict",
            "Often caused by mod conflicts. Try disabling recently added mods or check ReqGuard.",
            "error",
            |s| {
                s.contains("mixin")
                    || s.contains("mixinapplyerror")
                    || s.contains("mixin transformation")
            },
        ),
        (
            "mod_resolution",
            "Mod dependency unresolved",
            "Dependency resolution failed at launch — ReqGuard can catch this before starting.",
            "error",
            |s| s.contains("modresolutionexception") || s.contains("mod resolution"),
        ),
        (
            "missing_class",
            "Missing class or dependency",
            "A required mod or library may be missing. Check dependencies and loader components.",
            "error",
            |s| {
                s.contains("classnotfoundexception")
                    || s.contains("noclassdeffounderror")
            },
        ),
        (
            "incompatible",
            "Version incompatible",
            "A mod or loader may not match the current Minecraft version.",
            "warn",
            |s| {
                s.contains("incompatible")
                    || s.contains("incompatibleclasschangeerror")
            },
        ),
        (
            "fabric_loader",
            "Fabric Loader issue",
            "Verify Fabric Loader matches your game version.",
            "warn",
            |s| s.contains("fabric loader") || s.contains("fabric-loader") || s.contains("net.fabricmc.loader"),
        ),
        (
            "opengl",
            "Graphics / OpenGL",
            "Update graphics drivers or disable shader packs and retry.",
            "warn",
            |s| {
                s.contains("opengl")
                    || s.contains("glfw")
                    || s.contains("lwjgl")
                    || s.contains("graphics")
            },
        ),
    ];

    for (code, title, detail, severity, test) in rules {
        if test(&lower) && seen.insert(*code) {
            hints.push(CrashHint {
                code: (*code).into(),
                title: (*title).into(),
                detail: (*detail).into(),
                severity: (*severity).into(),
                params: params.clone(),
            });
        }
    }

    if hints.is_empty() {
        hints.push(CrashHint {
            code: "unknown".into(),
            title: "Unknown crash".into(),
            detail: "No known pattern matched. Review the crash report or latest.log for details."
                .into(),
            severity: "warn".into(),
            params,
        });
    }

    Ok(hints)
}

fn find_crash_source(mc: &Path, since: Option<SystemTime>) -> Option<(PathBuf, String)> {
    let crash_dir = mc.join("crash-reports");
    if crash_dir.is_dir() {
        let mut files: Vec<_> = fs::read_dir(&crash_dir)
            .ok()?
            .flatten()
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x.eq_ignore_ascii_case("txt"))
                    .unwrap_or(false)
            })
            .collect();
        files.sort_by_key(|e| {
            e.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });
        files.reverse();
        for entry in files {
            if !file_is_new_enough(&entry.path(), since) {
                continue;
            }
            if let Ok(text) = fs::read_to_string(entry.path()) {
                if !text.trim().is_empty() {
                    return Some((entry.path(), text));
                }
            }
        }
    }

    let latest = mc.join("logs").join("latest.log");
    if latest.is_file() && file_is_new_enough(&latest, since) {
        if let Ok(text) = fs::read_to_string(&latest) {
            if !text.trim().is_empty() {
                return Some((latest, text));
            }
        }
    }
    None
}

fn file_is_new_enough(path: &Path, since: Option<SystemTime>) -> bool {
    let Some(since) = since else {
        return true;
    };
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|modified| modified >= since)
        .unwrap_or(false)
}

fn extract_exception_type(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains("Exception")
            || trimmed.contains("Error:")
            || trimmed.ends_with("Error")
            || trimmed.starts_with("Caused by:")
        {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn extract_stack_frames(text: &str, limit: usize) -> Vec<String> {
    let mut frames = Vec::new();
    for line in text.lines() {
        if frames.len() >= limit {
            break;
        }
        let trimmed = line.trim();
        let frame = if let Some(rest) = trimmed.strip_prefix("at ") {
            rest
        } else if let Some(rest) = line.split("\tat ").nth(1) {
            rest.trim()
        } else {
            continue;
        };
        if !frame.is_empty() && !frames.iter().any(|x| x == frame) {
            frames.push(frame.to_string());
        }
    }
    frames
}
