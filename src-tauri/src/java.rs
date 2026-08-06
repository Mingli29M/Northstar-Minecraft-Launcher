use crate::download;
use crate::instances::get_instance;
use crate::models::{JavaInstall, JavaStatus};
use crate::paths::{app_root, load_settings, save_settings};
use flate2::read::GzDecoder;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;
use walkdir::WalkDir;
use zip::ZipArchive;

pub fn detect_java_installs() -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files\Java",
            r"C:\Program Files\Eclipse Adoptium",
            r"C:\Program Files\Microsoft",
            r"C:\Program Files\Zulu",
            r"C:\Program Files\Amazon Corretto",
            r"C:\Program Files\BellSoft",
        ];
        for root in candidates {
            let root = PathBuf::from(root);
            if !root.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let bin = entry.path().join("bin").join("java.exe");
                    if bin.exists() {
                        found.push(bin.to_string_lossy().to_string());
                    }
                }
            }
        }
        if let Ok(output) = Command::new("where").arg("java").output() {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let line = line.trim();
                    if !line.is_empty() && !found.iter().any(|f| f.eq_ignore_ascii_case(line)) {
                        found.push(line.to_string());
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        for cmd in ["java", "/usr/libexec/java_home"] {
            if let Ok(output) = Command::new(cmd).arg("-version").output() {
                if output.status.success() || !output.stderr.is_empty() {
                    if cmd == "java" {
                        if let Ok(which) = Command::new("which").arg("java").output() {
                            if which.status.success() {
                                let line = String::from_utf8_lossy(&which.stdout).trim().to_string();
                                if !line.is_empty() && !found.iter().any(|f| f.eq_ignore_ascii_case(&line)) {
                                    found.push(line);
                                }
                            }
                        }
                    }
                }
            }
        }
        for root in ["/usr/lib/jvm", "/Library/Java/JavaVirtualMachines"] {
            let root = PathBuf::from(root);
            if !root.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let bin = entry.path().join("bin").join("java");
                    if bin.exists() {
                        found.push(bin.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let managed = app_root()?.join("java");
    if managed.is_dir() {
        for entry in WalkDir::new(&managed).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let name = entry.file_name().to_string_lossy();
                #[cfg(target_os = "windows")]
                let is_java = name == "java.exe";
                #[cfg(not(target_os = "windows"))]
                let is_java = name == "java";
                if is_java {
                    if entry
                        .path()
                        .parent()
                        .and_then(|p| p.file_name())
                        .map(|p| p == "bin")
                        .unwrap_or(false)
                    {
                        let path = entry.path().to_string_lossy().to_string();
                        if !found.iter().any(|f| f.eq_ignore_ascii_case(&path)) {
                            found.push(path);
                        }
                    }
                }
            }
        }
    }
    Ok(found)
}

fn java_major_version(java: &str) -> Option<u32> {
    let output = Command::new(java).arg("-version").output().ok()?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    for line in text.lines() {
        if let Some(idx) = line.find('"') {
            let rest = &line[idx + 1..];
            let ver = rest.split('"').next().unwrap_or("");
            if let Some(major) = ver.split('.').next() {
                if major == "1" {
                    return ver.split('.').nth(1)?.parse().ok();
                }
                return major.parse().ok();
            }
        }
    }
    let lower = java.to_ascii_lowercase();
    for n in (8..=25).rev() {
        if lower.contains(&format!("jdk-{n}"))
            || lower.contains(&format!("jre-{n}"))
            || lower.contains(&format!("jdk{n}"))
            || lower.contains(&format!("temurin-{n}"))
        {
            return Some(n);
        }
    }
    None
}

fn required_java_major(game_version: &str) -> u32 {
    let parts: Vec<u32> = game_version
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    let (maj, min, patch) = (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    );
    if maj > 1 || (maj == 1 && min > 20) || (maj == 1 && min == 20 && patch >= 5) {
        21
    } else if maj == 1 && min >= 18 {
        17
    } else if maj == 1 && min >= 17 {
        16
    } else {
        8
    }
}

pub fn required_java_for_game(game_version: &str) -> u32 {
    required_java_major(game_version)
}

pub fn java_status(game_version: String) -> Result<JavaStatus, String> {
    let required_major = required_java_for_game(&game_version);
    let mut detected: Vec<JavaInstall> = Vec::new();

    let mut add = |path: String| {
        if Path::new(&path).exists() {
            if let Some(major) = java_major_version(&path) {
                if !detected.iter().any(|d| d.path.eq_ignore_ascii_case(&path)) {
                    detected.push(JavaInstall { path, major });
                }
            }
        }
    };

    for path in detect_java_installs()? {
        add(path);
    }
    if let Ok(settings) = load_settings() {
        if let Some(path) = settings.java_path {
            add(path);
        }
    }

    detected.sort_by(|a, b| b.major.cmp(&a.major).then_with(|| a.path.cmp(&b.path)));
    let satisfied = detected.iter().any(|d| d.major >= required_major);
    let recommended_path = detected
        .iter()
        .find(|d| d.major >= required_major)
        .map(|d| d.path.clone());

    Ok(JavaStatus {
        required_major,
        detected,
        satisfied,
        recommended_path,
    })
}

fn adoptium_platform() -> Result<(&'static str, &'static str), String> {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err("Unsupported OS for Temurin download".into());
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        return Err("Unsupported CPU architecture for Temurin download".into());
    };
    Ok((os, arch))
}

pub fn download_temurin(major: u32) -> Result<String, String> {
    let (os, arch) = adoptium_platform()?;
    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{major}/ga/{os}/{arch}/jdk/hotspot/normal/eclipse?project=jdk"
    );
    let dest_dir = app_root()?.join("java").join(format!("temurin-{major}"));
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    {
        let archive = dest_dir.join("temurin.zip");
        download::download_file(&url, &archive)?;
        extract_zip(&archive, &dest_dir)?;
        let _ = fs::remove_file(&archive);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let archive = dest_dir.join("temurin.tar.gz");
        download::download_file(&url, &archive)?;
        extract_tar_gz(&archive, &dest_dir)?;
        let _ = fs::remove_file(&archive);
    }

    let java_path = find_java_binary(&dest_dir)?;
    let mut settings = load_settings()?;
    settings.java_path = Some(java_path.clone());
    save_settings(&settings)?;
    Ok(java_path)
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

fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let file = fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let dec = GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    archive.unpack(dest).map_err(|e| e.to_string())
}

