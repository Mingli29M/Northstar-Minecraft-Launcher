use crate::instances::get_instance;
use crate::models::{IssueSeverity, LoaderKind, ReqIssue, ReqScanResult};
use crate::mods_platform::{
    fetch_project_slugs, file_sha1_hex, install_project_with_deps, lookup_versions_by_hashes,
    search_mods,
};
use crate::paths::minecraft_dir;
use chrono::Utc;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use std::time::Instant;
use zip::ZipArchive;

fn mk_issue(
    severity: IssueSeverity,
    mod_id: impl Into<String>,
    message: impl Into<String>,
    missing_mod_id: Option<String>,
    source_file: Option<String>,
    source: &str,
    project_id: Option<String>,
) -> ReqIssue {
    ReqIssue {
        severity,
        mod_id: mod_id.into(),
        message: message.into(),
        missing_mod_id,
        source_file,
        source: Some(source.into()),
        project_id,
    }
}

#[derive(Debug, Clone)]
struct ModMeta {
    id: String,
    version: String,
    file: String,
    /// Extra mod IDs this jar provides (Fabric API modules, `provides`, nested JiJ mods).
    provides: Vec<String>,
    depends: HashMap<String, String>,
    recommends: HashMap<String, String>,
    breaks: HashMap<String, String>,
    conflicts: HashMap<String, String>,
}

impl ModMeta {
    fn all_ids(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.id.as_str()).chain(self.provides.iter().map(|s| s.as_str()))
    }
}

/// Fast, offline scan used by the launch gate.
pub fn scan_instance(instance_id: &str) -> Result<ReqScanResult, String> {
    scan_instance_with_mode(instance_id, false)
}

/// Full background scan including Modrinth version/dependency validation.
pub fn scan_instance_deep(instance_id: &str) -> Result<ReqScanResult, String> {
    scan_instance_with_mode(instance_id, true)
}

