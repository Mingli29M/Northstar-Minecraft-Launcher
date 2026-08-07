export type LoaderKind = "vanilla" | "fabric" | "quilt" | "forge" | "neoforge";
export type AccountKind = "microsoft" | "offline" | "littleskin";
export type Locale = "en" | "zh" | "de";
export type ContentKind =
  | "saves"
  | "resourcepacks"
  | "shaderpacks"
  | "datapacks"
  | "screenshots"
  | "schematics";
export type ModrinthProjectType = "mod" | "modpack" | "resourcepack" | "shader" | "datapack";

export interface Instance {
  id: string;
  name: string;
  game_version: string;
  loader: LoaderKind;
  loader_version: string | null;
  java_path: string | null;
  memory_mb: number;
  jvm_args: string;
  env_vars: string;
  pre_command: string;
  post_command: string;
  created_at: string;
  last_played: string | null;
  /** Real on-disk folder name under instances root */
  folder: string | null;
  icon_path?: string | null;
}

export interface DetectedGameVersion {
  gameVersion: string;
  loader?: string | null;
  loaderVersion?: string | null;
  source: string;
  applied: boolean;
}

export interface InstanceFolder {
  id: string;
  name: string;
  created_at: string;
  path: string;
}

export interface Account {
  id: string;
  username: string;
  uuid: string;
  access_token: string;
  refresh_token: string;
  active: boolean;
  kind: AccountKind;
}

export interface LauncherSettings {
  instances_path: string | null;
  curseforge_api_key: string | null;
  java_path: string | null;
  last_instance_id: string | null;
  accent: string | null;
  locale: Locale | null;
  download_threads: number | null;
  /** `official` | `bmclapi` */
  download_source?: string | null;
  /** Override path for Host dedicated servers root */
  dedicated_path?: string | null;
  /** Solid page background (CSS) */
  background_color?: string | null;
  /** Optional local path or URL for body background */
  background_image?: string | null;
  /** `system` | `noto` | `source` | `plex` */
  font_family?: string | null;
  /** 0.9 | 1 | 1.1 | 1.25 */
  ui_scale?: number | null;
  /** Window / panel opacity 0.2–1.0 (default ~0.92). Lower values show the desktop through the window. */
  ui_panel_opacity?: number | null;
  /** When true, snapshot worlds before launch (wired in 1.2.0). */
  auto_backup_worlds?: boolean | null;
  /** Max automatic world backups to keep per world. */
  auto_backup_keep?: number | null;
  /** Include network-backed Modrinth checks in background ReqGuard scans. */
  reqguard_deep_validation?: boolean | null;
  /** Experimental local jar-metadata scan (off by default; unstable). */
  reqguard_local_scan?: boolean | null;
  /** Compact Launch page: version picker + Start + override only. */
  launch_only_selected?: boolean | null;
  /** Where the Start button sits on Launch: `top` (default) or `bottom`. */
  launch_start_position?: "top" | "bottom" | null;
}

export type DedicatedLoader =
  | "vanilla"
  | "fabric"
  | "quilt"
  | "forge"
  | "neoforge"
  | "paper"
  | "purpur";

export interface DedicatedServer {
  id: string;
  name: string;
  gameVersion: string;
  loader: DedicatedLoader | string;
  loaderVersion?: string | null;
  memoryMb: number;
  javaPath?: string | null;
  port: number;
  cpuAffinityMask?: number | null;
  createdAt: string;
  lastStarted?: string | null;
  installed: boolean;
  eulaAccepted: boolean;
}

export interface DedicatedStatus {
  id: string;
  running: boolean;
  pid?: number | null;
  upnpMapped: boolean;
}

export interface DedicatedProperties {
  motd: string;
  maxPlayers: number;
  difficulty: string;
  gamemode: string;
  onlineMode: boolean;
  whiteList: boolean;
  spawnMonsters: boolean;
  viewDistance: number;
  serverPort: number;
  levelName: string;
}

