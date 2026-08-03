use crate::dedicated;
use crate::paths::dedicated_runtime;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

fn ensure_under_runtime(runtime: &Path, dest: &Path) -> Result<(), String> {
    let r = runtime.canonicalize().unwrap_or_else(|_| runtime.to_path_buf());
    let d = if dest.exists() {
        dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf())
    } else {
        dest.to_path_buf()
    };
    if !d.starts_with(&r) && !dest.starts_with(runtime) {
        return Err("Refusing path outside dedicated runtime".into());
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap_or(path);
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).ok();
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(path, &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Upload a world folder or zip into runtime/{level_name or world}/.
pub fn upload_world(id: String, src_path: String) -> Result<String, String> {
    let _ = dedicated::get_dedicated(&id)?;
    let runtime = dedicated_runtime(&id)?;
    let src = PathBuf::from(&src_path);
    if !src.exists() {
        return Err("Source path not found".into());
    }

    let props = dedicated::get_dedicated_properties(&id).unwrap_or_default();
    let level = if props.level_name.trim().is_empty() {
        "world".into()
    } else {
        props.level_name
    };
    let dest = runtime.join(&level);
    ensure_under_runtime(&runtime, &dest)?;

    if src.is_dir() {
        if dest.exists() {
            fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
        }
        copy_dir_recursive(&src, &dest)?;
        return Ok(format!("World uploaded to {level}/"));
    }

    if src
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
    {
        if dest.exists() {
            fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
        }
        fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        let file = fs::File::open(&src).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = f.name().to_string();
            // Skip macOS junk
            if name.contains("__MACOSX") {
                continue;
            }
            let out_path = dest.join(name.trim_start_matches('/'));
            if f.is_dir() {
                fs::create_dir_all(&out_path).ok();
            } else {
                if let Some(p) = out_path.parent() {
                    fs::create_dir_all(p).ok();
                }
                let mut out = fs::File::create(&out_path).map_err(|e| e.to_string())?;
                std::io::copy(&mut f, &mut out).map_err(|e| e.to_string())?;
            }
        }
        return Ok(format!("World zip extracted to {level}/"));
    }

    Err("World upload expects a folder or .zip".into())
}

/// Copy a mod jar or folder of jars into runtime/mods/.
pub fn upload_mods(id: String, src_path: String) -> Result<String, String> {
    let _ = dedicated::get_dedicated(&id)?;
    let runtime = dedicated_runtime(&id)?;
    let mods = runtime.join("mods");
    fs::create_dir_all(&mods).map_err(|e| e.to_string())?;
    let src = PathBuf::from(&src_path);
    if !src.exists() {
        return Err("Source path not found".into());
    }

    let mut count = 0usize;
    if src.is_file() {
        let name = src
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "mod.jar".into());
        fs::copy(&src, mods.join(&name)).map_err(|e| e.to_string())?;
        count = 1;
    } else if src.is_dir() {
        for entry in fs::read_dir(&src).map_err(|e| e.to_string())?.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("jar") {
                let name = p.file_name().unwrap().to_string_lossy().to_string();
                fs::copy(&p, mods.join(name)).map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    } else {
        return Err("Mods upload expects a .jar or folder of jars".into());
    }
    Ok(format!("Uploaded {count} mod file(s)"))
}

fn zip_dir(src: &Path, dest_zip: &Path) -> Result<(), String> {
    let file = fs::File::create(dest_zip).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let src = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
    for entry in WalkDir::new(&src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(&src)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        zip.start_file(&rel, opts).map_err(|e| e.to_string())?;
        let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        zip.write_all(&buf).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn download_world_zip(id: String, dest_path: String) -> Result<String, String> {
    let _ = dedicated::get_dedicated(&id)?;
    let runtime = dedicated_runtime(&id)?;
    let props = dedicated::get_dedicated_properties(&id).unwrap_or_default();
    let level = if props.level_name.trim().is_empty() {
        "world".into()
    } else {
        props.level_name
    };
    let world = runtime.join(&level);
    if !world.is_dir() {
        return Err(format!("World folder '{level}' not found — start the server once to generate it"));
    }
    let dest = PathBuf::from(&dest_path);
    zip_dir(&world, &dest)?;
    Ok(dest_path)
}

pub fn download_mods_zip(id: String, dest_path: String) -> Result<String, String> {
    let _ = dedicated::get_dedicated(&id)?;
    let mods = dedicated_runtime(&id)?.join("mods");
    if !mods.is_dir() {
        fs::create_dir_all(&mods).ok();
    }
    let dest = PathBuf::from(&dest_path);
    zip_dir(&mods, &dest)?;
    Ok(dest_path)
}