fn scan_instance_with_mode(instance_id: &str, deep_scan: bool) -> Result<ReqScanResult, String> {
    let started = Instant::now();
    let inst = get_instance(instance_id)?;
    let mods_dir = minecraft_dir(instance_id)?.join("mods");
    fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;

    let mut metas = Vec::new();
    let mut issues = Vec::new();
    for entry in fs::read_dir(&mods_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".jar") || name.ends_with(".disabled") {
            continue;
        }
        match parse_jar(&entry.path(), &name) {
            Ok(mut list) => {
                // Ensure fabric-api filename registers the umbrella id even if metadata id differs.
                if filename_looks_like_fabric_api(&name)
                    && !list.iter().any(|m| {
                        m.id == "fabric-api" || m.provides.iter().any(|p| p == "fabric-api")
                    })
                {
                    if let Some(root) = list.iter_mut().find(|m| !m.file.contains("!/")) {
                        if root.id != "fabric-api" {
                            root.provides.push("fabric-api".into());
                        }
                    } else if let Some(fb) = filename_fallback_meta(&name) {
                        list.push(fb);
                    }
                }
                metas.extend(list);
            }
            Err(e) => {
                // Always register jar in `present` so UI-listed mods aren't reported missing.
                metas.push(filename_fallback_meta(&name).unwrap_or_else(|| ModMeta {
                    id: jar_stem_id(&name),
                    version: "*".into(),
                    file: name.clone(),
                    provides: Vec::new(),
                    depends: HashMap::new(),
                    recommends: HashMap::new(),
                    breaks: HashMap::new(),
                    conflicts: HashMap::new(),
                }));
                issues.push(mk_issue(
                    IssueSeverity::Warn,
                    jar_stem_id(&name),
                    format!("Could not read metadata: {name} — {e}"),
                    None,
                    Some(name),
                    "local",
                    None,
                ));
            }
        }
    }

    // id -> version (first wins; nested modules inherit parent version)
    let mut present: HashMap<String, String> = HashMap::new();
    // id -> providing root mod id (for nicer install hints)
    let mut provided_by: HashMap<String, String> = HashMap::new();
    for meta in &metas {
        for id in meta.all_ids() {
            let id = canonical_mod_id(id);
            present
                .entry(id.clone())
                .or_insert_with(|| meta.version.clone());
            if id != canonical_mod_id(&meta.id) {
                provided_by
                    .entry(id)
                    .or_insert_with(|| meta.id.clone());
            }
        }
    }

    let mut present_ids: HashSet<String> = present.keys().cloned().collect();
    present_ids.insert("minecraft".into());
    present_ids.insert("java".into());
    match inst.loader {
        LoaderKind::Fabric => {
            present_ids.insert("fabricloader".into());
            present_ids.insert("fabric-loader".into());
            present_ids.insert("fabric".into());
        }
        LoaderKind::Quilt => {
            present_ids.insert("quilt_loader".into());
            present_ids.insert("quilt-loader".into());
            present_ids.insert("quilted_fabric_api".into());
        }
        LoaderKind::Forge => {
            present_ids.insert("forge".into());
        }
        LoaderKind::NeoForge => {
            present_ids.insert("neoforge".into());
        }
        LoaderKind::Vanilla => {}
    }

    let has_fabric_api = present.contains_key("fabric-api")
        || metas.iter().any(|m| filename_looks_like_fabric_api(&m.file));
    let has_quilted_api =
        present.contains_key("quilted_fabric_api") || present.contains_key("quilted-fabric-api");
    if has_fabric_api {
        present_ids.insert("fabric".into());
        present_ids.insert("fabric-api".into());
    }
    if has_quilted_api {
        present_ids.insert("quilted_fabric_api".into());
        present_ids.insert("quilted-fabric-api".into());
    }

    for meta in &metas {
        // Only enforce depends on the root entry of each physical jar once.
        // Nested JiJ metas are recorded for provides only (depends already declared on root).
        if meta.file.contains("!/") {
            continue;
        }

        for (dep, range) in &meta.depends {
            let dep = canonical_mod_id(dep);
            if meta_provides_id(meta, &dep) {
                continue;
            }
            if dep == "minecraft" {
                if !version_satisfies(&inst.game_version, range) {
                    issues.push(mk_issue(
                        IssueSeverity::Error,
                        meta.id.clone(),
                        format!(
                            "{} requires Minecraft {} (instance is {})",
                            meta.id, range, inst.game_version
                        ),
                        None,
                        Some(meta.file.clone()),
                        "local",
                        None,
                    ));
                }
                continue;
            }
            if is_loader_dep(&dep) {
                continue;
            }
            if dep_satisfied(&dep, &present_ids, has_fabric_api, has_quilted_api) {
                if let Some(ver) = present.get(&dep) {
                    if !version_satisfies(ver, range) {
                        let via = provided_by
                            .get(&dep)
                            .map(|p| format!(" via `{p}`"))
                            .unwrap_or_default();
                        let umbrella_only = (has_fabric_api
                            && is_fabric_api_module(&dep)
                            && !present.contains_key(&dep))
                            || (has_quilted_api
                                && is_quilted_api_module(&dep)
                                && !present.contains_key(&dep));
                        if !umbrella_only {
                            // Unparseable / exotic ranges: warn when the ID is present.
                            let sev = if range_looks_exotic(range) {
                                IssueSeverity::Warn
                            } else {
                                IssueSeverity::Error
                            };
                            issues.push(mk_issue(
                                sev,
                                meta.id.clone(),
                                format!("{} needs `{dep}` {range}, found {ver}{via}", meta.id),
                                suggest_install_id(
                                    &dep,
                                    &provided_by,
                                    has_fabric_api,
                                    has_quilted_api,
                                ),
                                Some(meta.file.clone()),
                                "local",
                                None,
                            ));
                        }
                    }
                }
                continue;
            }

            let suggest =
                suggest_install_id(&dep, &provided_by, has_fabric_api, has_quilted_api);
            issues.push(mk_issue(
                IssueSeverity::Error,
                meta.id.clone(),
                format!(
                    "{} depends on missing `{dep}` ({range}){}",
                    meta.id,
                    match &suggest {
                        Some(s) if s != &dep => format!(" — install `{s}`"),
                        _ => String::new(),
                    }
                ),
                suggest,
                Some(meta.file.clone()),
                "local",
                None,
            ));
        }

        for (dep, range) in &meta.recommends {
            let dep = canonical_mod_id(dep);
            if meta_provides_id(meta, &dep) {
                continue;
            }
            if is_loader_dep(&dep) || dep == "minecraft" {
                continue;
            }
            if !dep_satisfied(&dep, &present_ids, has_fabric_api, has_quilted_api) {
                issues.push(mk_issue(
                    IssueSeverity::Warn,
                    meta.id.clone(),
                    format!("{} recommends `{dep}` ({range})", meta.id),
                    suggest_install_id(&dep, &provided_by, has_fabric_api, has_quilted_api),
                    Some(meta.file.clone()),
                    "local",
                    None,
                ));
            }
        }

        for (other, range) in &meta.breaks {
            let other = canonical_mod_id(other);
            if present_ids.contains(&other) {
                if let Some(ver) = present.get(&other) {
                    if version_satisfies(ver, range) || range == "*" {
                        issues.push(mk_issue(
                            IssueSeverity::Error,
                            meta.id.clone(),
                            format!("{} breaks `{other}` ({range})", meta.id),
                            None,
                            Some(meta.file.clone()),
                            "local",
                            None,
                        ));
                    }
                }
            }
        }

        for (other, range) in &meta.conflicts {
            let other = canonical_mod_id(other);
            if present_ids.contains(&other) {
                issues.push(mk_issue(
                    IssueSeverity::Warn,
                    meta.id.clone(),
                    format!("{} conflicts with `{other}` ({range})", meta.id),
                    None,
                    Some(meta.file.clone()),
                    "local",
                    None,
                ));
            }
        }
    }

    if deep_scan {
        // Modrinth dependency SoT is network-backed and intentionally kept out
        // of the synchronous launch gate.
        merge_modrinth_sot(
            &mods_dir,
            &metas,
            &present_ids,
            &inst.game_version,
            inst.loader.as_str(),
            &mut issues,
        );
    }

    dedupe_issues(&mut issues);

    Ok(ReqScanResult {
        issues,
        mod_count: metas.iter().filter(|m| !m.file.contains("!/")).count(),
        scanned_at: Utc::now().to_rfc3339(),
        deep_scan,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

fn canonical_mod_id(id: &str) -> String {
    let normalized = id.trim().to_ascii_lowercase().replace('_', "-");
    match normalized.as_str() {
        // Cloth Config has used both ids across loaders/releases.
        "cloth-config2" => "cloth-config".into(),
        "fabricloader" => "fabric-loader".into(),
        "quilt-loader" | "quiltloader" => "quilt-loader".into(),
        "quilted-fabric-api" => "quilted-fabric-api".into(),
        _ => normalized,
    }
}

fn meta_provides_id(meta: &ModMeta, dep: &str) -> bool {
    canonical_mod_id(&meta.id) == dep
        || meta
            .provides
            .iter()
            .any(|provided| canonical_mod_id(provided) == dep)
}

fn dedupe_issues(issues: &mut Vec<ReqIssue>) {
    let mut seen = HashSet::new();
    issues.retain(|issue| {
        let key = (
            issue.severity.clone() as u8,
            canonical_mod_id(&issue.mod_id),
            issue
                .missing_mod_id
                .as_deref()
                .map(canonical_mod_id)
                .unwrap_or_default(),
            issue.source.clone().unwrap_or_default(),
        );
        seen.insert(key)
    });
}

fn range_looks_exotic(range: &str) -> bool {
    let t = range.trim();
    if t.is_empty() || t == "*" {
        return false;
    }
    // Maven intervals and simple predicates are fine; everything else is exotic.
    if t.starts_with('[') || t.starts_with('(') {
        return false;
    }
    if t.starts_with(">=")
        || t.starts_with("<=")
        || t.starts_with('>')
        || t.starts_with('<')
        || t.starts_with('=')
        || t.starts_with('~')
        || t.starts_with('^')
    {
        return false;
    }
    if t.contains("||") || t.contains(' ') {
        return false;
    }
    // Bare version string is OK; otherwise treat as exotic/unparseable.
    t.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+'))
}

/// Cross-check installed jars against Modrinth version dependencies.
fn merge_modrinth_sot(
    mods_dir: &Path,
    metas: &[ModMeta],
    present_ids: &HashSet<String>,
    game_version: &str,
    loader: &str,
    issues: &mut Vec<ReqIssue>,
) {
    use rayon::prelude::*;

    let _ = (game_version, loader);
    let mut known_modrinth_projects: HashSet<String> = HashSet::new();

    // Pass 1: hash every root jar, then use Modrinth's batch endpoint. The old
    // 40-jar cap made large instances incorrectly report later jars as absent.
    let roots: Vec<_> = metas
        .iter()
        .filter(|m| !m.file.contains("!/"))
        .collect();
    let hashed_roots: Vec<_> = roots
        .par_iter()
        .filter_map(|meta| {
            let path = mods_dir.join(&meta.file);
            let hash = file_sha1_hex(&path).ok()?;
            Some((meta.id.clone(), meta.file.clone(), hash))
        })
        .collect();
    let mut versions_by_hash = HashMap::new();
    let hashes: Vec<String> = hashed_roots
        .iter()
        .map(|(_, _, hash)| hash.clone())
        .collect();
    for chunk in hashes.chunks(100) {
        if let Ok(found) = lookup_versions_by_hashes(chunk) {
            versions_by_hash.extend(found);
        }
    }
    let versions: Vec<_> = hashed_roots
        .into_iter()
        .filter_map(|(mod_id, file, hash)| {
            versions_by_hash
                .remove(&hash)
                .map(|version| (mod_id, file, version))
        })
        .collect();
    for (_, _, version) in &versions {
        if !version.project_id.is_empty() {
            known_modrinth_projects.insert(version.project_id.clone());
        }
    }

    // A manually installed or CurseForge jar may not resolve by Modrinth hash.
    // Map dependency project ids to their public slugs so local metadata can
    // still prove that the dependency is installed.
    let mut dependency_projects: Vec<String> = versions
        .iter()
        .flat_map(|(_, _, version)| {
            version
                .dependencies
                .iter()
                .filter_map(|dep| dep.project_id.clone())
        })
        .filter(|id| !id.is_empty() && !known_modrinth_projects.contains(id))
        .collect();
    dependency_projects.sort();
    dependency_projects.dedup();
    let mut project_slugs = HashMap::new();
    for chunk in dependency_projects.chunks(100) {
        if let Ok(found) = fetch_project_slugs(chunk) {
            project_slugs.extend(found);
        }
    }

    // Pass 2: emit missing required deps / incompatibles.
    let mut seen_required: HashSet<String> = HashSet::new();
    for (mod_id, file, version) in &versions {
        for dep in &version.dependencies {
            let dep_type = dep.dependency_type.to_ascii_lowercase();
            let Some(project_id) = dep.project_id.clone().filter(|s| !s.is_empty()) else {
                continue;
            };
            // Broken or redundant upstream metadata must never make a project
            // report itself as missing.
            if project_id == version.project_id {
                continue;
            }
            let dependency_present = modrinth_project_is_present(
                &project_id,
                &known_modrinth_projects,
                &project_slugs,
                present_ids,
            );
            if dep_type == "incompatible" && dependency_present {
                issues.push(mk_issue(
                    IssueSeverity::Error,
                    mod_id.clone(),
                    format!("{mod_id} is incompatible with Modrinth project `{project_id}` (SoT)"),
                    None,
                    Some(file.clone()),
                    "modrinth",
                    Some(project_id),
                ));
                continue;
            }
            if dep_type != "required" {
                continue;
            }
            if dependency_present {
                continue;
            }
            if !seen_required.insert(project_id.clone()) {
                continue;
            }
            issues.push(mk_issue(
                IssueSeverity::Error,
                mod_id.clone(),
                format!("{mod_id} requires Modrinth dependency `{project_id}` (not installed)"),
                Some(project_id.clone()),
                Some(file.clone()),
                "modrinth",
                Some(project_id),
            ));
        }
    }

    for issue in issues.iter_mut() {
        if issue.source.as_deref() != Some("local") {
            continue;
        }
        if !matches!(issue.severity, IssueSeverity::Error) {
            continue;
        }
        let Some(missing) = issue.missing_mod_id.as_deref() else {
            continue;
        };
        if present_ids.contains(missing) {
            issue.severity = IssueSeverity::Warn;
            issue.message = format!("{} (present via provides/umbrella — demoted)", issue.message);
        }
    }
}

fn modrinth_project_is_present(
    project_id: &str,
    known_projects: &HashSet<String>,
    project_slugs: &HashMap<String, String>,
    present_ids: &HashSet<String>,
) -> bool {
    if known_projects.contains(project_id) {
        return true;
    }
    let local_id = project_slugs
        .get(project_id)
        .map(|slug| canonical_mod_id(slug))
        .or_else(|| known_mod_id_for_project(project_id).map(str::to_string));
    local_id
        .as_deref()
        .map(|id| present_ids.contains(id))
        .unwrap_or(false)
}

fn known_mod_id_for_project(project_id: &str) -> Option<&'static str> {
    match project_id {
        // Fabric API. This fallback also works if project metadata lookup is
        // temporarily unavailable.
        "P7dR8mSH" => Some("fabric-api"),
        _ => None,
    }
}

fn jar_stem_id(file_name: &str) -> String {
    Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.to_string())
        .to_ascii_lowercase()
}

