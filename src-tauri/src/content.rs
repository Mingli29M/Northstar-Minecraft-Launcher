use crate::models::{ContentItem, LogLine};
use crate::paths::minecraft_dir;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
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

pub fn analyze_crash(instance_id: String) -> Result<Vec<crate::models::CrashHint>, String> {
    let mc = minecraft_dir(&instance_id)?;
    let mut blobs = String::new();
    for candidate in [mc.join("crash-reports"), mc.join("logs")] {
        if !candidate.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&candidate) {
            for entry in entries.flatten().take(8) {
                if let Ok(text) = fs::read_to_string(entry.path()) {
                    blobs.push_str(&text);
                    blobs.push('\n');
                }
            }
        }
    }
    let lower = blobs.to_lowercase();
    let mut hints = Vec::new();
    let rules = [
        ("OutOfMemoryError", "内存不足", "请在版本设置中提高内存，或关闭其它占用内存的程序。", "error"),
        ("mixin", "Mixin 冲突", "常见于模组冲突。尝试禁用最近添加的模组，或查看 ReqGuard。", "error"),
        ("incompatible", "版本不兼容", "模组/加载器与当前 Minecraft 版本可能不匹配。", "warn"),
        ("fabric loader", "Fabric Loader 问题", "检查 Fabric Loader 是否与游戏版本匹配。", "warn"),
        ("modresolutionexception", "模组依赖未满足", "启动前依赖解析失败——这正是 ReqGuard 要提前拦截的问题。", "error"),
        ("classnotfoundexception", "缺少类/依赖", "可能缺少前置模组或加载器组件。", "error"),
        ("opengl", "显卡 / OpenGL", "更新显卡驱动，或关闭光影后再试。", "warn"),
    ];
    for (needle, title, detail, severity) in rules {
        if lower.contains(&needle.to_lowercase()) {
            hints.push(crate::models::CrashHint {
                title: title.into(),
                detail: detail.into(),
                severity: severity.into(),
            });
        }
    }
    Ok(hints)
}
