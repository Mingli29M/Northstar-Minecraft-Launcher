use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

const MAX_LINES: usize = 2000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLine {
    pub text: String,
    /// info | progress | warn | error | game
    pub level: String,
    pub ts: String,
}

static LINES: Mutex<VecDeque<ConsoleLine>> = Mutex::new(VecDeque::new());

fn now_ts() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

pub fn append(app: Option<&AppHandle>, text: impl Into<String>, level: &str) {
    let line = ConsoleLine {
        text: text.into(),
        level: level.to_string(),
        ts: now_ts(),
    };
    if let Ok(mut lock) = LINES.lock() {
        lock.push_back(line.clone());
        while lock.len() > MAX_LINES {
            lock.pop_front();
        }
    }
    if let Some(app) = app {
        let _ = app.emit("euml:console-line", &line);
    }
}

pub fn history() -> Vec<ConsoleLine> {
    LINES
        .lock()
        .map(|l| l.iter().cloned().collect())
        .unwrap_or_default()
}

pub fn clear(app: Option<&AppHandle>) {
    if let Ok(mut lock) = LINES.lock() {
        lock.clear();
    }
    if let Some(app) = app {
        let _ = app.emit(
            "euml:console-cleared",
            serde_json::json!({ "ok": true }),
        );
    }
}
