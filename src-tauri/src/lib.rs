mod auth;
mod config_editor;
mod console_log;
mod content;
mod dedicated;
mod download;
mod favorites;
mod folders;
mod host_files;
mod host_process;
mod host_stats;
mod icons;
mod import;
mod instances;
mod java;
mod launch;
mod libraries;
mod loaders;
mod models;
mod mods_platform;
mod news;
mod paths;
mod reqguard;
mod servers;
mod upnp;
mod world_settings;

use models::{Instance, LauncherSettings, ModEntry};

#[tauri::command]
fn list_instances() -> Result<Vec<Instance>, String> {
    instances::list_instances()
}

#[tauri::command]
fn get_instance(id: String) -> Result<Instance, String> {
    instances::get_instance(&id)
}

#[tauri::command]
fn create_instance(
    name: String,
    game_version: String,
    loader: String,
    loader_version: Option<String>,
    memory_mb: u32,
    folder: Option<String>,
) -> Result<Instance, String> {
    instances::create_instance(name, game_version, loader, loader_version, memory_mb, folder)
}

#[tauri::command]
fn update_instance(instance: Instance) -> Result<Instance, String> {
    instances::update_instance(instance)
}

#[tauri::command]
fn move_instance(id: String, folder: Option<String>) -> Result<Instance, String> {
    instances::move_instance(id, folder)
}

#[tauri::command]
fn delete_instance(id: String) -> Result<(), String> {
    instances::delete_instance(&id)
}

#[tauri::command]
fn open_instance_folder(id: String) -> Result<(), String> {
    instances::open_instance_folder(&id)
}

#[tauri::command]
fn list_folders() -> Result<Vec<models::InstanceFolder>, String> {
    folders::list_folders()
}

#[tauri::command]
fn create_folder(name: String) -> Result<models::InstanceFolder, String> {
    folders::create_folder(name)
}

#[tauri::command]
fn rename_folder(id: String, name: String) -> Result<Vec<models::InstanceFolder>, String> {
    folders::rename_folder(id, name)
}

#[tauri::command]
fn delete_folder(id: String) -> Result<Vec<models::InstanceFolder>, String> {
    folders::delete_folder(id)
}

#[tauri::command]
fn open_disk_folder(name: String) -> Result<(), String> {
    folders::open_folder(name)
}

#[tauri::command]
fn get_settings() -> Result<LauncherSettings, String> {
    paths::load_settings()
}

#[tauri::command]
fn save_settings(settings: LauncherSettings) -> Result<LauncherSettings, String> {
    paths::save_settings(&settings)?;
    Ok(settings)
}

#[tauri::command]
fn list_accounts() -> Result<Vec<models::Account>, String> {
    auth::list_accounts()
}

#[tauri::command]
fn begin_ms_login() -> Result<auth::DeviceLoginStart, String> {
    auth::begin_ms_login()
}

#[tauri::command]
fn poll_ms_login(device_code: String) -> Result<Option<models::Account>, String> {
    auth::poll_ms_login(device_code)
}

#[tauri::command]
fn select_account(id: String) -> Result<Vec<models::Account>, String> {
    auth::select_account(id)
}

#[tauri::command]
fn remove_account(id: String) -> Result<Vec<models::Account>, String> {
    auth::remove_account(id)
}

#[tauri::command]
fn add_offline_account(username: String) -> Result<Vec<models::Account>, String> {
    auth::add_offline_account(username)
}

#[tauri::command]
fn add_littleskin_account(email: String, password: String) -> Result<Vec<models::Account>, String> {
    auth::add_littleskin_account(email, password)
}

#[tauri::command]
fn list_game_versions() -> Result<Vec<String>, String> {
    launch::list_game_versions()
}

#[tauri::command]
fn list_game_versions_detailed() -> Result<Vec<models::VersionInfo>, String> {
    launch::list_game_versions_detailed()
}

