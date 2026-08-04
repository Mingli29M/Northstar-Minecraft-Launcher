use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftNewsItem {
    pub title: String,
    pub tag: String,
    pub date: String,
    pub text: String,
    pub image_url: Option<String>,
    pub read_more_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftPatchNote {
    pub version: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub type_: String,
}

fn absolute_mojang_url(s: &str) -> String {
    if s.starts_with("http") {
        s.to_string()
    } else if s.starts_with('/') {
        format!("https://launchercontent.mojang.com{s}")
    } else {
        format!("https://launchercontent.mojang.com/{s}")
    }
}

fn parse_news_entries(data: &Value) -> Vec<(MinecraftNewsItem, bool)> {
    let mut items = Vec::new();
    for entry in data["entries"].as_array().into_iter().flatten() {
        let title = entry["title"].as_str().unwrap_or("News").to_string();
        let tag = entry["tag"]
            .as_str()
            .or_else(|| entry["category"].as_str())
            .unwrap_or("Minecraft")
            .to_string();
        let date = entry["date"].as_str().unwrap_or("").to_string();
        let text = entry["text"]
            .as_str()
            .or_else(|| entry["articleBody"].as_str())
            .unwrap_or("")
            .to_string();
        let image_url = entry
            .pointer("/newsPageImage/url")
            .and_then(|u| u.as_str())
            .or_else(|| entry.pointer("/playPageImage/url").and_then(|u| u.as_str()))
            .map(absolute_mojang_url);
        let read_more_url = entry["readMoreLink"]
            .as_str()
            .or_else(|| entry["newsPage"].as_str())
            .or_else(|| entry.pointer("/button/url").and_then(|u| u.as_str()))
            .map(|s| s.to_string());

        let news_types: Vec<String> = entry["newsType"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| t.as_str().map(|s| s.to_string()))
            .collect();
        let category = entry["category"].as_str().unwrap_or("");
        let is_java = news_types.iter().any(|t| t.eq_ignore_ascii_case("Java"))
            || category.to_lowercase().contains("java");

        items.push((
            MinecraftNewsItem {
                title,
                tag,
                date,
                text,
                image_url,
                read_more_url,
            },
            is_java,
        ));
    }
    items
}

/// Minecraft Launcher news — use v2 feed (legacy news.json stopped updating ~2023).
pub fn fetch_minecraft_news() -> Result<Vec<MinecraftNewsItem>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .user_agent("Northstar/1.1.1")
        .build()
        .map_err(|e| e.to_string())?;

    let mut data: Option<Value> = None;
    for url in [
        "https://launchercontent.mojang.com/v2/news.json",
        "https://launchercontent.mojang.com/news.json",
    ] {
        match client.get(url).send() {
            Ok(resp) => match resp.error_for_status() {
                Ok(ok) => match ok.json::<Value>() {
                    Ok(v) => {
                        data = Some(v);
                        break;
                    }
                    Err(_) => continue,
                },
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }
    let data = data.ok_or_else(|| "Failed to fetch Minecraft news".to_string())?;

    let mut items = parse_news_entries(&data);
    // Newest first
    items.sort_by(|a, b| b.0.date.cmp(&a.0.date));

    let mut out: Vec<MinecraftNewsItem> = items
        .iter()
        .filter(|(_, java)| *java)
        .map(|(item, _)| item.clone())
        .collect();
    if out.len() < 12 {
        for (item, java) in &items {
            if *java {
                continue;
            }
            if out.iter().any(|x| x.title == item.title && x.date == item.date) {
                continue;
            }
            out.push(item.clone());
            if out.len() >= 16 {
                break;
            }
        }
    }
    out.truncate(20);
    Ok(out)
}

/// Java patch notes / changelogs — v2 feed has current snapshots (legacy freezes at 2024).
pub fn fetch_minecraft_patch_notes() -> Result<Vec<MinecraftPatchNote>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .user_agent("Northstar/1.1.1")
        .build()
        .map_err(|e| e.to_string())?;

    let mut data: Option<Value> = None;
    for url in [
        "https://launchercontent.mojang.com/v2/javaPatchNotes.json",
        "https://launchercontent.mojang.com/javaPatchNotes.json",
    ] {
        match client.get(url).send() {
            Ok(resp) => match resp.error_for_status() {
                Ok(ok) => match ok.json::<Value>() {
                    Ok(v) => {
                        data = Some(v);
                        break;
                    }
                    Err(_) => continue,
                },
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }
    let data = data.ok_or_else(|| "Failed to fetch Minecraft patch notes".to_string())?;

    let entries = data
        .get("entries")
        .and_then(|e| e.as_array())
        .cloned()
        .or_else(|| data.as_array().cloned())
        .unwrap_or_default();

    let mut out = Vec::new();
    // v2 is newest-first; still take from the start (newest), not a random middle slice.
    for entry in entries.into_iter().take(40) {
        let version = entry["version"]
            .as_str()
            .or_else(|| entry["id"].as_str())
            .unwrap_or("")
            .to_string();
        let title = entry["title"]
            .as_str()
            .or_else(|| entry["version"].as_str())
            .unwrap_or(&version)
            .to_string();
        let body = entry["body"]
            .as_str()
            .or_else(|| entry["content"].as_str())
            .or_else(|| entry["shortText"].as_str())
            .unwrap_or("")
            .to_string();
        let type_ = entry["type"]
            .as_str()
            .or_else(|| entry["type_"].as_str())
            .unwrap_or("release")
            .to_string();
        if version.is_empty() && title.is_empty() && body.is_empty() {
            continue;
        }
        out.push(MinecraftPatchNote {
            version,
            title,
            body,
            type_,
        });
    }
    Ok(out)
}
