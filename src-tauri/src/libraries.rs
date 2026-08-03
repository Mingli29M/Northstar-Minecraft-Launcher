use crate::download::download_many;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use zip::ZipArchive;

/// Collect download jobs from a version/profile JSON libraries array.
pub fn collect_library_jobs(version: &Value, libraries_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut jobs = Vec::new();
    let Some(libs) = version.get("libraries").and_then(|l| l.as_array()) else {
        return jobs;
    };
    for lib in libs {
        if !library_allowed(lib) {
            continue;
        }
        // Modern Mojang artifact
        if let Some(artifact) = lib.pointer("/downloads/artifact") {
            let path = artifact.get("path").and_then(|p| p.as_str()).unwrap_or("");
            let url = artifact.get("url").and_then(|u| u.as_str()).unwrap_or("");
            if !path.is_empty() && !url.is_empty() {
                jobs.push((url.to_string(), libraries_dir.join(path)));
            }
        } else if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
            // Fabric/Quilt style: name + optional url (maven repo root)
            if let Some((rel, url)) = maven_artifact(name, lib.get("url").and_then(|u| u.as_str())) {
                jobs.push((url, libraries_dir.join(rel)));
            }
        }

        // Native classifiers (LWJGL etc.)
        if let Some(natives) = lib.get("natives").and_then(|n| n.as_object()) {
            let key = if cfg!(windows) {
                "windows"
            } else if cfg!(target_os = "macos") {
                "osx"
            } else {
                "linux"
            };
            if let Some(classifier) = natives.get(key).and_then(|c| c.as_str()) {
                let classifier = classifier.replace("${arch}", "64");
                if let Some(dl) = lib
                    .pointer(&format!("/downloads/classifiers/{classifier}"))
                    .cloned()
                {
                    let path = dl.get("path").and_then(|p| p.as_str()).unwrap_or("");
                    let url = dl.get("url").and_then(|u| u.as_str()).unwrap_or("");
                    if !path.is_empty() && !url.is_empty() {
                        jobs.push((url.to_string(), libraries_dir.join(path)));
                    }
                } else if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
                    // group:artifact:version → group:artifact:version:classifier
                    let native_name = format!("{name}:{classifier}");
                    if let Some((rel, url)) =
                        maven_artifact(&native_name, lib.get("url").and_then(|u| u.as_str()))
                    {
                        jobs.push((url, libraries_dir.join(rel)));
                    }
                }
            }
        }
    }
    jobs
}

fn library_allowed(lib: &Value) -> bool {
    let Some(rules) = lib.get("rules").and_then(|r| r.as_array()) else {
        return true;
    };
    let mut allowed = false;
    for rule in rules {
        let action = rule.get("action").and_then(|a| a.as_str()).unwrap_or("allow");
        let os_name = rule.pointer("/os/name").and_then(|n| n.as_str());
        let matches_os = match os_name {
            None => true,
            Some("windows") => cfg!(windows),
            Some("osx") => cfg!(target_os = "macos"),
            Some("linux") => cfg!(target_os = "linux"),
            _ => false,
        };
        if matches_os {
            allowed = action == "allow";
        }
    }
    allowed
}

/// `group:artifact:version` or `group:artifact:version:classifier` → (relative path, url)
pub fn maven_artifact(name: &str, repo: Option<&str>) -> Option<(String, String)> {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = parts.get(3).copied();
    let file = match classifier {
        Some(c) if !c.is_empty() => format!("{artifact}-{version}-{c}.jar"),
        _ => format!("{artifact}-{version}.jar"),
    };
    let rel = format!("{group}/{artifact}/{version}/{file}");
    let base = repo
        .unwrap_or("https://libraries.minecraft.net/")
        .trim_end_matches('/');
    // Prefer fabric maven when repo omitted but name is fabricmc — callers usually pass url.
    let url = format!("{base}/{rel}");
    Some((rel, url))
}

