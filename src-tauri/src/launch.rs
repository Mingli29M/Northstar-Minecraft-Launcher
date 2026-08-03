use crate::auth::active_account;
use crate::download::{download_file, download_many_progress, emit_progress, DownloadProgress};
use crate::instances::{get_instance, save_instance};
use crate::java::resolve_java;
use crate::models::LoaderKind;
use crate::paths::{meta_dir, minecraft_dir};
use crate::reqguard;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::AppHandle;

pub fn list_game_versions() -> Result<Vec<String>, String> {
    Ok(list_game_versions_detailed()?
        .into_iter()
        .filter(|v| v.type_ == "release")
        .map(|v| v.id)
        .collect())
}

pub fn list_game_versions_detailed() -> Result<Vec<crate::models::VersionInfo>, String> {
    let cache_path = meta_dir()?.join("version_manifest_cache.json");
    const TTL_SECS: u64 = 60 * 60;

    if let Ok(meta) = fs::metadata(&cache_path) {
        if let Ok(modified) = meta.modified() {
            if modified.elapsed().map(|d| d.as_secs() < TTL_SECS).unwrap_or(false) {
                if let Ok(raw) = fs::read_to_string(&cache_path) {
                    if let Ok(cached) = serde_json::from_str::<Vec<crate::models::VersionInfo>>(&raw) {
                        if !cached.is_empty() {
                            return Ok(cached);
                        }
                    }
                }
            }
        }
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let manifest: Value = client
        .get(crate::download::rewrite_url(
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
        ))
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let mut versions = Vec::new();
    if let Some(arr) = manifest.get("versions").and_then(|v| v.as_array()) {
        for v in arr {
            versions.push(crate::models::VersionInfo {
                id: v.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string(),
                type_: v.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                release_time: v
                    .get("releaseTime")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
    }
    if versions.is_empty() {
        versions = vec![crate::models::VersionInfo {
            id: "1.21.1".into(),
            type_: "release".into(),
            release_time: String::new(),
        }];
    }
    if let Ok(raw) = serde_json::to_string(&versions) {
        let _ = fs::write(cache_path, raw);
    }
    Ok(versions)
}

fn version_meta_url(game_version: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let manifest: Value = client
        .get(crate::download::rewrite_url(
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
        ))
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    for v in manifest["versions"].as_array().into_iter().flatten() {
        if v["id"].as_str() == Some(game_version) {
            return Ok(v["url"].as_str().unwrap_or("").to_string());
        }
    }
    Err(format!("Version {game_version} not found in manifest"))
}

pub fn prepare_instance(app: AppHandle, id: String) -> Result<String, String> {
    match prepare_instance_inner(&app, &id) {
        Ok(summary) => Ok(summary),
        Err(e) => {
            crate::download::emit_idle(Some(&app), format!("Download failed: {e}"));
            Err(e)
        }
    }
}

fn prepare_instance_inner(app: &AppHandle, id: &str) -> Result<String, String> {
    let inst = get_instance(id)?;
    let meta = meta_dir()?;
    let versions_dir = meta.join("versions").join(&inst.game_version);
    fs::create_dir_all(&versions_dir).map_err(|e| e.to_string())?;

    emit_progress(
        Some(app),
        DownloadProgress {
            phase: "client".into(),
            done: 0,
            total: 1,
            failed: 0,
            current_file: Some(inst.game_version.clone()),
            bytes_per_sec: None,
            message: format!("Preparing {}", inst.game_version),
            active: true,
        },
    );

    let version_json_path = versions_dir.join(format!("{}.json", inst.game_version));
    if !version_json_path.exists() {
        let url = version_meta_url(&inst.game_version)?;
        download_file(&url, &version_json_path)?;
    }

    let version: Value =
        serde_json::from_str(&fs::read_to_string(&version_json_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    if let Some(client) = version.get("downloads").and_then(|d| d.get("client")) {
        let url = client.get("url").and_then(|u| u.as_str()).unwrap_or("");
        let jar = versions_dir.join(format!("{}.jar", inst.game_version));
        if !url.is_empty() {
            download_file(url, &jar)?;
        }
    }

    let libraries = meta.join("libraries");
    fs::create_dir_all(&libraries).map_err(|e| e.to_string())?;

    // Copy loader profile early so we can download its maven libraries too.
    let inst_version = minecraft_dir(id)?.join("versions").join(instance_version_id(&inst));
    fs::create_dir_all(&inst_version).map_err(|e| e.to_string())?;
    let profile_path = instance_dir_profile(&inst)?;
    let profile_json: Option<Value> = if profile_path.exists() {
        fs::copy(
            &profile_path,
            inst_version.join(format!("{}.json", instance_version_id(&inst))),
        )
        .map_err(|e| e.to_string())?;
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap_or_default()).ok()
    } else {
        fs::copy(
            &version_json_path,
            inst_version.join(format!("{}.json", instance_version_id(&inst))),
        )
        .map_err(|e| e.to_string())?;
        let jar_src = versions_dir.join(format!("{}.jar", inst.game_version));
        let jar_dst = inst_version.join(format!("{}.jar", instance_version_id(&inst)));
        if jar_src.exists() && !jar_dst.exists() {
            fs::copy(&jar_src, &jar_dst).map_err(|e| e.to_string())?;
        }
        None
    };

    let natives = meta.join("natives").join(&inst.game_version);
    let (lib_ok, lib_fail) = crate::libraries::ensure_libraries(
        Some(app),
        &version,
        profile_json.as_ref(),
        &libraries,
        &natives,
    )?;

    // Asset index + parallel objects — emit phase immediately so UI isn't stuck on "libraries done"
    let mut asset_ok = 0usize;
    let mut asset_fail = 0usize;
    emit_progress(
        Some(app),
        DownloadProgress {
            phase: "assets".into(),
            done: 0,
            total: 0,
            failed: 0,
            current_file: None,
            bytes_per_sec: None,
            message: "Assets: fetching index…".into(),
            active: true,
        },
    );
    if let Some(asset_index) = version.get("assetIndex") {
        let index_id = asset_index.get("id").and_then(|i| i.as_str()).unwrap_or("legacy");
        let url = asset_index.get("url").and_then(|u| u.as_str()).unwrap_or("");
        let index_path = meta.join("assets").join("indexes").join(format!("{index_id}.json"));
        if !url.is_empty() {
            download_file(url, &index_path)?;
        }
        if index_path.exists() {
            let index: Value = serde_json::from_str(
                &fs::read_to_string(&index_path).unwrap_or_default(),
            )
            .unwrap_or(Value::Null);
            let objects_dir = meta.join("assets").join("objects");
            fs::create_dir_all(&objects_dir).ok();
            let mut jobs = Vec::new();
            let mut already = 0usize;
            if let Some(objects) = index.get("objects").and_then(|o| o.as_object()) {
                for (_name, obj) in objects {
                    let hash = obj.get("hash").and_then(|h| h.as_str()).unwrap_or("");
                    if hash.len() < 2 {
                        continue;
                    }
                    let prefix = &hash[..2];
                    let dest = objects_dir.join(prefix).join(hash);
                    if dest.exists()
                        && fs::metadata(&dest).map(|m| m.len() > 0).unwrap_or(false)
                    {
                        already += 1;
                        continue;
                    }
                    let url = format!("https://resources.download.minecraft.net/{prefix}/{hash}");
                    jobs.push((url, dest));
                }
            }
            let total_objects = already + jobs.len();
            emit_progress(
                Some(app),
                DownloadProgress {
                    phase: "assets".into(),
                    done: already,
                    total: total_objects,
                    failed: 0,
                    current_file: None,
                    bytes_per_sec: None,
                    message: format!(
                        "Assets: {already}/{total_objects} present, downloading {}…",
                        jobs.len()
                    ),
                    active: true,
                },
            );
            let (ok, fail) = download_many_progress(
                jobs,
                Some(app),
                "assets",
                already,
                Some(total_objects),
            )?;
            asset_ok = already + ok;
            asset_fail = fail;
        }
    }

    let summary = format!(
        "Prepared {} ({}) — libs ok/fail {}/{}, assets ok/fail {}/{}",
        inst.name, inst.game_version, lib_ok, lib_fail, asset_ok, asset_fail
    );
    emit_progress(
        Some(app),
        DownloadProgress {
            phase: "done".into(),
            done: lib_ok + asset_ok,
            total: lib_ok + lib_fail + asset_ok + asset_fail,
            failed: lib_fail + asset_fail,
            current_file: None,
            bytes_per_sec: None,
            message: summary.clone(),
            active: false,
        },
    );
    Ok(summary)
}

fn instance_version_id(inst: &crate::models::Instance) -> String {
    match inst.loader {
        LoaderKind::Vanilla => inst.game_version.clone(),
        _ => format!(
            "{}-{}-{}",
            inst.game_version,
            inst.loader.as_str(),
            inst.loader_version.as_deref().unwrap_or("latest")
        ),
    }
}

fn instance_dir_profile(inst: &crate::models::Instance) -> Result<PathBuf, String> {
    Ok(crate::paths::instance_dir(&inst.id)?
        .join("patches")
        .join("version.json"))
}

pub fn launch_instance(
    app: AppHandle,
    id: String,
    req_override: bool,
    server: Option<String>,
) -> Result<String, String> {
    let mut inst = get_instance(&id)?;

    if !req_override {
        let scan = reqguard::scan_instance(&id)?;
        let hard = scan
            .issues
            .iter()
            .any(|i| matches!(i.severity, crate::models::IssueSeverity::Error));
        if hard {
            let count = scan
                .issues
                .iter()
                .filter(|i| matches!(i.severity, crate::models::IssueSeverity::Error))
                .count();
            for issue in scan.issues.iter().filter(|i| {
                matches!(i.severity, crate::models::IssueSeverity::Error)
            }) {
                crate::console_log::append(Some(&app), issue.message.clone(), "error");
            }
            let msg = format!(
                "ReqGuard blocked launch: {count} error(s). Enable “Override ReqGuard” to force start, or fix dependencies."
            );
            crate::console_log::append(Some(&app), msg.clone(), "error");
            return Err(msg);
        }
    }

    let account = active_account()?.ok_or("No account. Add Microsoft, LittleSkin, or offline under Accounts.")?;
    let user_type = match account.kind {
        crate::models::AccountKind::Offline => "legacy",
        crate::models::AccountKind::Microsoft => "msa",
        crate::models::AccountKind::LittleSkin => "mojang",
    };
    let java = resolve_java(id.clone())?;
    let mc = minecraft_dir(&id)?;
    let meta = meta_dir()?;

    // Ensure Fabric/Quilt/Forge profile exists, then prepare libraries/assets.
    if !matches!(inst.loader, LoaderKind::Vanilla) {
        let profile_path = instance_dir_profile(&inst)?;
        if !profile_path.exists() {
            crate::loaders::install_loader(id.clone())?;
            inst = get_instance(&id)?;
        }
    }

    let version_id = instance_version_id(&inst);
    let version_json = mc
        .join("versions")
        .join(&version_id)
        .join(format!("{version_id}.json"));
    if !version_json.exists() {
        prepare_instance(app.clone(), id.clone())?;
    }
    // Re-copy profile into version folder if prepare ran before loader install
    if let Ok(profile_path) = instance_dir_profile(&inst) {
        if profile_path.exists() {
            let _ = fs::create_dir_all(mc.join("versions").join(&version_id));
            let _ = fs::copy(
                &profile_path,
                mc.join("versions")
                    .join(&version_id)
                    .join(format!("{version_id}.json")),
            );
        }
    }
    if !version_json.exists() {
        return Err(format!(
            "Version profile missing after prepare: {}",
            version_json.display()
        ));
    }
    let version: Value = serde_json::from_str(
        &fs::read_to_string(&version_json).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let vanilla_json_path = meta
        .join("versions")
        .join(&inst.game_version)
        .join(format!("{}.json", inst.game_version));
    let vanilla: Value = if vanilla_json_path.exists() {
        serde_json::from_str(&fs::read_to_string(&vanilla_json_path).unwrap_or_default())
            .unwrap_or(Value::Null)
    } else {
        Value::Null
    };

    let libraries = meta.join("libraries");
    let natives = meta.join("natives").join(&inst.game_version);
    // Download vanilla + loader (Fabric/Quilt) libraries and extract natives.
    if vanilla.is_null() {
        let _ = crate::libraries::ensure_libraries(Some(&app), &version, None, &libraries, &natives);
    } else {
        let _ = crate::libraries::ensure_libraries(
            Some(&app),
            &vanilla,
            Some(&version),
            &libraries,
            &natives,
        );
    }

    let mut classpath = crate::libraries::classpath_entries(&vanilla, &libraries);
    for p in crate::libraries::classpath_entries(&version, &libraries) {
        if !classpath.iter().any(|c| c == &p) {
            classpath.push(p);
        }
    }

    let client_jar = mc
        .join("versions")
        .join(&version_id)
        .join(format!("{version_id}.jar"));
    let vanilla_jar = meta
        .join("versions")
        .join(&inst.game_version)
        .join(format!("{}.jar", inst.game_version));
    if client_jar.exists() {
        classpath.push(client_jar);
    } else if vanilla_jar.exists() {
        classpath.push(vanilla_jar.clone());
        // Fabric profiles often omit the client jar in the version folder — copy for next time
        let _ = fs::create_dir_all(mc.join("versions").join(&version_id));
        let _ = fs::copy(&vanilla_jar, mc.join("versions").join(&version_id).join(format!("{version_id}.jar")));
    }

    if classpath.is_empty() {
        return Err(
            "Classpath is empty — run prepare/download again (libraries missing).".into(),
        );
    }

    let main_class = version
        .get("mainClass")
        .and_then(|m| m.as_str())
        .or_else(|| vanilla.get("mainClass").and_then(|m| m.as_str()))
        .unwrap_or("net.minecraft.client.main.Main");

    // Fabric/Quilt require their loader jar on the classpath.
    if main_class.contains("fabricmc") || main_class.contains("quiltmc") {
        let has_loader = classpath.iter().any(|p| {
            let s = p.to_string_lossy().to_lowercase();
            s.contains("fabric-loader") || s.contains("quilt-loader")
        });
        if !has_loader {
            return Err(
                "Fabric/Quilt loader jar missing from classpath. Reinstall the loader (Versions → Advanced → Reinstall loader), then Start again."
                    .into(),
            );
        }
    }

    let cp = classpath
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(if cfg!(windows) { ";" } else { ":" });

    fs::create_dir_all(&natives).ok();

    let asset_index = version
        .pointer("/assetIndex/id")
        .or_else(|| vanilla.pointer("/assetIndex/id"))
        .and_then(|v| v.as_str())
        .unwrap_or("1.21");

    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("natives_directory", natives.to_string_lossy().to_string());
    vars.insert("launcher_name", "EUML".into());
    vars.insert("launcher_version", "0.1.0".into());
    vars.insert("classpath", cp.clone());
    vars.insert(
        "classpath_separator",
        if cfg!(windows) {
            ";".into()
        } else {
            ":".into()
        },
    );
    vars.insert("library_directory", libraries.to_string_lossy().to_string());
    vars.insert("auth_player_name", account.username.clone());
    vars.insert("auth_uuid", account.uuid.clone());
    vars.insert("auth_access_token", account.access_token.clone());
    vars.insert("user_type", user_type.into());
    vars.insert("version_name", version_id.clone());
    vars.insert("game_directory", mc.to_string_lossy().to_string());
    vars.insert("assets_root", meta.join("assets").to_string_lossy().to_string());
    vars.insert("game_assets", meta.join("assets").to_string_lossy().to_string());
    vars.insert("assets_index_name", asset_index.to_string());
    vars.insert("version_type", "release".into());
    vars.insert("clientid", String::new());
    vars.insert("auth_xuid", String::new());
    vars.insert("user_properties", "{}".into());

    // Prefer modern arguments from vanilla + overlay (Fabric/Quilt); fall back to legacy list.
    let mut jvm_args = Vec::new();
    jvm_args.push(format!("-Xmx{}M", inst.memory_mb));
    jvm_args.push(format!("-Xms{}M", (inst.memory_mb / 2).max(512)));
    if !inst.jvm_args.trim().is_empty() {
        jvm_args.extend(inst.jvm_args.split_whitespace().map(|s| s.to_string()));
    }

    let mut from_json_jvm = collect_feature_args(&vanilla, "jvm");
    from_json_jvm.extend(collect_feature_args(&version, "jvm"));
    let mut from_json_game = collect_feature_args(&vanilla, "game");
    // Loader profile game args (usually empty for Fabric)
    from_json_game.extend(collect_feature_args(&version, "game"));

    let use_json_args = !from_json_jvm.is_empty() || !from_json_game.is_empty();

    let mut args: Vec<String> = Vec::new();
    if use_json_args {
        for a in jvm_args {
            args.push(a);
        }
        let mut skip_next = false;
        for a in from_json_jvm {
            if skip_next {
                skip_next = false;
                continue;
            }
            let raw = a.trim();
            // Skip -cp / -classpath and the following value; we append a clean pair later.
            if raw == "-cp" || raw == "-classpath" {
                skip_next = true;
                continue;
            }
            if raw.contains("${classpath}") {
                continue;
            }
            let s = normalize_jvm_arg(&substitute_vars(&a, &vars), &cp);
            if s.is_empty() {
                continue;
            }
            // Expanded classpath value without a preceding -cp (common Mojang template pair)
            if s == cp || looks_like_classpath(&s) {
                continue;
            }
            args.push(s);
        }
        // Prefer a clean FabricMcEmu (JSON sometimes has spaces around the value)
        args.retain(|a| !a.starts_with("-DFabricMcEmu="));
        if main_class.contains("fabricmc") {
            args.push("-DFabricMcEmu=net.minecraft.client.main.Main".into());
        }
        args.push("-cp".into());
        args.push(cp.clone());
        args.push(main_class.to_string());
        for a in from_json_game {
            args.push(substitute_vars(&a, &vars));
        }
    } else {
        args.extend(jvm_args);
        args.push(format!("-Djava.library.path={}", natives.to_string_lossy()));
        // Fabric emulator property when using Knot without full arg merge
        if main_class.contains("fabricmc") {
            args.push("-DFabricMcEmu=net.minecraft.client.main.Main".into());
        }
        args.push("-cp".into());
        args.push(cp.clone());
        args.push(main_class.to_string());
        args.push("--username".into());
        args.push(account.username.clone());
        args.push("--uuid".into());
        args.push(account.uuid.clone());
        args.push("--accessToken".into());
        args.push(account.access_token.clone());
        args.push("--version".into());
        args.push(version_id.clone());
        args.push("--gameDir".into());
        args.push(mc.to_string_lossy().to_string());
        args.push("--assetsDir".into());
        args.push(meta.join("assets").to_string_lossy().to_string());
        args.push("--assetIndex".into());
        args.push(asset_index.to_string());
        args.push("--userType".into());
        args.push(user_type.into());
        args.push("--versionType".into());
        args.push("release".into());
    }

    // Ensure Fabric emulator property is present for KnotClient
    if main_class.contains("fabricmc")
        && !args.iter().any(|a| a.starts_with("-DFabricMcEmu="))
    {
        if let Some(pos) = args.iter().position(|a| a == "-cp") {
            args.insert(pos, "-DFabricMcEmu=net.minecraft.client.main.Main".into());
        }
    }

    if let Some(addr) = server.filter(|s| !s.trim().is_empty()) {
        let addr = addr.trim().to_string();
        args.push("--quickPlayMultiplayer".into());
        args.push(addr.clone());
        let (host, port) = match addr.rsplit_once(':') {
            Some((h, p)) if p.parse::<u16>().is_ok() => (h.to_string(), p.to_string()),
            _ => (addr, "25565".into()),
        };
        args.push("--server".into());
        args.push(host);
        args.push("--port".into());
        args.push(port);
    }

    if !inst.pre_command.trim().is_empty() {
        let _ = Command::new("cmd")
            .args(["/C", &inst.pre_command])
            .current_dir(&mc)
            .status();
    }

    let launch_log = mc.join("logs");
    fs::create_dir_all(&launch_log).ok();
    let _ = fs::write(
        launch_log.join("euml-last-launch.txt"),
        format!("{java}\n{}\n", args.join("\n")),
    );

    crate::console_log::append(
        Some(&app),
        format!("Spawning Minecraft ({main_class})…"),
        "info",
    );

    let mut cmd = Command::new(&java);
    cmd.args(&args).current_dir(&mc);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    for pair in inst.env_vars.split(';') {
        if let Some((k, v)) = pair.split_once('=') {
            cmd.env(k.trim(), v.trim());
        }
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| {
            let msg = format!("Failed to spawn Minecraft ({java}): {e}");
            crate::console_log::append(Some(&app), msg.clone(), "error");
            msg
        })?;

    // Tee stdout/stderr to files + console
    let stdout_path = launch_log.join("euml-last-stdout.txt");
    let stderr_path = launch_log.join("euml-last-stderr.txt");
    if let Some(stdout) = child.stdout.take() {
        let app_c = app.clone();
        let path = stdout_path.clone();
        std::thread::spawn(move || tee_process_output(stdout, path, app_c, "game"));
    }
    if let Some(stderr) = child.stderr.take() {
        let app_c = app.clone();
        let path = stderr_path.clone();
        std::thread::spawn(move || tee_process_output(stderr, path, app_c, "game"));
    }

    // Detect instant crash (process dies within ~2s)
    std::thread::sleep(std::time::Duration::from_millis(1800));
    match child.try_wait() {
        Ok(Some(status)) => {
            let tail = fs::read_to_string(&stderr_path)
                .unwrap_or_default()
                .lines()
                .rev()
                .take(12)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            let msg = if tail.trim().is_empty() {
                format!(
                    "Minecraft exited immediately ({status}). Check logs in {}",
                    launch_log.display()
                )
            } else {
                format!("Minecraft exited immediately ({status}):\n{tail}")
            };
            crate::console_log::append(Some(&app), msg.clone(), "error");
            return Err(msg);
        }
        Ok(None) => {
            crate::console_log::append(
                Some(&app),
                format!("Game process running (pid {:?})", child.id()),
                "info",
            );
        }
        Err(e) => {
            crate::console_log::append(
                Some(&app),
                format!("Could not poll game process: {e}"),
                "warn",
            );
        }
    }

    // Detach — don't wait on the child for the rest of the session
    drop(child);

    if !inst.post_command.trim().is_empty() {
        let _ = fs::write(mc.join("logs").join("euml-post-pending.txt"), &inst.post_command);
    }

    inst.last_played = Some(Utc::now().to_rfc3339());
    save_instance(&inst)?;

    let override_note = if req_override {
        " (ReqGuard override)"
    } else {
        ""
    };
    let summary = format!(
        "Launched {} as {}{override_note}",
        inst.name, account.username
    );
    crate::console_log::append(Some(&app), summary.clone(), "info");
    Ok(summary)
}

fn tee_process_output(
    mut reader: impl std::io::Read + Send + 'static,
    path: PathBuf,
    app: AppHandle,
    level: &'static str,
) {
    use std::io::{BufRead, BufReader, Write};
    let mut file = fs::File::create(&path).ok();
    let buf = BufReader::new(&mut reader);
    for line in buf.lines() {
        let Ok(line) = line else { break };
        if let Some(f) = file.as_mut() {
            let _ = writeln!(f, "{line}");
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lvl = if trimmed.contains("ERROR")
            || trimmed.contains("Exception")
            || trimmed.contains("FATAL")
        {
            "error"
        } else if trimmed.contains("WARN") {
            "warn"
        } else {
            level
        };
        crate::console_log::append(Some(&app), trimmed.to_string(), lvl);
    }
}

fn collect_feature_args(version: &Value, kind: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(arr) = version
        .pointer(&format!("/arguments/{kind}"))
        .and_then(|a| a.as_array())
    else {
        return out;
    };
    for item in arr {
        match item {
            Value::String(s) => out.push(s.clone()),
            Value::Object(obj) => {
                if !argument_allowed(obj) {
                    continue;
                }
                match obj.get("value") {
                    Some(Value::String(s)) => out.push(s.clone()),
                    Some(Value::Array(a)) => {
                        for v in a {
                            if let Some(s) = v.as_str() {
                                out.push(s.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    out
}

fn argument_allowed(obj: &serde_json::Map<String, Value>) -> bool {
    let Some(rules) = obj.get("rules").and_then(|r| r.as_array()) else {
        return true;
    };
    let mut allowed = false;
    for rule in rules {
        let action = rule.get("action").and_then(|a| a.as_str()).unwrap_or("allow");
        let os_name = rule.pointer("/os/name").and_then(|n| n.as_str());
        let os_arch = rule.pointer("/os/arch").and_then(|n| n.as_str());
        let matches_os = match os_name {
            None => true,
            Some("windows") => cfg!(windows),
            Some("osx") => cfg!(target_os = "macos"),
            Some("linux") => cfg!(target_os = "linux"),
            _ => false,
        };
        let matches_arch = match os_arch {
            None => true,
            Some("x86") => cfg!(target_arch = "x86"),
            Some("x86_64") | Some("amd64") => cfg!(target_arch = "x86_64"),
            Some("arm64") | Some("aarch64") => cfg!(target_arch = "aarch64"),
            _ => true,
        };
        // features (is_demo_user, has_custom_resolution, etc.) — skip if required
        let features = rule.get("features").and_then(|f| f.as_object());
        let matches_features = match features {
            None => true,
            Some(f) => {
                // We don't enable demo / custom resolution features by default
                !f.values().any(|v| v.as_bool() == Some(true))
            }
        };
        if matches_os && matches_arch && matches_features {
            allowed = action == "allow";
        }
    }
    allowed
}

fn substitute_vars(input: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = input.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("${{{k}}}"), v);
    }
    out
}

/// Trim junk in -Dkey= value forms and drop empty tokens.
fn normalize_jvm_arg(s: &str, classpath: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Some((key, val)) = s.split_once('=') {
        if key.starts_with("-D") {
            return format!("{}={}", key.trim(), val.trim());
        }
    }
    if s == classpath {
        return String::new();
    }
    s.to_string()
}

fn looks_like_classpath(s: &str) -> bool {
    if s.starts_with('-') {
        return false;
    }
    let jars = s.matches(".jar").count();
    jars >= 3 && (s.contains(';') || s.contains(':'))
}
