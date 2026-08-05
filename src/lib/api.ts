import { invoke } from "@tauri-apps/api/core";
import { cached, cacheInvalidate, cacheSet } from "./cache";
import type {
  Account,
  ContentItem,
  CrashHint,
  Instance,
  InstanceFolder,
  LauncherSettings,
  LogLine,
  ModEntry,
  ModrinthHit,
  ModrinthProjectType,
  ModUpdateResult,
  MinecraftNewsItem,
  MinecraftPatchNote,
  ParsedConfig,
  ModrinthProjectDetails,
  ReqScanResult,
  ServerEntry,
  FavoriteEntry,
  VersionInfo,
  WorldSettings,
  DedicatedServer,
  DedicatedStatus,
  DedicatedProperties,
  DedicatedPlayerLists,
  DedicatedNetworkInfo,
  HostLiveStats,
} from "./types";

const INSTANCES = "instances";
const ACCOUNTS = "accounts";
const SETTINGS = "settings";
const VERSIONS = "versions_detailed";
const JAVA = "java_installs";

function bumpInstances<T>(value: T): T {
  cacheInvalidate(INSTANCES);
  return value;
}

function bumpAccounts(list: Account[]): Account[] {
  return cacheSet(ACCOUNTS, list);
}

export const api = {
  listInstances: () => cached(INSTANCES, 15_000, () => invoke<Instance[]>("list_instances")),
  getInstance: (id: string) => invoke<Instance>("get_instance", { id }),
  createInstance: (payload: {
    name: string;
    gameVersion: string;
    loader: string;
    loaderVersion?: string | null;
    memoryMb?: number;
    folder?: string | null;
  }) =>
    invoke<Instance>("create_instance", {
      name: payload.name,
      gameVersion: payload.gameVersion,
      loader: payload.loader,
      loaderVersion: payload.loaderVersion ?? null,
      memoryMb: payload.memoryMb ?? 4096,
      folder: payload.folder ?? null,
    }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),
  updateInstance: (instance: Instance) =>
    invoke<Instance>("update_instance", { instance }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),
  moveInstance: (id: string, folder?: string | null) =>
    invoke<Instance>("move_instance", { id, folder: folder ?? null }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),
  deleteInstance: (id: string) =>
    invoke<void>("delete_instance", { id }).then(() => {
      cacheInvalidate(INSTANCES);
    }),
  openInstanceFolder: (id: string) => invoke<void>("open_instance_folder", { id }),
  listFolders: () => invoke<InstanceFolder[]>("list_folders"),
  createFolder: (name: string) => invoke<InstanceFolder>("create_folder", { name }),
  renameFolder: (id: string, name: string) => invoke<InstanceFolder[]>("rename_folder", { id, name }),
  deleteFolder: (id: string) =>
    invoke<InstanceFolder[]>("delete_folder", { id }).then((folders) => {
      cacheInvalidate(INSTANCES);
      return folders;
    }),
  openDiskFolder: (name: string) => invoke<void>("open_disk_folder", { name }),
  getSettings: () => cached(SETTINGS, 30_000, () => invoke<LauncherSettings>("get_settings")),
  saveSettings: (settings: LauncherSettings) =>
    invoke<LauncherSettings>("save_settings", { settings }).then((s) => cacheSet(SETTINGS, s)),

  listAccounts: () => cached(ACCOUNTS, 15_000, () => invoke<Account[]>("list_accounts")),
  beginMsLogin: () =>
    invoke<{ user_code: string; verification_uri: string; device_code: string; interval: number; expires_in: number }>(
      "begin_ms_login",
    ),
  pollMsLogin: (deviceCode: string) => invoke<Account | null>("poll_ms_login", { deviceCode }),
  addOfflineAccount: (username: string) =>
    invoke<Account[]>("add_offline_account", { username }).then(bumpAccounts),
  addLittleskinAccount: (email: string, password: string) =>
    invoke<Account[]>("add_littleskin_account", { email, password }).then(bumpAccounts),
  selectAccount: (id: string) => invoke<Account[]>("select_account", { id }).then(bumpAccounts),
  removeAccount: (id: string) => invoke<Account[]>("remove_account", { id }).then(bumpAccounts),
  /** Cached player-head data URL (fetched in Rust; works when CDN is blocked in the WebView). */
  resolveAccountAvatar: (kind: string, uuid: string, username: string) =>
    invoke<string | null>("resolve_account_avatar", { kind, uuid, username }),

  listVersions: () => invoke<string[]>("list_game_versions"),
  listVersionsDetailed: () =>
    cached(VERSIONS, 30 * 60_000, () => invoke<VersionInfo[]>("list_game_versions_detailed")),
  resolveJava: (instanceId: string) => invoke<string>("resolve_java", { instanceId }),
  prepareInstance: (id: string) => invoke<string>("prepare_instance", { id }).then(bumpInstances),
  launchInstance: (id: string, reqOverride?: boolean, server?: string | null) =>
    invoke<string>("launch_instance", {
      id,
      reqOverride: Boolean(reqOverride),
      server: server ?? null,
    }).then((msg) => {
      cacheInvalidate(INSTANCES);
      return msg;
    }),

  installLoader: (id: string) =>
    invoke<Instance>("install_loader", { id }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),

  searchMods: (query: string, gameVersion: string, loader: string, categories?: string[]) =>
    invoke<ModrinthHit[]>("search_mods", {
      query,
      gameVersion,
      loader,
      categories: categories ?? null,
    }),
  searchContent: (
    query: string,
    gameVersion: string,
    loader: string,
    projectType: ModrinthProjectType,
    categories?: string[],
  ) =>
    invoke<ModrinthHit[]>("search_content", {
      query,
      gameVersion,
      loader,
      projectType,
      categories: categories ?? null,
    }),
  getModrinthProject: (projectId: string, gameVersion: string, loader: string) =>
    invoke<ModrinthProjectDetails>("get_modrinth_project", { projectId, gameVersion, loader }),
  updateInstanceMods: (instanceId: string) =>
    invoke<ModUpdateResult[]>("update_instance_mods", { instanceId }),
  parseConfigFile: (relative: string, contents: string) =>
    invoke<ParsedConfig>("parse_config_file", { relative, contents }),
  applyConfigFields: (
    relative: string,
    original: string,
    fields: { key: string; value: string; value_type: string }[],
  ) => invoke<string>("apply_config_fields", { relative, original, fields }),
  configsForMod: (instanceId: string, modFileName: string) =>
    invoke<string[]>("configs_for_mod", { instanceId, modFileName }),
  fetchMinecraftNews: () => invoke<MinecraftNewsItem[]>("fetch_minecraft_news"),
  fetchMinecraftPatchNotes: () => invoke<MinecraftPatchNote[]>("fetch_minecraft_patch_notes"),
  setInstanceIcon: (id: string, imagePath: string) =>
    invoke<Instance>("set_instance_icon", { id, imagePath }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),
  reinstallLoader: (id: string) =>
    invoke<Instance>("reinstall_loader", { id }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),
  changeInstanceVersion: (
    id: string,
    gameVersion: string,
    loader: string,
    loaderVersion?: string | null,
  ) =>
    invoke<Instance>("change_instance_version", {
      id,
      gameVersion,
      loader,
      loaderVersion: loaderVersion ?? null,
    }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),
  installMod: (instanceId: string, projectId: string, versionId: string) =>
    invoke<ModEntry>("install_mod", { instanceId, projectId, versionId }),
  installContentFromModrinth: (
    instanceId: string,
    versionId: string,
    kind: string,
    worldName?: string | null,
  ) =>
    invoke<ContentItem>("install_content_from_modrinth", {
      instanceId,
      versionId,
      kind,
      worldName: worldName ?? null,
    }),
  listInstanceMods: (instanceId: string) => invoke<ModEntry[]>("list_instance_mods", { instanceId }),
  setModEnabled: (instanceId: string, fileName: string, enabled: boolean) =>
    invoke<ModEntry[]>("set_mod_enabled", { instanceId, fileName, enabled }),
  uninstallMod: (instanceId: string, fileName: string) =>
    invoke<ModEntry[]>("uninstall_mod", { instanceId, fileName }),
  importMrpack: (path: string) =>
    invoke<Instance>("import_mrpack", { path }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),
  exportMrpack: (instanceId: string, destPath: string) =>
    invoke<string>("export_mrpack", { instanceId, destPath }),

  reqguardScan: (instanceId: string) => invoke<ReqScanResult>("reqguard_scan", { instanceId }),
  reqguardResolve: (instanceId: string, missingModId: string) =>
    invoke<ReqScanResult>("reqguard_resolve", { instanceId, missingModId }),

  listContent: (instanceId: string, kind: string) =>
    invoke<ContentItem[]>("list_content", { instanceId, kind }),
  installContentZip: (instanceId: string, kind: string, zipPath: string) =>
    invoke<ContentItem[]>("install_content_zip", { instanceId, kind, zipPath }),
  deleteContent: (instanceId: string, kind: string, name: string) =>
    invoke<ContentItem[]>("delete_content", { instanceId, kind, name }),
  openContentItem: (path: string) => invoke<void>("open_content_item", { path }),
  importSave: (instanceId: string, srcPath: string) =>
    invoke<ContentItem[]>("import_save", { instanceId, srcPath }),
  listWorlds: (instanceId: string) => invoke<ContentItem[]>("list_worlds", { instanceId }),
  listScreenshots: (instanceId: string) => invoke<ContentItem[]>("list_screenshots", { instanceId }),
  listDatapacks: (instanceId: string, worldName: string) =>
    invoke<ContentItem[]>("list_datapacks", { instanceId, worldName }),
  installDatapack: (instanceId: string, worldName: string, srcPath: string) =>
    invoke<ContentItem[]>("install_datapack", { instanceId, worldName, srcPath }),
  deleteDatapack: (instanceId: string, worldName: string, name: string) =>
    invoke<ContentItem[]>("delete_datapack", { instanceId, worldName, name }),
  readLogs: (instanceId: string) => invoke<LogLine[]>("read_logs", { instanceId }),
  analyzeCrash: (instanceId: string) => invoke<CrashHint[]>("analyze_crash", { instanceId }),

  listServers: (instanceId: string) => invoke<ServerEntry[]>("list_servers", { instanceId }),
  addServer: (instanceId: string, name: string, ip: string) =>
    invoke<ServerEntry[]>("add_server", { instanceId, name, ip }),
  updateServer: (instanceId: string, index: number, name: string, ip: string) =>
    invoke<ServerEntry[]>("update_server", { instanceId, index, name, ip }),
  removeServer: (instanceId: string, index: number) =>
    invoke<ServerEntry[]>("remove_server", { instanceId, index }),

  getWorldSettings: (instanceId: string, worldName: string) =>
    invoke<WorldSettings>("get_world_settings", { instanceId, worldName }),
  saveWorldSettings: (instanceId: string, settings: WorldSettings) =>
    invoke<WorldSettings>("save_world_settings", { instanceId, settings }),
  listInstanceConfigs: (instanceId: string) =>
    invoke<string[]>("list_instance_configs", { instanceId }),
  readInstanceTextFile: (instanceId: string, relative: string) =>
    invoke<string>("read_instance_text_file", { instanceId, relative }),
  writeInstanceTextFile: (instanceId: string, relative: string, contents: string) =>
    invoke<void>("write_instance_text_file", { instanceId, relative, contents }),

  importForeignInstance: (path: string, folder?: string | null) =>
    invoke<Instance>("import_foreign_instance", { path, folder: folder ?? null }).then((inst) => {
      cacheInvalidate(INSTANCES);
      return inst;
    }),
  importInstanceFolder: (path: string, folder?: string | null) =>
    invoke<Instance[]>("import_instance_folder", { path, folder: folder ?? null }).then((list) => {
      cacheInvalidate(INSTANCES);
      return list;
    }),
  detectJavaInstalls: () => cached(JAVA, 60_000, () => invoke<string[]>("detect_java_installs")),

  listFavorites: () => invoke<FavoriteEntry[]>("list_favorites"),
  toggleFavorite: (payload: {
    id: string;
    kind: string;
    label: string;
    subtitle?: string | null;
    iconUrl?: string | null;
  }) =>
    invoke<FavoriteEntry[]>("toggle_favorite", {
      id: payload.id,
      kind: payload.kind,
      label: payload.label,
      subtitle: payload.subtitle ?? null,
      iconUrl: payload.iconUrl ?? null,
    }),
  removeFavorite: (id: string) => invoke<FavoriteEntry[]>("remove_favorite", { id }),

  listDedicated: () => invoke<DedicatedServer[]>("list_dedicated"),
  createDedicated: (payload: {
    name: string;
    gameVersion: string;
    loader: string;
    memoryMb?: number;
    port?: number;
  }) =>
    invoke<DedicatedServer>("create_dedicated", {
      name: payload.name,
      gameVersion: payload.gameVersion,
      loader: payload.loader,
      memoryMb: payload.memoryMb ?? null,
      port: payload.port ?? null,
    }),
  updateDedicated: (server: DedicatedServer) =>
    invoke<DedicatedServer>("update_dedicated", { server }),
  deleteDedicated: (id: string) => invoke<void>("delete_dedicated", { id }),
  openDedicatedFolder: (id: string) => invoke<void>("open_dedicated_folder", { id }),
  installDedicated: (id: string) => invoke<DedicatedServer>("install_dedicated", { id }),
  acceptDedicatedEula: (id: string) => invoke<DedicatedServer>("accept_dedicated_eula", { id }),
  getDedicatedProperties: (id: string) =>
    invoke<DedicatedProperties>("get_dedicated_properties", { id }),
  setDedicatedProperties: (id: string, props: DedicatedProperties) =>
    invoke<void>("set_dedicated_properties", { id, props }),
  getDedicatedPlayerLists: (id: string) =>
    invoke<DedicatedPlayerLists>("get_dedicated_player_lists", { id }),
  setDedicatedPlayerLists: (id: string, lists: DedicatedPlayerLists) =>
    invoke<void>("set_dedicated_player_lists", { id, lists }),
  startDedicated: (id: string) => invoke<DedicatedStatus>("start_dedicated", { id }),
  stopDedicated: (id: string) => invoke<DedicatedStatus>("stop_dedicated", { id }),
  dedicatedStatus: (id: string) => invoke<DedicatedStatus>("dedicated_status", { id }),
  dedicatedSendCommand: (id: string, command: string) =>
    invoke<void>("dedicated_send_command", { id, command }),
  dedicatedUpnpMap: (id: string) => invoke<DedicatedNetworkInfo>("dedicated_upnp_map", { id }),
  dedicatedUpnpUnmap: (id: string) => invoke<DedicatedNetworkInfo>("dedicated_upnp_unmap", { id }),
  dedicatedNetworkInfo: (id: string) =>
    invoke<DedicatedNetworkInfo>("dedicated_network_info", { id }),
  dedicatedFirewallRule: (id: string) => invoke<string>("dedicated_firewall_rule", { id }),
  importDedicatedMrpack: (id: string, path: string) =>
    invoke<DedicatedServer>("import_dedicated_mrpack", { id, path }),
  dedicatedLiveStats: (id: string) => invoke<HostLiveStats>("dedicated_live_stats", { id }),
  dedicatedCpuCount: () => invoke<number>("dedicated_cpu_count"),
  dedicatedUploadWorld: (id: string, srcPath: string) =>
    invoke<string>("dedicated_upload_world", { id, srcPath }),
  dedicatedUploadMods: (id: string, srcPath: string) =>
    invoke<string>("dedicated_upload_mods", { id, srcPath }),
  dedicatedDownloadWorld: (id: string, destPath: string) =>
    invoke<string>("dedicated_download_world", { id, destPath }),
  dedicatedDownloadMods: (id: string, destPath: string) =>
    invoke<string>("dedicated_download_mods", { id, destPath }),
};

export type { ModrinthHit };