fn filename_looks_like_fabric_api(file_name: &str) -> bool {
    let stem = jar_stem_id(file_name);
    let normalized = stem.replace('_', "-");
    normalized == "fabric-api"
        || normalized.starts_with("fabric-api-")
        || normalized.starts_with("fabric-api+")
}

fn filename_fallback_meta(file_name: &str) -> Option<ModMeta> {
    if !filename_looks_like_fabric_api(file_name) {
        return None;
    }
    Some(ModMeta {
        id: "fabric-api".into(),
        version: "*".into(),
        file: file_name.to_string(),
        provides: Vec::new(),
        depends: HashMap::new(),
        recommends: HashMap::new(),
        breaks: HashMap::new(),
        conflicts: HashMap::new(),
    })
}

fn is_fabric_api_module(dep: &str) -> bool {
    // Only treat known Fabric API module patterns as covered by the fabric-api umbrella.
    // Do NOT treat arbitrary fabric-* mods (e.g. fabric-language-kotlin) as FAPI modules.
    if matches!(
        dep,
        "fabric-api" | "fabric-loader" | "fabric" | "fabric-language-kotlin"
    ) {
        return false;
    }
    if dep == "fabric-api-base" || dep.starts_with("fabric-api-") {
        return true;
    }
    dep.starts_with("fabric-")
        && (dep.contains("-v1")
            || dep.contains("-v2")
            || dep.contains("-v3")
            || dep.contains("-v4")
            || dep.contains("-v5"))
}