fn find_java_binary(root: &Path) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let name = "java.exe";
    #[cfg(not(target_os = "windows"))]
    let name = "java";

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.file_name().to_string_lossy() == name {
            if entry
                .path()
                .parent()
                .and_then(|p| p.file_name())
                .map(|p| p == "bin")
                .unwrap_or(false)
            {
                return Ok(entry.path().to_string_lossy().to_string());
            }
        }
    }
    Err("Java binary not found after Temurin extraction".into())
}

/// Resolve Java for a game version without requiring a client instance.
pub fn resolve_java_path(game_version: &str, override_path: Option<&str>) -> Result<String, String> {
    let need = required_java_major(game_version);
    if let Some(path) = override_path {
        if Path::new(path).exists() {
            if let Some(v) = java_major_version(path) {
                if v < need {
                    return Err(format!(
                        "Java is {v}, but Minecraft {game_version} needs Java {need}+"
                    ));
                }
            }
            return Ok(path.to_string());
        }
    }
    let settings = load_settings()?;
    if let Some(path) = &settings.java_path {
        if Path::new(path).exists() {
            if let Some(v) = java_major_version(path) {
                if v >= need {
                    return Ok(path.clone());
                }
            } else {
                return Ok(path.clone());
            }
        }
    }
    let mut ranked: Vec<(u32, String)> = detect_java_installs()?
        .into_iter()
        .filter_map(|p| java_major_version(&p).map(|v| (v, p)))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    if let Some((_, path)) = ranked.iter().find(|(v, _)| *v >= need) {
        return Ok(path.clone());
    }
    Err(format!(
        "No Java {need}+ found for Minecraft {game_version}. Install Temurin {need}+ in Settings."
    ))
}

pub fn resolve_java(instance_id: String) -> Result<String, String> {
    let inst = get_instance(&instance_id)?;
    let override_path = inst.java_path.as_deref();
    resolve_java_path(&inst.game_version, override_path)
}
