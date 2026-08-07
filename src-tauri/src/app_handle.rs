//! Process-wide `AppHandle`.
//!
//! Low-level helpers (single-file downloads, extraction, sidecar management)
//! are reached through call chains that never carried an `AppHandle`, so their
//! progress never reached the UI. Storing the handle once at startup lets those
//! helpers emit console/progress events without rewriting every signature.

use std::sync::OnceLock;
use tauri::AppHandle;

static HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set(app: AppHandle) {
    let _ = HANDLE.set(app);
}

pub fn get() -> Option<&'static AppHandle> {
    HANDLE.get()
}