fn is_quilted_api_module(dep: &str) -> bool {
    (dep.starts_with("quilt_") || dep.starts_with("quilted_"))
        && dep != "quilted_fabric_api"
        && dep != "quilt_loader"
}

fn dep_satisfied(
    dep: &str,
    present_ids: &HashSet<String>,
    has_fabric_api: bool,
    has_quilted_api: bool,
) -> bool {
    if present_ids.contains(dep) {
        return true;
    }
    if has_fabric_api && is_fabric_api_module(dep) {
        return true;
    }
    if has_quilted_api && is_quilted_api_module(dep) {
        return true;
    }
    false
}

fn suggest_install_id(
    dep: &str,
    provided_by: &HashMap<String, String>,
    has_fabric_api: bool,
    has_quilted_api: bool,
) -> Option<String> {
    if let Some(parent) = provided_by.get(dep) {
        return Some(parent.clone());
    }
    // Only suggest installing fabric-api when the umbrella itself is missing.
    if is_fabric_api_module(dep) {
        return if has_fabric_api {
            None
        } else {
            Some("fabric-api".into())
        };
    }
    if is_quilted_api_module(dep) {
        return if has_quilted_api {
            None
        } else {
            Some("quilted-fabric-api".into())
        };
    }
    Some(dep.to_string())
}