#[tauri::command]
fn resolve_java(instance_id: String) -> Result<String, String> {
    java::resolve_java(instance_id)
}

#[tauri::command]
fn detect_java_installs() -> Result<Vec<String>, String> {
    java::detect_java_installs()
}

#[tauri::command]
async fn prepare_instance(app: tauri::AppHandle, id: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || launch::prepare_instance(app, id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_world_settings(instance_id: String, world_name: String) -> Result<world_settings::WorldSettings, String> {
    world_settings::get_world_settings(instance_id, world_name)
}

#[tauri::command]
fn save_world_settings(
    instance_id: String,
    settings: world_settings::WorldSettings,
) -> Result<world_settings::WorldSettings, String> {
    world_settings::save_world_settings(instance_id, settings)
}

#[tauri::command]
fn list_instance_configs(instance_id: String) -> Result<Vec<String>, String> {
    world_settings::list_instance_configs(instance_id)
}

#[tauri::command]
fn read_instance_text_file(instance_id: String, relative: String) -> Result<String, String> {
    world_settings::read_instance_text_file(instance_id, relative)
}

#[tauri::command]
fn write_instance_text_file(
    instance_id: String,
    relative: String,
    contents: String,
) -> Result<(), String> {
    world_settings::write_instance_text_file(instance_id, relative, contents)
}

#[tauri::command]
async fn launch_instance(
    app: tauri::AppHandle,
    id: String,
    req_override: bool,
    server: Option<String>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || launch::launch_instance(app, id, req_override, server))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn install_loader(id: String) -> Result<Instance, String> {
    loaders::install_loader(id)
}

#[tauri::command]
async fn search_mods(
    query: String,
    game_version: String,
    loader: String,
    categories: Option<Vec<String>>,
) -> Result<Vec<mods_platform::ModrinthHit>, String> {
    tokio::task::spawn_blocking(move || {
        mods_platform::search_mods(query, game_version, loader, categories)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn search_content(
    query: String,
    game_version: String,
    loader: String,
    project_type: String,
    categories: Option<Vec<String>>,
) -> Result<Vec<mods_platform::ModrinthHit>, String> {
    tokio::task::spawn_blocking(move || {
        mods_platform::search_content(query, game_version, loader, project_type, categories)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_modrinth_project(
    project_id: String,
    game_version: String,
    loader: String,
) -> Result<mods_platform::ModrinthProjectDetails, String> {
    tokio::task::spawn_blocking(move || {
        mods_platform::get_modrinth_project(project_id, game_version, loader)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn update_instance_mods(instance_id: String) -> Result<Vec<mods_platform::ModUpdateResult>, String> {
    tokio::task::spawn_blocking(move || mods_platform::update_instance_mods(instance_id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn parse_config_file(relative: String, contents: String) -> config_editor::ParsedConfig {
    config_editor::parse_config(relative, contents)
}

#[tauri::command]
fn apply_config_fields(
    relative: String,
    original: String,
    fields: Vec<config_editor::ConfigField>,
) -> Result<String, String> {
    config_editor::apply_config_fields(relative, original, fields)
}

#[tauri::command]
fn configs_for_mod(instance_id: String, mod_file_name: String) -> Result<Vec<String>, String> {
    config_editor::configs_for_mod(instance_id, mod_file_name)
}

#[tauri::command]
async fn fetch_minecraft_news() -> Result<Vec<news::MinecraftNewsItem>, String> {
    tokio::task::spawn_blocking(news::fetch_minecraft_news)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn fetch_minecraft_patch_notes() -> Result<Vec<news::MinecraftPatchNote>, String> {
    tokio::task::spawn_blocking(news::fetch_minecraft_patch_notes)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn set_instance_icon(id: String, image_path: String) -> Result<models::Instance, String> {
    let mut inst = instances::get_instance(&id)?;
    let bytes = std::fs::read(&image_path).map_err(|e| e.to_string())?;
    // reuse icons helper style data url
    let b64 = {
        const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
            let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(T[((n >> 18) & 63) as usize] as char);
            out.push(T[((n >> 12) & 63) as usize] as char);
            out.push(if chunk.len() > 1 {
                T[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                T[(n & 63) as usize] as char
            } else {
                '='
            });
        }
        out
    };
    let mime = if image_path.to_lowercase().ends_with(".jpg") || image_path.to_lowercase().ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "image/png"
    };
    inst.icon_path = Some(format!("data:{mime};base64,{b64}"));
    instances::save_instance(&inst)?;
    Ok(inst)
}

#[tauri::command]
fn reinstall_loader(id: String) -> Result<models::Instance, String> {
    loaders::install_loader(id)
}

#[tauri::command]
fn change_instance_version(
    id: String,
    game_version: String,
    loader: String,
    loader_version: Option<String>,
) -> Result<models::Instance, String> {
    let mut inst = instances::get_instance(&id)?;
    let raw_gv = game_version.clone();
    inst.loader = models::LoaderKind::from_str_loose(&loader);
    inst.loader = models::infer_loader(&inst.name, &raw_gv, inst.loader.clone());
    inst.game_version = models::normalize_game_version(&raw_gv);
    inst.loader_version = loader_version;
    instances::save_instance(&inst)?;
    if inst.loader != models::LoaderKind::Vanilla {
        let _ = loaders::install_loader(inst.id.clone());
        inst = instances::get_instance(&id)?;
    }
    Ok(inst)
}

#[tauri::command]
fn install_mod(instance_id: String, project_id: String, version_id: String) -> Result<ModEntry, String> {
    mods_platform::install_mod(instance_id, project_id, version_id)
}

#[tauri::command]
fn install_content_from_modrinth(
    instance_id: String,
    version_id: String,
    kind: String,
    world_name: Option<String>,
) -> Result<models::ContentItem, String> {
    mods_platform::install_content_from_modrinth(instance_id, version_id, kind, world_name)
}

#[tauri::command]
fn list_instance_mods(instance_id: String) -> Result<Vec<ModEntry>, String> {
    mods_platform::list_instance_mods(instance_id)
}

#[tauri::command]
fn set_mod_enabled(instance_id: String, file_name: String, enabled: bool) -> Result<Vec<ModEntry>, String> {
    mods_platform::set_mod_enabled(instance_id, file_name, enabled)
}

#[tauri::command]
fn uninstall_mod(instance_id: String, file_name: String) -> Result<Vec<ModEntry>, String> {
    mods_platform::uninstall_mod(instance_id, file_name)
}

#[tauri::command]
fn import_mrpack(path: String) -> Result<Instance, String> {
    mods_platform::import_mrpack(path)
}

#[tauri::command]
fn export_mrpack(instance_id: String, dest_path: String) -> Result<String, String> {
    mods_platform::export_mrpack(instance_id, dest_path)
}

#[tauri::command]
fn reqguard_scan(instance_id: String) -> Result<models::ReqScanResult, String> {
    reqguard::scan_instance(&instance_id)
}

#[tauri::command]
fn reqguard_resolve(instance_id: String, missing_mod_id: String) -> Result<models::ReqScanResult, String> {
    reqguard::resolve_missing(instance_id, missing_mod_id)
}

#[tauri::command]
fn list_content(instance_id: String, kind: String) -> Result<Vec<models::ContentItem>, String> {
    content::list_content(instance_id, kind)
}

#[tauri::command]
fn install_content_zip(
    instance_id: String,
    kind: String,
    zip_path: String,
) -> Result<Vec<models::ContentItem>, String> {
    content::install_content_zip(instance_id, kind, zip_path)
}

#[tauri::command]
fn delete_content(
    instance_id: String,
    kind: String,
    name: String,
) -> Result<Vec<models::ContentItem>, String> {
    content::delete_content(instance_id, kind, name)
}

#[tauri::command]
fn open_content_item(path: String) -> Result<(), String> {
    content::open_content_item(path)
}

#[tauri::command]
fn import_save(instance_id: String, src_path: String) -> Result<Vec<models::ContentItem>, String> {
    content::import_save(instance_id, src_path)
}

#[tauri::command]
fn list_worlds(instance_id: String) -> Result<Vec<models::ContentItem>, String> {
    content::list_worlds(instance_id)
}

#[tauri::command]
fn list_screenshots(instance_id: String) -> Result<Vec<models::ContentItem>, String> {
    content::list_screenshots(instance_id)
}

#[tauri::command]
fn list_datapacks(instance_id: String, world_name: String) -> Result<Vec<models::ContentItem>, String> {
    content::list_datapacks(instance_id, world_name)
}

#[tauri::command]
fn install_datapack(
    instance_id: String,
    world_name: String,
    src_path: String,
) -> Result<Vec<models::ContentItem>, String> {
    content::install_datapack(instance_id, world_name, src_path)
}

#[tauri::command]
fn delete_datapack(
    instance_id: String,
    world_name: String,
    name: String,
) -> Result<Vec<models::ContentItem>, String> {
    content::delete_datapack(instance_id, world_name, name)
}

#[tauri::command]
fn read_logs(instance_id: String) -> Result<Vec<models::LogLine>, String> {
    content::read_logs(instance_id)
}

#[tauri::command]
fn analyze_crash(instance_id: String) -> Result<Vec<models::CrashHint>, String> {
    content::analyze_crash(instance_id)
}

#[tauri::command]
fn import_foreign_instance(path: String, folder: Option<String>) -> Result<Instance, String> {
    import::import_foreign_instance(path, folder)
}

#[tauri::command]
fn import_instance_folder(path: String, folder: Option<String>) -> Result<Vec<Instance>, String> {
    import::import_instance_folder(path, folder)
}

#[tauri::command]
fn list_servers(instance_id: String) -> Result<Vec<servers::ServerEntry>, String> {
    servers::list_servers(instance_id)
}

#[tauri::command]
fn add_server(instance_id: String, name: String, ip: String) -> Result<Vec<servers::ServerEntry>, String> {
    servers::add_server(instance_id, name, ip)
}

#[tauri::command]
fn update_server(
    instance_id: String,
    index: usize,
    name: String,
    ip: String,
) -> Result<Vec<servers::ServerEntry>, String> {
    servers::update_server(instance_id, index, name, ip)
}

#[tauri::command]
fn remove_server(instance_id: String, index: usize) -> Result<Vec<servers::ServerEntry>, String> {
    servers::remove_server(instance_id, index)
}

#[tauri::command]
fn get_console_lines() -> Vec<console_log::ConsoleLine> {
    console_log::history()
}

#[tauri::command]
fn clear_console(app: tauri::AppHandle) {
    console_log::clear(Some(&app));
}

#[tauri::command]
fn append_console(app: tauri::AppHandle, text: String, level: Option<String>) {
    console_log::append(Some(&app), text, level.as_deref().unwrap_or("info"));
}

#[tauri::command]
fn list_favorites() -> Result<Vec<favorites::FavoriteEntry>, String> {
    favorites::list_favorites()
}

#[tauri::command]
fn toggle_favorite(
    id: String,
    kind: String,
    label: String,
    subtitle: Option<String>,
    icon_url: Option<String>,
) -> Result<Vec<favorites::FavoriteEntry>, String> {
    favorites::toggle_favorite(id, kind, label, subtitle, icon_url)
}

#[tauri::command]
fn remove_favorite(id: String) -> Result<Vec<favorites::FavoriteEntry>, String> {
    favorites::remove_favorite(id)
}

#[tauri::command]
fn open_console_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
    if let Some(existing) = app.get_webview_window("console") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(
        &app,
        "console",
        WebviewUrl::App("index.html?eumlWindow=console".into()),
    )
    .title("Northstar Console")
    .inner_size(860.0, 520.0)
    .min_inner_size(480.0, 280.0)
    .resizable(true)
    .build()
    .map_err(|e| format!("Failed to open console window: {e}"))?;
    Ok(())
}

#[tauri::command]
fn close_console_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(existing) = app.get_webview_window("console") {
        existing.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn list_dedicated() -> Result<Vec<dedicated::HostServer>, String> {
    dedicated::list_dedicated()
}

#[tauri::command]
fn create_dedicated(
    name: String,
    game_version: String,
    loader: String,
    memory_mb: Option<u32>,
    port: Option<u16>,
) -> Result<dedicated::HostServer, String> {
    dedicated::create_dedicated(name, game_version, loader, memory_mb, port)
}

#[tauri::command]
fn update_dedicated(server: dedicated::HostServer) -> Result<dedicated::HostServer, String> {
    dedicated::update_dedicated(server)
}

#[tauri::command]
async fn delete_dedicated(app: tauri::AppHandle, id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        host_process::stop_if_running(Some(&app), &id);
        dedicated::delete_dedicated(id)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn open_dedicated_folder(id: String) -> Result<(), String> {
    dedicated::open_dedicated_folder(id)
}

#[tauri::command]
async fn install_dedicated(id: String) -> Result<dedicated::HostServer, String> {
    tokio::task::spawn_blocking(move || dedicated::install_dedicated(id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn accept_dedicated_eula(id: String) -> Result<dedicated::HostServer, String> {
    dedicated::accept_dedicated_eula(id)
}

#[tauri::command]
fn get_dedicated_properties(id: String) -> Result<dedicated::DedicatedProperties, String> {
    dedicated::get_dedicated_properties(&id)
}

#[tauri::command]
fn set_dedicated_properties(
    id: String,
    props: dedicated::DedicatedProperties,
) -> Result<(), String> {
    dedicated::set_dedicated_properties(&id, props)
}

#[tauri::command]
fn get_dedicated_player_lists(id: String) -> Result<dedicated::PlayerLists, String> {
    dedicated::get_dedicated_player_lists(&id)
}

#[tauri::command]
fn set_dedicated_player_lists(id: String, lists: dedicated::PlayerLists) -> Result<(), String> {
    dedicated::set_dedicated_player_lists(&id, lists)
}

#[tauri::command]
async fn start_dedicated(
    app: tauri::AppHandle,
    id: String,
) -> Result<host_process::DedicatedStatus, String> {
    tokio::task::spawn_blocking(move || host_process::start_dedicated(app, id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn stop_dedicated(
    app: tauri::AppHandle,
    id: String,
) -> Result<host_process::DedicatedStatus, String> {
    tokio::task::spawn_blocking(move || host_process::stop_dedicated(app, id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn dedicated_status(id: String) -> Result<host_process::DedicatedStatus, String> {
    host_process::dedicated_status(id)
}

#[tauri::command]
fn dedicated_send_command(id: String, command: String) -> Result<(), String> {
    host_process::dedicated_send_command(id, command)
}

#[tauri::command]
async fn dedicated_upnp_map(id: String) -> Result<upnp::NetworkInfo, String> {
    tokio::task::spawn_blocking(move || {
        let server = dedicated::get_dedicated(&id)?;
        // Cascade: UPnP → NAT-PMP → PCP (relay not implemented)
        Ok(upnp::map_port_cascade(server.port))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dedicated_upnp_unmap(id: String) -> Result<upnp::NetworkInfo, String> {
    tokio::task::spawn_blocking(move || {
        let server = dedicated::get_dedicated(&id)?;
        let _ = upnp::unmap_port(server.port);
        Ok(upnp::network_info(server.port, false))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dedicated_network_info(id: String) -> Result<upnp::NetworkInfo, String> {
    tokio::task::spawn_blocking(move || {
        let server = dedicated::get_dedicated(&id)?;
        let status = host_process::dedicated_status(id).ok();
        let mapped = status.map(|s| s.upnp_mapped).unwrap_or(false)
            || upnp::mapping_method_for(server.port).is_some();
        let method = upnp::mapping_method_for(server.port);
        Ok(upnp::network_info_full(
            server.port,
            mapped,
            method,
            Vec::new(),
            false,
        ))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dedicated_firewall_rule(id: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let server = dedicated::get_dedicated(&id)?;
        upnp::try_add_firewall_rule(server.port)?;
        Ok(format!("Firewall rule added for TCP {}", server.port))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_dedicated_mrpack(
    id: String,
    path: String,
) -> Result<dedicated::HostServer, String> {
    tokio::task::spawn_blocking(move || dedicated::import_dedicated_mrpack(id, path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dedicated_live_stats(
    app: tauri::AppHandle,
    id: String,
) -> Result<host_stats::HostLiveStats, String> {
    tokio::task::spawn_blocking(move || host_stats::refresh_live_stats(Some(&app), &id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn dedicated_cpu_count() -> u32 {
    host_process::logical_cpu_count()
}

#[tauri::command]
async fn dedicated_upload_world(id: String, src_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || host_files::upload_world(id, src_path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dedicated_upload_mods(id: String, src_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || host_files::upload_mods(id, src_path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dedicated_download_world(id: String, dest_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || host_files::download_world_zip(id, dest_path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn dedicated_download_mods(id: String, dest_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || host_files::download_mods_zip(id, dest_path))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            list_instances,
            get_instance,
            create_instance,
            update_instance,
            move_instance,
            delete_instance,
            open_instance_folder,
            list_folders,
            create_folder,
            rename_folder,
            delete_folder,
            open_disk_folder,
            get_settings,
            save_settings,
            list_accounts,
            begin_ms_login,
            poll_ms_login,
            add_offline_account,
            add_littleskin_account,
            select_account,
            remove_account,
            list_game_versions,
            list_game_versions_detailed,
            resolve_java,
            detect_java_installs,
            prepare_instance,
            launch_instance,
            install_loader,
            search_mods,
            search_content,
            get_modrinth_project,
            update_instance_mods,
            parse_config_file,
            apply_config_fields,
            configs_for_mod,
            fetch_minecraft_news,
            fetch_minecraft_patch_notes,
            set_instance_icon,
            reinstall_loader,
            change_instance_version,
            install_mod,
            install_content_from_modrinth,
            list_instance_mods,
            set_mod_enabled,
            uninstall_mod,
            import_mrpack,
            export_mrpack,
            reqguard_scan,
            reqguard_resolve,
            list_content,
            install_content_zip,
            delete_content,
            open_content_item,
            import_save,
            list_worlds,
            list_screenshots,
            list_datapacks,
            install_datapack,
            delete_datapack,
            read_logs,
            analyze_crash,
            import_foreign_instance,
            import_instance_folder,
            list_servers,
            add_server,
            update_server,
            remove_server,
            get_console_lines,
            clear_console,
            append_console,
            open_console_window,
            close_console_window,
            list_favorites,
            toggle_favorite,
            remove_favorite,
            list_dedicated,
            create_dedicated,
            update_dedicated,
            delete_dedicated,
            open_dedicated_folder,
            install_dedicated,
            accept_dedicated_eula,
            get_dedicated_properties,
            set_dedicated_properties,
            get_dedicated_player_lists,
            set_dedicated_player_lists,
            start_dedicated,
            stop_dedicated,
            dedicated_status,
            dedicated_send_command,
            dedicated_upnp_map,
            dedicated_upnp_unmap,
            dedicated_network_info,
            dedicated_firewall_rule,
            import_dedicated_mrpack,
            dedicated_live_stats,
            dedicated_cpu_count,
            dedicated_upload_world,
            dedicated_upload_mods,
            dedicated_download_world,
            dedicated_download_mods,
            get_world_settings,
            save_world_settings,
            list_instance_configs,
            read_instance_text_file,
            write_instance_text_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Northstar");
}
