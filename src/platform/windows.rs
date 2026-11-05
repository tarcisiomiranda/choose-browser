use crate::browser::{normalize_icon, Browser};
use iced::widget::image::Handle;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::io;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{LocalFree, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
use windows_sys::Win32::System::Console::{
    AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE,
};
use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsW;
use windows_sys::Win32::UI::Shell::{
    CommandLineToArgvW, ExtractIconExW, SHChangeNotify, ShellExecuteW, SHCNE_ASSOCCHANGED,
    SHCNF_IDLIST,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, DrawIconEx, DI_NORMAL, HICON, SW_SHOWNORMAL,
};
use winreg::enums::{KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, KEY_WRITE};
use winreg::{RegKey, HKCU, HKLM};

const CLIENTS: &str = r"Software\Clients\StartMenuInternet";
const CLIENT_ID: &str = "ChooseBrowser";
const APP_NAME: &str = "Choose Browser";
const PROG_ID: &str = "ChooseBrowser.Url";
const REGISTERED_APPS: &str = r"Software\RegisteredApplications";

pub fn attach_console() {
    unsafe {
        let output = GetStdHandle(STD_OUTPUT_HANDLE);
        if output.is_null() || output == INVALID_HANDLE_VALUE {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

pub fn detect_browsers() -> Vec<Browser> {
    let mut browsers = Vec::new();
    let mut seen = HashSet::new();
    for root in [HKCU, HKLM] {
        for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
            if let Ok(clients) = root.open_subkey_with_flags(CLIENTS, KEY_READ | view) {
                collect_browsers(&clients, &mut browsers, &mut seen);
            }
        }
    }
    browsers.sort_by_key(|browser| browser.name.to_lowercase());
    browsers
}

fn collect_browsers(clients: &RegKey, browsers: &mut Vec<Browser>, seen: &mut HashSet<String>) {
    for id in clients.enum_keys().flatten() {
        if id.eq_ignore_ascii_case(CLIENT_ID) {
            continue;
        }
        let Some(browser) = clients
            .open_subkey(&id)
            .ok()
            .and_then(|key| read_browser(&key, &id))
        else {
            continue;
        };
        let Ok(parts) = parse_command(&browser.exec) else {
            continue;
        };
        let executable = parts[0].to_string_lossy().to_lowercase();
        if seen.insert(executable) {
            browsers.push(browser);
        }
    }
}

fn read_browser(key: &RegKey, id: &str) -> Option<Browser> {
    let exec: String = key
        .open_subkey(r"shell\open\command")
        .ok()?
        .get_value("")
        .ok()?;
    let parts = parse_command(&exec).ok()?;
    let executable = PathBuf::from(&parts[0]);
    if !executable.is_file()
        || executable
            .file_name()?
            .to_string_lossy()
            .eq_ignore_ascii_case("choose-browser.exe")
    {
        return None;
    }

    let name = key
        .get_value::<String, _>("")
        .ok()
        .filter(|value| !value.trim().is_empty() && !value.starts_with('@'))
        .unwrap_or_else(|| id.to_string());
    let (icon_path, icon_index) = key
        .open_subkey("DefaultIcon")
        .ok()
        .and_then(|icon| icon.get_value::<String, _>("").ok())
        .and_then(|value| parse_icon_location(&value).ok())
        .unwrap_or_else(|| (executable.clone(), 0));
    let icon = load_icon(&icon_path, icon_index).or_else(|| load_icon(&executable, 0));

    Some(Browser {
        name,
        exec,
        icon,
        icon_path: Some(icon_path),
        desktop_file: id.to_string(),
    })
}

fn wide(value: impl AsRef<OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}

fn expand_environment(value: &str) -> io::Result<OsString> {
    let input = wide(value);
    unsafe {
        let needed = ExpandEnvironmentStringsW(input.as_ptr(), null_mut(), 0);
        if needed == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut output = vec![0; needed as usize];
        let written = ExpandEnvironmentStringsW(input.as_ptr(), output.as_mut_ptr(), needed);
        if written == 0 || written > needed {
            return Err(io::Error::other("Could not expand browser command"));
        }
        Ok(OsString::from_wide(&output[..written as usize - 1]))
    }
}

fn parse_command(exec: &str) -> io::Result<Vec<OsString>> {
    if exec.trim().is_empty() || exec.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Empty or invalid browser command",
        ));
    }
    let input = wide(expand_environment(exec)?);
    unsafe {
        let mut count = 0;
        let argv = CommandLineToArgvW(input.as_ptr(), &mut count);
        if argv.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut parts = Vec::with_capacity(count as usize);
        for pointer in std::slice::from_raw_parts(argv, count as usize) {
            let mut length = 0;
            while *pointer.add(length) != 0 {
                length += 1;
            }
            parts.push(OsString::from_wide(std::slice::from_raw_parts(
                *pointer, length,
            )));
        }
        LocalFree(argv.cast());
        if parts.first().is_none_or(|part| part.is_empty()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Missing browser executable",
            ));
        }
        Ok(parts)
    }
}