fn is_loader_dep(id: &str) -> bool {
    matches!(
        id,
        "fabricloader"
            | "fabric-loader"
            | "fabric"
            | "quilt_loader"
            | "quilt-loader"
            | "forge"
            | "neoforge"
            | "java"
            | "minecraft"
    )
}

/// Parse a mod jar into one or more metas (root + nested JiJ modules as provides sources).
fn parse_jar(path: &Path, file_name: &str) -> Result<Vec<ModMeta>, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    parse_jar_bytes(&bytes, file_name, false)
}

fn parse_jar_bytes(bytes: &[u8], file_name: &str, nested: bool) -> Result<Vec<ModMeta>, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| e.to_string())?;

    // Prefer Fabric / Quilt metadata
    let fabric_raw = {
        let mut raw = None;
        {
            if let Ok(mut f) = archive.by_name("fabric.mod.json") {
                let mut s = String::new();
                if f.read_to_string(&mut s).is_ok() {
                    raw = Some(s);
                }
            }
        }
        if raw.is_none() {
            if let Ok(mut f) = archive.by_name("quilt.mod.json") {
                let mut s = String::new();
                if f.read_to_string(&mut s).is_ok() {
                    raw = Some(s);
                }
            }
        }
        raw
    };

    if let Some(raw) = fabric_raw {
        let mut meta = parse_fabric_json(&raw, file_name)?;
        let mut out: Vec<ModMeta> = Vec::new();

        let mut nested_paths = nested_jar_paths_from_fabric(&raw);
        if !nested {
            for i in 0..archive.len() {
                if let Ok(e) = archive.by_index(i) {
                    let n = e.name().to_string();
                    if n.starts_with("META-INF/jars/") && n.ends_with(".jar") && !nested_paths.contains(&n)
                    {
                        nested_paths.push(n);
                    }
                }
            }
        }

        for nested_path in nested_paths {
            let buf = {
                let Ok(mut nf) = archive.by_name(&nested_path) else {
                    continue;
                };
                let mut buf = Vec::new();
                if nf.read_to_end(&mut buf).is_err() {
                    continue;
                }
                buf
            };
            let nested_name = format!("{file_name}!/{nested_path}");
            if let Ok(nested_metas) = parse_jar_bytes(&buf, &nested_name, true) {
                for nm in nested_metas {
                    if !meta.provides.iter().any(|p| p == &nm.id) && nm.id != meta.id {
                        meta.provides.push(nm.id.clone());
                    }
                    for p in &nm.provides {
                        if !meta.provides.iter().any(|x| x == p) && p != &meta.id {
                            meta.provides.push(p.clone());
                        }
                    }
                    if !out.iter().any(|m| m.id == nm.id && m.file == nm.file) {
                        out.push(nm);
                    }
                }
            }
        }

        out.insert(0, meta);
        return Ok(out);
    }

    for name in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Ok(mut f) = archive.by_name(name) {
            let mut raw = String::new();
            f.read_to_string(&mut raw).map_err(|e| e.to_string())?;
            return Ok(vec![parse_mods_toml(&raw, file_name)?]);
        }
    }

    Err("No known mod metadata".into())
}

fn nested_jar_paths_from_fabric(raw: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(arr) = v.get("jars").and_then(|j| j.as_array()) {
        for j in arr {
            if let Some(file) = j.get("file").and_then(|f| f.as_str()) {
                out.push(file.to_string());
            }
        }
    }
    out
}

fn parse_fabric_json(raw: &str, file_name: &str) -> Result<ModMeta, String> {
    let v: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    let id = v["id"]
        .as_str()
        .unwrap_or("unknown")
        .to_ascii_lowercase();
    let version = v["version"].as_str().unwrap_or("*").to_string();
    let mut provides = Vec::new();
    if let Some(arr) = v.get("provides").and_then(|p| p.as_array()) {
        for p in arr {
            if let Some(s) = p.as_str() {
                let s = s.to_ascii_lowercase();
                if s != id {
                    provides.push(s);
                }
            }
        }
    }
    Ok(ModMeta {
        id,
        version,
        file: file_name.to_string(),
        provides,
        depends: map_deps(v.get("depends")),
        recommends: map_deps(v.get("recommends")),
        breaks: map_deps(v.get("breaks")),
        conflicts: map_deps(v.get("conflicts")),
    })
}