pub fn classpath_entries(version: &Value, libraries_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Some(libs) = version.get("libraries").and_then(|l| l.as_array()) else {
        return out;
    };
    for lib in libs {
        if !library_allowed(lib) {
            continue;
        }
        // Skip natives-only jars on classpath
        let has_artifact = lib.pointer("/downloads/artifact").is_some()
            || (lib.get("name").is_some() && lib.get("natives").is_none());
        let natives_only = lib.get("natives").is_some()
            && lib.pointer("/downloads/artifact").is_none()
            && lib.get("name").is_some();
        // Include main jar if present; natives jars with artifact still get artifact on CP
        if natives_only {
            continue;
        }
        if let Some(path) = lib.pointer("/downloads/artifact/path").and_then(|p| p.as_str()) {
            let p = libraries_dir.join(path);
            if p.exists() {
                out.push(p);
            }
            continue;
        }
        if has_artifact {
            if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
                if let Some((rel, _)) = maven_artifact(name, None) {
                    let p = libraries_dir.join(rel);
                    if p.exists() {
                        out.push(p);
                    }
                }
            }
        }
    }
    out
}

/// Download libraries for vanilla + optional loader profile; extract natives.
pub fn ensure_libraries(
    app: Option<&AppHandle>,
    vanilla: &Value,
    profile: Option<&Value>,
    libraries_dir: &Path,
    natives_dir: &Path,
) -> Result<(usize, usize), String> {
    fs::create_dir_all(libraries_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(natives_dir).map_err(|e| e.to_string())?;

    let mut jobs = collect_library_jobs(vanilla, libraries_dir);
    if let Some(p) = profile {
        for j in collect_library_jobs(p, libraries_dir) {
            if !jobs.iter().any(|(_, dest)| dest == &j.1) {
                jobs.push(j);
            }
        }
    }
    let result = download_many(jobs, app, "libraries")?;

    // Extract natives from downloaded classifier jars referenced by vanilla (+ profile)
    extract_natives(vanilla, libraries_dir, natives_dir);
    if let Some(p) = profile {
        extract_natives(p, libraries_dir, natives_dir);
    }

    Ok(result)
}

fn extract_natives(version: &Value, libraries_dir: &Path, natives_dir: &Path) {
    let Some(libs) = version.get("libraries").and_then(|l| l.as_array()) else {
        return;
    };
    let key = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    };
    for lib in libs {
        if !library_allowed(lib) {
            continue;
        }
        let Some(natives) = lib.get("natives").and_then(|n| n.as_object()) else {
            continue;
        };
        let Some(classifier) = natives.get(key).and_then(|c| c.as_str()) else {
            continue;
        };
        let classifier = classifier.replace("${arch}", "64");
        let jar_path = if let Some(path) = lib
            .pointer(&format!("/downloads/classifiers/{classifier}/path"))
            .and_then(|p| p.as_str())
        {
            libraries_dir.join(path)
        } else if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
            let native_name = format!("{name}:{classifier}");
            match maven_artifact(&native_name, None) {
                Some((rel, _)) => libraries_dir.join(rel),
                None => continue,
            }
        } else {
            continue;
        };
        if jar_path.exists() {
            let _ = unzip_natives(&jar_path, natives_dir);
        }
    }
}

fn unzip_natives(jar: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(jar).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if name.ends_with('/') || name.starts_with("META-INF") {
            continue;
        }
        let lower = name.to_lowercase();
        if !(lower.ends_with(".dll")
            || lower.ends_with(".so")
            || lower.ends_with(".dylib")
            || lower.ends_with(".jnilib"))
        {
            continue;
        }
        let filename = Path::new(&name)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or(name);
        let out = dest.join(filename);
        if out.exists() {
            continue;
        }
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        let mut f = fs::File::create(&out).map_err(|e| e.to_string())?;
        f.write_all(&buf).map_err(|e| e.to_string())?;
    }
    Ok(())
}