fn browser_command(exec: &str, url: &str) -> io::Result<Command> {
    let parts = parse_command(exec)?;
    let mut command = Command::new(&parts[0]);
    let mut replaced = false;
    for arg in &parts[1..] {
        let value = arg.to_string_lossy();
        let placeholders = ["%1", "%l", "%L", "%u", "%U", "%*"];
        if placeholders
            .iter()
            .any(|placeholder| value.contains(placeholder))
        {
            replaced = true;
            if !url.is_empty() {
                let mut value = value.into_owned();
                for placeholder in placeholders {
                    value = value.replace(placeholder, "\0");
                }
                command.arg(value.replace('\0', url));
            }
        } else {
            command.arg(arg);
        }
    }
    if !replaced && !url.is_empty() {
        command.arg(url);
    }
    Ok(command)
}

pub fn open_url(exec: &str, url: &str) -> io::Result<()> {
    browser_command(exec, url)?.spawn().map(|_| ())
}

fn parse_icon_location(value: &str) -> io::Result<(PathBuf, i32)> {
    let (path, index) = value
        .rsplit_once(',')
        .and_then(|(path, index)| index.trim().parse::<i32>().ok().map(|index| (path, index)))
        .unwrap_or((value, 0));
    let path = path.trim().trim_matches('"');
    Ok((PathBuf::from(expand_environment(path)?), index))
}

fn load_icon(path: &Path, index: i32) -> Option<Handle> {
    let path = wide(path);
    unsafe {
        let mut icon = null_mut();
        if ExtractIconExW(path.as_ptr(), index, &mut icon, null_mut(), 1) == 0 || icon.is_null() {
            return None;
        }
        let pixels = icon_pixels(icon);
        DestroyIcon(icon);
        let normalized = normalize_icon(pixels?);
        Some(Handle::from_rgba(
            normalized.width(),
            normalized.height(),
            normalized.into_raw(),
        ))
    }
}