fn map_deps(v: Option<&Value>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(obj) = v.and_then(|x| x.as_object()) {
        for (k, val) in obj {
            let range = match val {
                Value::String(s) => s.clone(),
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(" || "),
                _ => "*".into(),
            };
            map.insert(k.to_ascii_lowercase(), range);
        }
    }
    map
}

fn parse_mods_toml(raw: &str, file_name: &str) -> Result<ModMeta, String> {
    let value: toml::Value = toml::from_str(raw).map_err(|e| e.to_string())?;
    let mut id = "unknown".to_string();
    let mut version = "*".to_string();
    if let Some(mods) = value.get("mods").and_then(|m| m.as_array()) {
        if let Some(first) = mods.first() {
            id = first
                .get("modId")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_ascii_lowercase();
            version = first
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string();
        }
    }

    let mut depends = HashMap::new();
    let mut breaks = HashMap::new();
    let mut conflicts = HashMap::new();
    let mut recommends = HashMap::new();
    let mut provides = Vec::new();

    // Collect additional modIds from [[mods]] as provides (multi-mod jars)
    if let Some(mods) = value.get("mods").and_then(|m| m.as_array()) {
        for (i, m) in mods.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if let Some(mid) = m.get("modId").and_then(|v| v.as_str()) {
                let mid = mid.to_ascii_lowercase();
                if mid != id {
                    provides.push(mid);
                }
            }
        }
    }

    if let Some(deps_table) = value.get("dependencies").and_then(|d| d.as_table()) {
        for (_owner_mod, entries) in deps_table {
            // Keep external requirements declared by every provided mod, but
            // dependencies satisfied by this same physical jar are filtered
            // below. That prevents Cloth Config -> Cloth Config without hiding
            // a real dependency of another mod bundled in the jar.
            let list = match entries {
                toml::Value::Array(a) => a.clone(),
                other => vec![other.clone()],
            };
            for entry in list {
                let dep_id = entry
                    .get("modId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if dep_id.is_empty() {
                    continue;
                }
                if canonical_mod_id(&dep_id) == canonical_mod_id(&id)
                    || provides
                        .iter()
                        .any(|provided| canonical_mod_id(provided) == canonical_mod_id(&dep_id))
                {
                    continue;
                }
                let range = entry
                    .get("versionRange")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string();
                let mandatory = entry.get("mandatory").and_then(|v| v.as_bool());
                let dtype = entry.get("type").and_then(|v| v.as_str());
                let kind = if let Some(m) = mandatory {
                    if m {
                        "required"
                    } else {
                        "optional"
                    }
                } else {
                    dtype.unwrap_or("required")
                };
                match kind {
                    "required" => {
                        depends.insert(dep_id, range);
                    }
                    "optional" => {
                        recommends.insert(dep_id, range);
                    }
                    "incompatible" => {
                        breaks.insert(dep_id, range);
                    }
                    "discouraged" => {
                        conflicts.insert(dep_id, range);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(ModMeta {
        id,
        version,
        file: file_name.to_string(),
        provides,
        depends,
        recommends,
        breaks,
        conflicts,
    })
}

/// Loose matcher for Fabric / Maven / npm-style version predicates.
fn version_satisfies(have: &str, range: &str) -> bool {
    let range = range.trim();
    if range.is_empty() || range == "*" {
        return true;
    }
    if range.contains("||") {
        return range.split("||").any(|r| version_satisfies(have, r.trim()));
    }
    if range.starts_with('[') || range.starts_with('(') {
        return maven_interval_satisfies(have, range);
    }
    let parts = split_and_predicates(range);
    if parts.len() > 1 {
        return parts.iter().all(|p| version_satisfies(have, p));
    }
    compare_single_predicate(have, parts.first().map(|s| s.as_str()).unwrap_or(range))
}

fn split_and_predicates(range: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let bytes = range.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let at_op = (bytes[i] == b'>' || bytes[i] == b'<' || bytes[i] == b'=')
            && (i == 0 || bytes[i - 1].is_ascii_whitespace());
        if at_op && i > start {
            let chunk = range[start..i].trim();
            if !chunk.is_empty() {
                out.push(chunk.to_string());
            }
            start = i;
        }
        i += 1;
    }
    let chunk = range[start..].trim();
    if !chunk.is_empty() {
        out.push(chunk.to_string());
    }
    if out.is_empty() {
        out.push(range.to_string());
    }
    out
}

fn compare_single_predicate(have: &str, pred: &str) -> bool {
    let pred = pred.trim();
    if pred.is_empty() || pred == "*" {
        return true;
    }
    if let Some(prefix) = pred
        .strip_suffix(".x")
        .or_else(|| pred.strip_suffix(".*"))
        .or_else(|| pred.strip_suffix(".X"))
    {
        let prefix = prefix.trim_end_matches('.');
        return have == prefix
            || have.starts_with(&format!("{prefix}."))
            || have.starts_with(prefix);
    }

    let (op, raw_ver) = if let Some(rest) = pred.strip_prefix(">=") {
        (">=", rest)
    } else if let Some(rest) = pred.strip_prefix("<=") {
        ("<=", rest)
    } else if let Some(rest) = pred.strip_prefix("=>") {
        (">=", rest)
    } else if let Some(rest) = pred.strip_prefix("=<") {
        ("<=", rest)
    } else if let Some(rest) = pred.strip_prefix('>') {
        (">", rest)
    } else if let Some(rest) = pred.strip_prefix('<') {
        ("<", rest)
    } else if let Some(rest) = pred.strip_prefix('=') {
        ("=", rest)
    } else if let Some(rest) = pred.strip_prefix('~') {
        let ver = rest.trim();
        return version_satisfies(have, &format!(">={ver}")) && soft_tilde_upper(have, ver);
    } else if let Some(rest) = pred.strip_prefix('^') {
        let ver = rest.trim();
        return version_satisfies(have, &format!(">={ver}"));
    } else {
        ("=", pred)
    };

    let ver = raw_ver.trim().trim_end_matches('+').trim_end_matches('-');
    let have_n = normalize_semver(have);
    let ver_n = normalize_semver(ver);
    if let (Ok(h), Ok(r)) = (semver::Version::parse(&have_n), semver::Version::parse(&ver_n)) {
        return match op {
            ">=" => h >= r,
            ">" => h > r,
            "<=" => h <= r,
            "<" => h < r,
            _ => h == r,
        };
    }
    if let (Some(h), Some(r)) = (parse_mc_tuple(have), parse_mc_tuple(ver)) {
        return match op {
            ">=" => h >= r,
            ">" => h > r,
            "<=" => h <= r,
            "<" => h < r,
            _ => h == r,
        };
    }
    match op {
        "=" => have == ver || have.starts_with(ver),
        _ => false,
    }
}

fn soft_tilde_upper(have: &str, base: &str) -> bool {
    let Some(mut t) = parse_mc_tuple(base) else {
        return true;
    };
    if t.len() >= 2 {
        t[1] += 1;
        t.truncate(2);
        t.push(0);
    } else if let Some(first) = t.first_mut() {
        *first += 1;
        t.push(0);
        t.push(0);
    }
    let upper = t
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".");
    version_satisfies(have, &format!("<{upper}"))
}

fn parse_mc_tuple(v: &str) -> Option<Vec<u64>> {
    let core = v.split(|c| c == '-' || c == '+' || c == ' ').next().unwrap_or(v);
    let mut nums = Vec::new();
    for part in core.split('.') {
        if part.is_empty() {
            continue;
        }
        let n = part.parse::<u64>().ok()?;
        nums.push(n);
    }
    if nums.is_empty() {
        None
    } else {
        while nums.len() < 3 {
            nums.push(0);
        }
        Some(nums)
    }
}

fn maven_interval_satisfies(have: &str, range: &str) -> bool {
    let range = range.trim();
    if range == "*" || range == "[,)" || range == "(,)" {
        return true;
    }
    let soft = range.trim_start_matches(['[', '(']).trim_end_matches([']', ')']);
    let (lower, upper) = match soft.split_once(',') {
        Some(p) => p,
        None => return soft.trim() == have || soft.contains('*'),
    };
    let lower = lower.trim();
    let upper = upper.trim();
    let lower_inc = range.starts_with('[');
    let upper_inc = range.ends_with(']');

    if !lower.is_empty() {
        let ok = if lower_inc {
            version_satisfies(have, &format!(">={lower}"))
        } else {
            version_satisfies(have, &format!(">{lower}"))
        };
        if !ok {
            return false;
        }
    }
    if !upper.is_empty() {
        let ok = if upper_inc {
            version_satisfies(have, &format!("<={upper}"))
        } else {
            version_satisfies(have, &format!("<{upper}"))
        };
        if !ok {
            return false;
        }
    }
    true
}

fn normalize_semver(v: &str) -> String {
    let core = v
        .split(|c| c == '-' || c == '+')
        .next()
        .unwrap_or(v)
        .trim();
    let parts: Vec<&str> = core.split('.').filter(|p| !p.is_empty()).collect();
    match parts.len() {
        0 => "0.0.0".into(),
        1 => format!("{}.0.0", parts[0]),
        2 => format!("{}.{}.0", parts[0], parts[1]),
        _ => format!("{}.{}.{}", parts[0], parts[1], parts[2]),
    }
}

pub fn resolve_missing(instance_id: String, missing_mod_id: String) -> Result<ReqScanResult, String> {
    let inst = get_instance(&instance_id)?;
    let loader = inst.loader.as_str();

    // Prefer direct Modrinth project id (from SoT issues) over fuzzy title search.
    let looks_like_project_id = missing_mod_id.len() >= 8
        && missing_mod_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if looks_like_project_id {
        if install_project_with_deps(
            &instance_id,
            &missing_mod_id,
            &inst.game_version,
            loader,
            0,
        )
        .is_ok()
        {
            return scan_instance(&instance_id);
        }
    }

    let hits = search_mods(
        missing_mod_id.clone(),
        inst.game_version.clone(),
        loader.to_string(),
        None,
    )?;
    let hit = hits
        .into_iter()
        .find(|h| {
            h.slug == missing_mod_id
                || h.project_id == missing_mod_id
                || h.title.eq_ignore_ascii_case(&missing_mod_id)
        })
        .or_else(|| {
            search_mods(
                missing_mod_id.clone(),
                inst.game_version.clone(),
                loader.to_string(),
                None,
            )
            .ok()
            .and_then(|mut v| v.drain(..).next())
        })
        .ok_or_else(|| format!("Could not find `{missing_mod_id}` on Modrinth for this instance"))?;

    install_project_with_deps(
        &instance_id,
        &hit.project_id,
        &inst.game_version,
        loader,
        0,
    )?;
    scan_instance(&instance_id)
}

/// Install all missing required deps reported by the latest scan.
pub fn resolve_all_missing(instance_id: String) -> Result<ReqScanResult, String> {
    let scan = scan_instance(&instance_id)?;
    let mut targets: Vec<String> = scan
        .issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::Error))
        .filter_map(|i| {
            i.project_id
                .clone()
                .or_else(|| i.missing_mod_id.clone())
        })
        .collect();
    targets.sort();
    targets.dedup();
    for target in targets {
        let _ = resolve_missing(instance_id.clone(), target);
    }
    scan_instance(&instance_id)
}