export interface DedicatedPlayerLists {
  whitelist: { uuid: string; name: string }[];
  ops: {
    uuid: string;
    name: string;
    level: number;
    bypassesPlayerLimit: boolean;
  }[];
  bannedPlayers: {
    uuid: string;
    name: string;
    created?: string;
    source?: string;
    expires?: string;
    reason?: string;
  }[];
  bannedIps: {
    ip: string;
    created?: string;
    source?: string;
    expires?: string;
    reason?: string;
  }[];
}

export interface DedicatedNetworkInfo {
  lanIp?: string | null;
  port: number;
  upnpStatus: string;
  upnpMessage: string;
  firewallHint: string;
  firewallRuleAdded: boolean;
  adapters?: { name: string; ipv4: string }[];
  joinAddress?: string | null;
  wlanHint?: string;
  publicIp?: string | null;
  wanJoinAddress?: string | null;
  internetHint?: string;
  mapMethod?: string | null;
  mapAttempts?: { method: string; ok: boolean; message: string }[];
  needsManual?: boolean;
  manualHint?: string;
  /** Always false — relay needs an external server (not implemented). */
  relayAvailable?: boolean;
  relayHint?: string;
}

export interface HostLiveStats {
  playersOnline: number;
  playersMax?: number | null;
  playerNames: string[];
  tps?: number | null;
  mspt?: number | null;
  entityCount?: number | null;
  mobCount?: number | null;
  cpuPercent?: number | null;
  ramUsedMb?: number | null;
  ramTotalMb?: number | null;
  ramSystemUsedMb?: number | null;
  netDownBps?: number | null;
  netUpBps?: number | null;
  note: string;
}

export interface HangarProject {
  slug: string;
  name: string;
  description: string;
  icon_url?: string | null;
  author: string;
  category?: string | null;
  downloads?: number | null;
}

export interface HangarVersion {
  name: string;
  created_at?: string | null;
  platform_versions: string[];
  download_url?: string | null;
  external_url?: string | null;
  file_name?: string | null;
}

export interface HostPluginEntry {
  name: string;
  enabled: boolean;
  path: string;
}

export interface ExitBlockers {
  servers: string[];
  terracotta: boolean;
}

export interface TerracottaInfo {
  version: string;
  installed: boolean;
  running: boolean;
  port: number | null;
  binaryPath: string | null;
  installDir: string;
  supported: boolean;
  platformClassifier: string;
  upstreamName: string;
  upstreamUrl: string;
  upstreamLicense: string;
  attribution: string;
  licenseNote: string;
}

export interface TerracottaProfile {
  machineId?: string | null;
  name?: string | null;
  vendor?: string | null;
  kind?: string | null;
}

export interface TerracottaState {
  phase: string;
  index?: number | null;
  port?: number | null;
  room?: string | null;
  url?: string | null;
  difficulty?: string | null;
  profiles: TerracottaProfile[];
  profileIndex?: number | null;
  exceptionType?: number | null;
  rawState?: string | null;
  message?: string | null;
}

export interface ModEntry {
  file_name: string;
  enabled: boolean;
  path: string;
  icon_path?: string | null;
}

export type IssueSeverity = "error" | "warn" | "info";

export interface ReqIssue {
  severity: IssueSeverity;
  mod_id: string;
  message: string;
  missing_mod_id: string | null;
  source_file: string | null;
  source?: string | null;
  project_id?: string | null;
}

export interface ReqScanResult {
  issues: ReqIssue[];
  mod_count: number;
  scanned_at: string;
  local_scan?: boolean;
  deep_scan?: boolean;
  duration_ms?: number;
}

export interface ContentItem {
  name: string;
  path: string;
  kind: string;
  icon_path?: string | null;
}

export interface WorldBackup {
  name: string;
  path: string;
  created_at: string;
}

export interface WorldInfo {
  name: string;
  path: string;
  backup_count: number;
  has_backups: boolean;
  icon_path?: string | null;
}

export interface LitematicaInfo {
  present: boolean;
  schematics_path: string;
}

