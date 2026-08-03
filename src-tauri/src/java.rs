use crate::instances::get_instance;
use crate::paths::load_settings;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn detect_java_installs() -> Result<Vec<String>, String> {
    let mut found = Vec::new();
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
        if let Ok(entries) = std::fs::read_dir(&root) {
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
