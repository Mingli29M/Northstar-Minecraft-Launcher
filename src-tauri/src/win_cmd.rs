//! Windows process helpers: hide console windows for helper tools / server Java.

use std::process::Command;

/// CREATE_NO_WINDOW — suppress visible consoles for console-subsystem binaries
/// (powershell, netstat, netsh, java.exe) when the launcher itself has no console.
#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Apply CREATE_NO_WINDOW on Windows; no-op elsewhere.
pub fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}

/// Combine existing creation flags with CREATE_NO_WINDOW on Windows.
#[cfg(windows)]
pub fn with_no_window(flags: u32) -> u32 {
    flags | CREATE_NO_WINDOW
}