#[cfg(test)]
mod version_tests {
    use super::{
        canonical_mod_id, is_fabric_api_module, modrinth_project_is_present, parse_mods_toml,
        version_satisfies,
    };
    use std::collections::{HashMap, HashSet};

    #[test]
    fn le_includes_equality() {
        assert!(version_satisfies("1.21.11", "<=1.21.11"));
        assert!(version_satisfies("1.21.10", "<=1.21.11"));
        assert!(!version_satisfies("1.21.12", "<=1.21.11"));
        assert!(!version_satisfies("1.21.11", "<1.21.11"));
    }

    #[test]
    fn and_range() {
        assert!(version_satisfies("1.21.11", ">=1.21 <=1.21.11"));
        assert!(!version_satisfies("1.20.1", ">=1.21 <=1.21.11"));
    }

    #[test]
    fn maven_interval() {
        assert!(version_satisfies("1.21.11", "[1.21,1.21.11]"));
        assert!(!version_satisfies("1.21.11", "[1.21,1.21.11)"));
        assert!(version_satisfies("1.21.1", "[1.21,1.21.11]"));
    }

    #[test]
    fn fabric_api_umbrella_modules() {
        assert!(is_fabric_api_module("fabric-rendering-v1"));
        assert!(is_fabric_api_module("fabric-api-base"));
        assert!(!is_fabric_api_module("fabric-language-kotlin"));
        assert!(!is_fabric_api_module("fabric-api"));
    }

