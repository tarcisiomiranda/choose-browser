use std::env;

pub fn copy_to_clipboard(text: &str) {
    let result = std::process::Command::new("wl-copy").arg(text).status();
    if matches!(result, Ok(status) if status.success()) {
        return;
    }
    let _ = std::process::Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()
        });
}

pub fn install() -> std::io::Result<()> {
    let exe_path = env::current_exe().expect("Failed to get executable path");
    let exe_str = exe_path.display();

    let desktop_entry = format!(
        r#"[Desktop Entry]
Version=1.0
Name=Choose Browser
Comment=Choose which browser to open links with
Exec={} %u
Icon=choose-browser
Terminal=false
Type=Application
Categories=Network;WebBrowser;
MimeType=x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/about;x-scheme-handler/unknown;text/html;
StartupNotify=false
"#,
        exe_str
    );

    let data_dir = dirs::data_dir().expect("Cannot find data directory");
    let applications_dir = data_dir.join("applications");
    std::fs::create_dir_all(&applications_dir).expect("Failed to create applications directory");

    let desktop_path = applications_dir.join("choose-browser.desktop");
    std::fs::write(&desktop_path, &desktop_entry).expect("Failed to write .desktop file");

    println!("Installed .desktop file to: {}", desktop_path.display());

    // Register as default browser using xdg-mime
    let schemes = ["x-scheme-handler/http", "x-scheme-handler/https"];

    for scheme in &schemes {
        let status = std::process::Command::new("xdg-mime")
            .args(["default", "choose-browser.desktop", scheme])
            .status();

        match status {
            Ok(s) if s.success() => println!("Set default for {}", scheme),
            _ => eprintln!("Warning: failed to set default for {}", scheme),
        }
    }

    // Also try xdg-settings for the default web browser
    let status = std::process::Command::new("xdg-settings")
        .args(["set", "default-web-browser", "choose-browser.desktop"])
        .status();

    match status {
        Ok(s) if s.success() => println!("Set as default web browser via xdg-settings"),
        _ => eprintln!("Warning: xdg-settings command failed (this is normal on some DEs)"),
    }

    // Update desktop database
    let _ = std::process::Command::new("update-desktop-database")
        .arg(applications_dir.to_str().unwrap())
        .status();

    println!("\nChoose Browser installed successfully!");
    println!("It is now set as your default browser.");
    println!("To uninstall, run: choose-browser --uninstall");
    Ok(())
}

pub fn uninstall() -> std::io::Result<()> {
    let data_dir = dirs::data_dir().expect("Cannot find data directory");
    let desktop_path = data_dir.join("applications/choose-browser.desktop");

    // Check if we are the current default
    let prev_browser = std::process::Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();

    if desktop_path.exists() {
        std::fs::remove_file(&desktop_path).expect("Failed to remove .desktop file");
        println!("Removed .desktop file: {}", desktop_path.display());
    } else {
        println!("No .desktop file found at: {}", desktop_path.display());
    }

    // Reset to a known browser if we were the default
    if prev_browser == "choose-browser.desktop" {
        let fallbacks = [
            "firefox.desktop",
            "google-chrome.desktop",
            "chromium-browser.desktop",
        ];
        for fallback in &fallbacks {
            let status = std::process::Command::new("xdg-settings")
                .args(["set", "default-web-browser", fallback])
                .status();
            if let Ok(s) = status {
                if s.success() {
                    println!("Reset default browser to: {}", fallback);
                    break;
                }
            }
        }
    }

    let _ = std::process::Command::new("update-desktop-database")
        .arg(data_dir.join("applications").to_str().unwrap())
        .status();

    println!("\nChoose Browser uninstalled successfully!");
    Ok(())
}
