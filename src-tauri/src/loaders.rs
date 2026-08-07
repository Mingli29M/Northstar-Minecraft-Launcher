use crate::download::{download_file, rewrite_url};
use crate::instances::{get_instance, save_instance};
use crate::models::LoaderKind;
use crate::paths::{instance_dir, meta_dir};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn install_loader(id: String) -> Result<crate::models::Instance, String> {
    let mut inst = get_instance(&id)?;
    match inst.loader {
        LoaderKind::Vanilla => Ok(inst),
        LoaderKind::Fabric => install_fabric(&mut inst),
        LoaderKind::Quilt => install_quilt(&mut inst),
        LoaderKind::NeoForge => install_neoforge(&mut inst),
        LoaderKind::Forge => install_forge(&mut inst),
    }
}

/// True when `patches/version.json` is our old empty Forge/NeoForge stub (or otherwise unusable).
pub fn is_stub_profile(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return true;
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return true;
    };
    if v.get("euml_note").is_some() {
        return true;
    }
    let libs_empty = v
        .get("libraries")
        .and_then(|l| l.as_array())
        .map(|a| a.is_empty())
        .unwrap_or(true);
    let no_args = v.get("arguments").is_none() && v.get("minecraftArguments").is_none();
    libs_empty && no_args
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent("Northstar/1.2.4")
        .build()
        .map_err(|e| e.to_string())
}

fn install_fabric(inst: &mut crate::models::Instance) -> Result<crate::models::Instance, String> {
    let client = http_client()?;
    let loaders: Value = client
        .get(rewrite_url(&format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}",
            inst.game_version
        )))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let loader_ver = loaders
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.pointer("/loader/version"))
        .and_then(|v| v.as_str())
        .ok_or("No Fabric loader for this Minecraft version")?
        .to_string();

    let profile: Value = client
        .get(rewrite_url(&format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
            inst.game_version, loader_ver
        )))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    write_profile(inst, &profile)?;
    inst.loader_version = Some(loader_ver);
    save_instance(inst)?;
    Ok(inst.clone())
}

fn install_quilt(inst: &mut crate::models::Instance) -> Result<crate::models::Instance, String> {
    let client = http_client()?;
    let loaders: Value = client
        .get(rewrite_url(&format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}",
            inst.game_version
        )))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let loader_ver = loaders
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.pointer("/loader/version"))
        .and_then(|v| v.as_str())
        .ok_or("No Quilt loader for this Minecraft version")?
        .to_string();

    let profile: Value = client
        .get(rewrite_url(&format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
            inst.game_version, loader_ver
        )))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    write_profile(inst, &profile)?;
    inst.loader_version = Some(loader_ver);
    save_instance(inst)?;
    Ok(inst.clone())
}

fn install_neoforge(inst: &mut crate::models::Instance) -> Result<crate::models::Instance, String> {
    let client = http_client()?;
    let neo_ver = resolve_neoforge_version(&client, &inst.game_version, inst.loader_version.as_deref())?;
    let installer_name = format!("neoforge-{neo_ver}-installer.jar");
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{neo_ver}/{installer_name}"
    );
    let installer_path = installer_cache_dir()?.join(&installer_name);
    download_file(&installer_url, &installer_path)?;

    let meta = meta_dir()?;
    ensure_launcher_profiles(&meta)?;
    run_forge_family_installer(&inst.game_version, &installer_path, &meta)?;

    let profile_path = meta
        .join("versions")
        .join(format!("neoforge-{neo_ver}"))
        .join(format!("neoforge-{neo_ver}.json"));
    let profile = read_installed_profile(&profile_path).or_else(|_| {
        find_installed_profile(&meta, &format!("neoforge-{neo_ver}"))
            .ok_or_else(|| format!("NeoForge {neo_ver} installed but version profile not found"))
    })?;

    apply_installed_profile(inst, &profile, neo_ver)?;
    Ok(inst.clone())
}

fn install_forge(inst: &mut crate::models::Instance) -> Result<crate::models::Instance, String> {
    let client = http_client()?;
    let forge_ver = resolve_forge_version(&client, &inst.game_version, inst.loader_version.as_deref())?;
    let full = if forge_ver.contains('-') && forge_ver.starts_with(&inst.game_version) {
        forge_ver.clone()
    } else {
        format!("{}-{}", inst.game_version, forge_ver)
    };
    let installer_name = format!("forge-{full}-installer.jar");
    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{full}/{installer_name}"
    );
    let installer_path = installer_cache_dir()?.join(&installer_name);
    download_file(&installer_url, &installer_path)?;

    let meta = meta_dir()?;
    ensure_launcher_profiles(&meta)?;
    run_forge_family_installer(&inst.game_version, &installer_path, &meta)?;

    let candidates = [
        format!("{}-forge-{}", inst.game_version, forge_ver),
        full.clone(),
        format!("forge-{full}"),
    ];
    let mut profile = None;
    for id in &candidates {
        let p = meta.join("versions").join(id).join(format!("{id}.json"));
        if let Ok(v) = read_installed_profile(&p) {
            profile = Some(v);
            break;
        }
    }
    let profile = profile
        .or_else(|| find_installed_profile(&meta, "forge"))
        .ok_or_else(|| format!("Forge {full} installed but version profile not found"))?;

    apply_installed_profile(inst, &profile, forge_ver)?;
    Ok(inst.clone())
}

