#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{copy_to_clipboard, install, uninstall};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{attach_console, detect_browsers, install, open_url, uninstall};

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
compile_error!("choose-browser supports Linux and Windows");
