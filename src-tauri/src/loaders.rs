use crate::download::rewrite_url;
use crate::instances::{get_instance, save_instance};
use crate::models::LoaderKind;
use crate::paths::instance_dir;
use serde_json::{json, Value};
use std::fs;

pub fn install_loader(id: String) -> Result<crate::models::Instance, String> {
    let mut inst = get_instance(&id)?;
    match inst.loader {
        LoaderKind::Vanilla => Ok(inst),
        LoaderKind::Fabric => install_fabric(&mut inst),
        LoaderKind::Quilt => install_quilt(&mut inst),
        LoaderKind::Forge | LoaderKind::NeoForge => install_forge_family(&mut inst),
    }
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .user_agent("Northstar/1.2.3")
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

fn install_forge_family(inst: &mut crate::models::Instance) -> Result<crate::models::Instance, String> {
    let client = http_client()?;
    let is_neo = matches!(inst.loader, LoaderKind::NeoForge);
    let loader_ver = if is_neo {
        inst.loader_version
            .clone()
            .unwrap_or_else(|| "recommended".into())
    } else {
        let promo: Value = client
            .get(rewrite_url(
                "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
            ))
            .send()
            .map_err(|e| e.to_string())?
            .json()
            .unwrap_or(Value::Null);
        promo
            .pointer(&format!("/promos/{}-recommended", inst.game_version))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| inst.loader_version.clone())
            .unwrap_or_else(|| "recommended".into())
    };

    let profile = json!({
        "id": format!("{}-{}-{}", inst.game_version, inst.loader.as_str(), loader_ver),
        "inheritsFrom": inst.game_version,
        "mainClass": if is_neo {
            "cpw.mods.bootstraplauncher.BootstrapLauncher"
        } else {
            "cpw.mods.modlauncher.Launcher"
        },
        "libraries": [],
        "euml_note": "Forge/NeoForge full installer bootstrap is staged; vanilla libs still download via prepare."
    });

    write_profile(inst, &profile)?;
    inst.loader_version = Some(loader_ver);
    save_instance(inst)?;
    Ok(inst.clone())
}

fn write_profile(inst: &crate::models::Instance, profile: &Value) -> Result<(), String> {
    let dir = instance_dir(&inst.id)?.join("patches");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("version.json");
    fs::write(path, serde_json::to_string_pretty(profile).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(())
}