fn resolve_neoforge_version(
    client: &reqwest::blocking::Client,
    mc: &str,
    requested: Option<&str>,
) -> Result<String, String> {
    if let Some(req) = requested {
        let req = req.trim();
        if !req.is_empty()
            && !req.eq_ignore_ascii_case("recommended")
            && !req.eq_ignore_ascii_case("latest")
        {
            return Ok(req.to_string());
        }
    }

    let parts: Vec<_> = mc.split('.').collect();
    let filter_prefix = if parts.len() >= 2 {
        format!("{}.{}", parts[1], parts.get(2).unwrap_or(&"0"))
    } else {
        mc.to_string()
    };

    let versions: Value = client
        .get(rewrite_url(
            "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge",
        ))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("NeoForge versions API: {e}"))?
        .json()
        .map_err(|e| e.to_string())?;

    let list: Vec<&str> = versions
        .get("versions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .filter(|v| {
            !v.contains("beta")
                && !v.contains("alpha")
                && (v.starts_with(&format!("{filter_prefix}.")) || *v == filter_prefix)
        })
        .collect();

    list.last()
        .map(|s| (*s).to_string())
        .ok_or_else(|| format!("No NeoForge release for Minecraft {mc}"))
}

fn resolve_forge_version(
    client: &reqwest::blocking::Client,
    mc: &str,
    requested: Option<&str>,
) -> Result<String, String> {
    if let Some(req) = requested {
        let req = req.trim();
        if !req.is_empty()
            && !req.eq_ignore_ascii_case("recommended")
            && !req.eq_ignore_ascii_case("latest")
        {
            return Ok(req.to_string());
        }
    }

    let promo: Value = client
        .get(rewrite_url(
            "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
        ))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("Forge promotions: {e}"))?
        .json()
        .map_err(|e| e.to_string())?;

    promo
        .pointer(&format!("/promos/{mc}-recommended"))
        .or_else(|| promo.pointer(&format!("/promos/{mc}-latest")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("No Forge promotion for Minecraft {mc}"))
}

fn installer_cache_dir() -> Result<PathBuf, String> {
    let dir = meta_dir()?.join("installers");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn ensure_launcher_profiles(meta: &Path) -> Result<(), String> {
    let path = meta.join("launcher_profiles.json");
    if !path.exists() {
        fs::write(&path, r#"{"profiles":{}}"#).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn run_forge_family_installer(mc: &str, installer: &Path, meta: &Path) -> Result<(), String> {
    let java = crate::java::resolve_java_path(mc, None)?;
    let mut cmd = Command::new(&java);
    cmd.current_dir(meta)
        .arg("-jar")
        .arg(installer)
        .arg("--installClient")
        .arg(meta);
    crate::win_cmd::hide_console(&mut cmd);
    let status = cmd
        .status()
        .map_err(|e| format!("Failed to run loader installer: {e}"))?;
    if !status.success() {
        return Err(format!(
            "Loader installer failed (exit {}). Check that Java can reach maven.neoforged.net / maven.minecraftforge.net.",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

fn read_installed_profile(path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn find_installed_profile(meta: &Path, must_contain: &str) -> Option<Value> {
    let versions = meta.join("versions");
    let rd = fs::read_dir(versions).ok()?;
    let needle = must_contain.to_ascii_lowercase();
    let mut best: Option<(String, PathBuf)> = None;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.to_ascii_lowercase().contains(&needle) {
            continue;
        }
        let json = entry.path().join(format!("{name}.json"));
        if json.is_file() {
            if best.as_ref().map(|(n, _)| name > *n).unwrap_or(true) {
                best = Some((name, json));
            }
        }
    }
    best.and_then(|(_, p)| read_installed_profile(&p).ok())
}

fn apply_installed_profile(
    inst: &mut crate::models::Instance,
    profile: &Value,
    loader_ver: String,
) -> Result<(), String> {
    if let Some(inherits) = profile
        .get("inheritsFrom")
        .and_then(|v| v.as_str())
        .map(crate::models::normalize_game_version)
        .filter(|s| crate::models::is_plausible_game_version(s))
    {
        inst.game_version = inherits;
    }
    write_profile(inst, profile)?;
    inst.loader_version = Some(loader_ver);
    save_instance(inst)?;
    Ok(())
}

fn write_profile(inst: &crate::models::Instance, profile: &Value) -> Result<(), String> {
    let dir = instance_dir(&inst.id)?.join("patches");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("version.json");
    // Never persist our historical stub marker on real profiles.
    let mut out = profile.clone();
    if let Some(obj) = out.as_object_mut() {
        obj.remove("euml_note");
    }
    fs::write(
        path,
        serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