export interface WorldSettings {
  world_name: string;
  seed: string;
  difficulty: number;
  game_type: number;
  hardcore: boolean;
  allow_commands: boolean;
  do_daylight_cycle: boolean;
  keep_inventory: boolean;
  mob_griefing: boolean;
  do_mob_spawning: boolean;
}

export interface DownloadProgressBytes {
  bytesDone?: number | null;
  bytesTotal?: number | null;
  byteSpeed?: number | null;
}

export interface DownloadProgress extends DownloadProgressBytes {
  phase: string;
  done: number;
  total: number;
  failed: number;
  currentFile?: string | null;
  bytesPerSec?: number | null;
  message: string;
  active: boolean;
}

export interface ServerEntry {
  name: string;
  ip: string;
  icon?: string | null;
  accept_textures?: number | null;
}

export interface LogLine {
  text: string;
  level: string;
}

export interface CrashHint {
  code: string;
  title: string;
  detail: string;
  severity: string;
  params?: string[];
}

export interface GameExitAnalysis {
  instance_id: string;
  exit_code: number | null;
  success: boolean;
  summary: string;
  occurred_at: string;
  hints: CrashHint[];
}

export interface JavaInstall {
  path: string;
  major: number;
}

export interface JavaStatus {
  required_major: number;
  detected: JavaInstall[];
  satisfied: boolean;
  recommended_path: string | null;
}

export interface VersionInfo {
  id: string;
  type_: string;
  release_time: string;
}

export interface ModrinthFile {
  url: string;
  filename: string;
  primary: boolean;
  sha1?: string | null;
}

export interface ModrinthDependency {
  project_id?: string | null;
  version_id?: string | null;
  /** required | optional | incompatible | embedded */
  dependency_type: string;
  project_title?: string | null;
  project_slug?: string | null;
}

export interface ModrinthVersion {
  id: string;
  version_number: string;
  name?: string;
  version_type?: string;
  loaders?: string[];
  game_versions?: string[];
  date_published?: string;
  files: ModrinthFile[];
  dependencies?: ModrinthDependency[];
  project_id?: string;
}

export interface ModrinthGalleryImage {
  url: string;
  featured?: boolean;
  title?: string | null;
}

export interface ModrinthHit {
  project_id: string;
  title: string;
  description: string;
  slug: string;
  icon_url?: string | null;
  categories?: string[];
  versions: ModrinthVersion[];
}

export interface ModrinthProjectDetails {
  project_id: string;
  slug: string;
  title: string;
  description: string;
  body: string;
  icon_url?: string | null;
  categories: string[];
  downloads: number;
  followers: number;
  project_type: string;
  modrinth_url: string;
  source_url?: string | null;
  issues_url?: string | null;
  wiki_url?: string | null;
  discord_url?: string | null;
  mcmod_url: string;
  curseforge_url: string;
  gallery: ModrinthGalleryImage[];
  versions: ModrinthVersion[];
}

export interface InstalledModMarkers {
  version_ids: string[];
  filenames: string[];
}

export interface ParsedConfig {
  format: "toml" | "json" | "properties" | "text";
  fields: { key: string; label?: string; section?: string; value: string; value_type: string }[];
  raw: string;
}

export interface MinecraftNewsItem {
  title: string;
  tag: string;
  date: string;
  text: string;
  image_url?: string | null;
  read_more_url?: string | null;
}

export interface MinecraftPatchNote {
  version: string;
  title: string;
  body: string;
  type_: string;
}

export interface ModUpdateResult {
  file_name: string;
  updated: boolean;
  message: string;
}

export type FavoriteKind = "instance" | "modrinth" | "server" | "mcversion" | "dedicated";

export interface FavoriteEntry {
  id: string;
  kind: FavoriteKind | string;
  label: string;
  subtitle?: string | null;
  iconUrl?: string | null;
  addedAt: string;
}

export function favoriteId(kind: FavoriteKind, key: string): string {
  return `${kind}:${key}`;
}

export function normalizeServerKey(ip: string): string {
  return ip.trim().toLowerCase();
}