    #[test]
    fn known_dependency_aliases_are_canonical() {
        assert_eq!(canonical_mod_id("cloth_config2"), "cloth-config");
        assert_eq!(canonical_mod_id("cloth-config"), "cloth-config");
        assert_eq!(canonical_mod_id("fabricloader"), "fabric-loader");
    }

    #[test]
    fn modrinth_project_slug_matches_manual_local_install() {
        let known = HashSet::new();
        let present = HashSet::from(["fabric-api".to_string()]);
        let slugs = HashMap::from([("P7dR8mSH".to_string(), "fabric-api".to_string())]);
        assert!(modrinth_project_is_present(
            "P7dR8mSH",
            &known,
            &slugs,
            &present
        ));
        // Built-in fallback protects this high-frequency dependency when the
        // project metadata request itself is unavailable.
        assert!(modrinth_project_is_present(
            "P7dR8mSH",
            &known,
            &HashMap::new(),
            &present
        ));
    }

    #[test]
    fn forge_multimod_metadata_does_not_create_self_dependency() {
        let raw = r#"
            [[mods]]
            modId = "cloth_config"
            version = "1.0"

            [[mods]]
            modId = "cloth_config_api"
            version = "1.0"

            [[dependencies.cloth_config]]
            modId = "cloth_config"
            mandatory = true
            versionRange = "*"

            [[dependencies.cloth_config_api]]
            modId = "cloth_config"
            mandatory = true
            versionRange = "*"

            [[dependencies.cloth_config_api]]
            modId = "fabric_api"
            mandatory = true
            versionRange = "*"
        "#;
        let meta = parse_mods_toml(raw, "cloth-config.jar").expect("valid Forge metadata");
        assert!(!meta.depends.contains_key("cloth_config"));
        assert_eq!(meta.depends.get("fabric_api"), Some(&"*".to_string()));
    }
}