unsafe fn icon_pixels(icon: HICON) -> Option<::image::RgbaImage> {
    let dc = CreateCompatibleDC(null_mut());
    if dc.is_null() {
        return None;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: 32,
            biHeight: -32,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits = null_mut();
    let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    if bitmap.is_null() {
        DeleteDC(dc);
        return None;
    }
    let previous = SelectObject(dc, bitmap);
    std::ptr::write_bytes(bits, 0, 32 * 32 * 4);
    let drawn = DrawIconEx(dc, 0, 0, icon, 32, 32, 0, null_mut(), DI_NORMAL);
    GdiFlush();
    let mut pixels = std::slice::from_raw_parts(bits.cast::<u8>(), 32 * 32 * 4).to_vec();
    SelectObject(dc, previous);
    DeleteObject(bitmap);
    DeleteDC(dc);
    if drawn == 0 {
        return None;
    }
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    ::image::RgbaImage::from_raw(32, 32, pixels)
}

fn set_value(root: &RegKey, path: &str, name: &str, value: &str) -> io::Result<()> {
    let (key, _) = root.create_subkey(path)?;
    key.set_value(name, &value)
}

fn register(root: &RegKey, executable: &Path) -> io::Result<()> {
    let client = format!(r"{CLIENTS}\{CLIENT_ID}");
    let capabilities = format!(r"{client}\Capabilities");
    let class = format!(r"Software\Classes\{PROG_ID}");
    let command = format!("\"{}\" \"%1\"", executable.display());
    let icon = format!("\"{}\",0", executable.display());

    set_value(root, &client, "", APP_NAME)?;
    set_value(root, &format!(r"{client}\DefaultIcon"), "", &icon)?;
    set_value(root, &format!(r"{client}\shell\open\command"), "", &command)?;
    set_value(root, &capabilities, "ApplicationName", APP_NAME)?;
    set_value(
        root,
        &capabilities,
        "ApplicationDescription",
        "Choose which browser opens your links",
    )?;
    set_value(root, &capabilities, "ApplicationIcon", &icon)?;
    for scheme in ["http", "https"] {
        set_value(
            root,
            &format!(r"{capabilities}\URLAssociations"),
            scheme,
            PROG_ID,
        )?;
    }
    set_value(
        root,
        &format!(r"{capabilities}\Startmenu"),
        "StartMenuInternet",
        CLIENT_ID,
    )?;
    set_value(root, &class, "", "Choose Browser URL")?;
    set_value(root, &class, "URL Protocol", "")?;
    set_value(root, &format!(r"{class}\DefaultIcon"), "", &icon)?;
    set_value(root, &format!(r"{class}\shell\open\command"), "", &command)?;
    set_value(root, REGISTERED_APPS, APP_NAME, &capabilities)
}

fn ignore_missing(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

fn unregister(root: &RegKey) -> io::Result<()> {
    match root.open_subkey_with_flags(REGISTERED_APPS, KEY_WRITE) {
        Ok(key) => ignore_missing(key.delete_value(APP_NAME))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    ignore_missing(root.delete_subkey_all(format!(r"{CLIENTS}\{CLIENT_ID}")))?;
    ignore_missing(root.delete_subkey_all(format!(r"Software\Classes\{PROG_ID}")))
}

fn notify_associations() {
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED as i32, SHCNF_IDLIST, null(), null());
    }
}

fn open_default_settings() -> io::Result<()> {
    let uri = wide("ms-settings:defaultapps?registeredAppUser=Choose%20Browser");
    let verb = wide("open");
    let result = unsafe {
        ShellExecuteW(
            null_mut(),
            verb.as_ptr(),
            uri.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        )
    };
    if result as isize <= 32 {
        return Err(io::Error::other(
            "Open Settings > Apps > Default apps and select Choose Browser",
        ));
    }
    Ok(())
}

pub fn install() -> io::Result<()> {
    register(HKCU, &std::env::current_exe()?)?;
    notify_associations();
    println!("Choose Browser registered for the current user.");
    println!("In Settings > Apps > Default apps, select Choose Browser for HTTP and HTTPS.");
    if let Err(error) = open_default_settings() {
        eprintln!("{error}");
    }
    Ok(())
}

pub fn uninstall() -> io::Result<()> {
    unregister(HKCU)?;
    notify_associations();
    println!("Choose Browser registration removed. Your configuration has been kept.");
    println!("Select your preferred browser in Settings > Apps > Default apps.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RegistrySandbox(String);

    impl RegistrySandbox {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(format!(
                r"Software\ChooseBrowserTests\{}-{nonce}",
                std::process::id()
            ))
        }

        fn key(&self) -> RegKey {
            HKCU.create_subkey(&self.0).unwrap().0
        }
    }

    impl Drop for RegistrySandbox {
        fn drop(&mut self) {
            HKCU.delete_subkey_all(&self.0).unwrap();
        }
    }

    fn args(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn command_preserves_paths_flags_and_url() {
        let url = "https://example.com/ação com espaço?q=a&b=%1%20%L\"|calc.exe";
        let command = browser_command(
            r#""C:\Program Files\Browser\browser.exe" --profile-directory="Profile 1" "%1""#,
            url,
        )
        .unwrap();
        assert_eq!(
            command.get_program(),
            r"C:\Program Files\Browser\browser.exe"
        );
        assert_eq!(args(&command), ["--profile-directory=Profile 1", url]);
    }

    #[test]
    fn command_appends_url_and_handles_empty_url() {
        let command = browser_command(
            r#""C:\Browser\browser.exe" --new-window"#,
            "https://example.com",
        )
        .unwrap();
        assert_eq!(args(&command), ["--new-window", "https://example.com"]);
        let command = browser_command(r#""C:\Browser\browser.exe" -osint -url "%L""#, "").unwrap();
        assert_eq!(args(&command), ["-osint", "-url"]);
        assert!(browser_command("", "https://example.com").is_err());
    }

    #[test]
    fn command_expands_environment_in_executable() {
        let command =
            browser_command(r#""%SystemRoot%\explorer.exe" "%1""#, "https://example.com").unwrap();
        assert_eq!(
            Path::new(command.get_program()),
            Path::new(&std::env::var_os("SystemRoot").unwrap()).join("explorer.exe")
        );
    }

    #[test]
    fn icon_location_handles_quotes_commas_and_resource_ids() {
        assert_eq!(
            parse_icon_location(r#""C:\Program Files\Browser, Inc\browser.exe",-7"#).unwrap(),
            (
                PathBuf::from(r"C:\Program Files\Browser, Inc\browser.exe"),
                -7
            )
        );
    }

    #[test]
    fn registration_is_idempotent_and_uninstall_preserves_other_apps() {
        let sandbox = RegistrySandbox::new();
        let root = sandbox.key();
        let exe = Path::new(r"C:\Program Files\Choose Browser\choose-browser.exe");
        set_value(
            &root,
            REGISTERED_APPS,
            "Other Browser",
            r"Software\Other\Capabilities",
        )
        .unwrap();
        set_value(&root, r"Software\Classes\OtherBrowser", "", "Other Browser").unwrap();
        register(&root, exe).unwrap();
        register(&root, exe).unwrap();
        let associations = root
            .open_subkey(format!(
                r"{CLIENTS}\{CLIENT_ID}\Capabilities\URLAssociations"
            ))
            .unwrap();
        assert_eq!(
            associations.get_value::<String, _>("https").unwrap(),
            PROG_ID
        );
        let command: String = root
            .open_subkey(format!(r"Software\Classes\{PROG_ID}\shell\open\command"))
            .unwrap()
            .get_value("")
            .unwrap();
        let parsed = browser_command(&command, "https://example.com/a b").unwrap();
        assert_eq!(parsed.get_program(), exe);
        assert_eq!(args(&parsed), ["https://example.com/a b"]);
        unregister(&root).unwrap();
        unregister(&root).unwrap();
        assert!(root.open_subkey(format!(r"{CLIENTS}\{CLIENT_ID}")).is_err());
        assert!(root
            .open_subkey(format!(r"Software\Classes\{PROG_ID}"))
            .is_err());
        assert!(root.open_subkey(r"Software\Classes\OtherBrowser").is_ok());
        assert!(root
            .open_subkey(REGISTERED_APPS)
            .unwrap()
            .get_value::<String, _>("Other Browser")
            .is_ok());
    }

    #[test]
    fn discovery_skips_self_missing_executables_and_duplicates() {
        let sandbox = RegistrySandbox::new();
        let clients = sandbox.key();
        let exe = std::env::current_exe().unwrap();
        for id in ["Browser", "Duplicate", CLIENT_ID] {
            set_value(
                &clients,
                &format!(r"{id}\shell\open\command"),
                "",
                &format!("\"{}\" \"%1\"", exe.display()),
            )
            .unwrap();
        }
        set_value(
            &clients,
            r"Missing\shell\open\command",
            "",
            r#""C:\nonexistent\browser.exe" "%1""#,
        )
        .unwrap();
        let mut browsers = Vec::new();
        collect_browsers(&clients, &mut browsers, &mut HashSet::new());
        assert_eq!(browsers.len(), 1);
        assert_eq!(browsers[0].desktop_file, "Browser");
    }
}
