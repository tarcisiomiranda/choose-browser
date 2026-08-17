use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ::image::imageops::{self, FilterType};
use ::image::{ImageReader, Rgba, RgbaImage};
use iced::widget::image as iced_image;

const ICON_CANVAS_SIZE: u32 = 32;
const ICON_CONTENT_SIZE: u32 = 24;
const FULL_CANVAS_INSET_DIVISOR: u32 = 14;

#[derive(Clone)]
pub struct Browser {
    pub name: String,
    pub exec: String,
    pub icon: Option<iced_image::Handle>,
    pub icon_path: Option<PathBuf>,
    pub desktop_file: String,
}

impl Browser {
    pub fn open_url(&self, url: &str) {
        let exec = self
            .exec
            .replace("%u", url)
            .replace("%U", url)
            .replace("%f", url)
            .replace("%F", url);

        let parts: Vec<&str> = exec.split_whitespace().collect();
        if let Some((cmd, args)) = parts.split_first() {
            let _ = Command::new(cmd).args(args).spawn();
        }
    }
}

pub fn detect_browsers() -> Vec<Browser> {
    let known_browsers = [
        "firefox",
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "brave-browser",
        "vivaldi",
        "vivaldi-stable",
        "opera",
        "opera-stable",
        "microsoft-edge",
        "microsoft-edge-stable",
        "epiphany",
        "epiphany-browser",
        "falkon",
        "konqueror",
        "midori",
        "zen-browser",
        "floorp",
        "waterfox",
        "librewolf",
        "thorium-browser",
        "tor-browser",
    ];

    let desktop_dirs = get_desktop_dirs();
    let mut browsers: Vec<Browser> = Vec::new();
    let mut seen: HashMap<String, bool> = HashMap::new();

    for dir in &desktop_dirs {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "desktop") {
                continue;
            }

            if let Some(browser) = parse_desktop_file(&path, &known_browsers) {
                let key = browser.name.clone();
                if !seen.contains_key(&key) {
                    seen.insert(key, true);
                    browsers.push(browser);
                }
            }
        }
    }

    browsers.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    browsers
}

fn get_desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));

    if let Some(data_home) = dirs::data_dir() {
        dirs.push(data_home.join("applications"));
    }

    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_data_dirs.split(':') {
            let p = PathBuf::from(dir).join("applications");
            if !dirs.contains(&p) {
                dirs.push(p);
            }
        }
    }

    // Flatpak and Snap locations
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    if let Some(data_home) = dirs::data_dir() {
        dirs.push(data_home.join("flatpak/exports/share/applications"));
    }
    dirs.push(PathBuf::from("/snap/bin/../share/applications"));

    dirs
}

fn parse_desktop_file(path: &Path, known_browsers: &[&str]) -> Option<Browser> {
    let content = fs::read_to_string(path).ok()?;

    let mut name = String::new();
    let mut exec = String::new();
    let mut icon = String::new();
    let mut categories = String::new();
    let mut mime_types = String::new();
    let mut no_display = false;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed != "[Desktop Entry]" {
            if in_desktop_entry {
                break;
            }
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        if let Some(val) = trimmed.strip_prefix("Name=") {
            if name.is_empty() {
                name = val.to_string();
            }
        } else if let Some(val) = trimmed.strip_prefix("Exec=") {
            exec = val.to_string();
        } else if let Some(val) = trimmed.strip_prefix("Icon=") {
            icon = val.to_string();
        } else if let Some(val) = trimmed.strip_prefix("Categories=") {
            categories = val.to_string();
        } else if let Some(val) = trimmed.strip_prefix("MimeType=") {
            mime_types = val.to_string();
        } else if trimmed == "NoDisplay=true" {
            no_display = true;
        }
    }

    if name.is_empty() || exec.is_empty() || no_display {
        return None;
    }

    let file_stem = path.file_stem()?.to_str()?;

    let is_browser = known_browsers.iter().any(|b| file_stem.contains(b))
        || categories.to_lowercase().contains("webbrowser")
        || (mime_types.contains("x-scheme-handler/http")
            && mime_types.contains("x-scheme-handler/https"));

    // Skip ourselves
    if file_stem == "choose-browser" {
        return None;
    }

    if !is_browser {
        return None;
    }

    let icon_path = find_icon(&icon);
    let icon_handle = icon_path.as_ref().and_then(|path| load_icon_handle(path));

    Some(Browser {
        name,
        exec,
        icon: icon_handle,
        icon_path,
        desktop_file: file_stem.to_string(),
    })
}

fn find_icon(icon_name: &str) -> Option<PathBuf> {
    // If it's already a full path
    let p = PathBuf::from(icon_name);
    if p.is_absolute() && p.exists() {
        return Some(p);
    }

    if let Some(path) = find_icon_in_common_dirs(icon_name) {
        return Some(path);
    }

    // Try freedesktop-icons lookup
    if let Some(path) = freedesktop_icons::lookup(icon_name).with_size(128).find() {
        return Some(path);
    }

    if let Some(path) = freedesktop_icons::lookup(icon_name).with_size(64).find() {
        return Some(path);
    }

    None
}

fn find_icon_in_common_dirs(icon_name: &str) -> Option<PathBuf> {
    let sizes = [
        "256x256", "128x128", "64x64", "48x48", "32x32", "24x24", "16x16", "scalable",
    ];
    let themes = ["hicolor", "Adwaita", "breeze", "Papirus"];
    let extensions = ["png", "svg", "xpm"];
    let icon_dirs = ["/usr/share/icons"];
    let pixmap_dirs = ["/usr/share/pixmaps"];

    for base in &icon_dirs {
        for theme in &themes {
            for size in &sizes {
                let category = if *size == "scalable" {
                    "scalable"
                } else {
                    *size
                };
                for ext in &extensions {
                    let path = PathBuf::from(base)
                        .join(theme)
                        .join(category)
                        .join("apps")
                        .join(format!("{}.{}", icon_name, ext));
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    for base in &pixmap_dirs {
        for ext in &extensions {
            let path = PathBuf::from(base).join(format!("{}.{}", icon_name, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

fn load_icon_handle(path: &Path) -> Option<iced_image::Handle> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    if matches!(extension.as_deref(), Some("svg" | "xpm")) {
        return None;
    }

    let icon = ImageReader::open(path).ok()?.decode().ok()?.into_rgba8();
    let normalized = normalize_icon(icon);

    Some(iced_image::Handle::from_rgba(
        ICON_CANVAS_SIZE,
        ICON_CANVAS_SIZE,
        normalized.into_raw(),
    ))
}

fn normalize_icon(icon: RgbaImage) -> RgbaImage {
    let cropped = crop_transparent_padding(&icon);
    let (source_width, source_height) = cropped.dimensions();
    let scale = (ICON_CONTENT_SIZE as f32 / source_width as f32)
        .min(ICON_CONTENT_SIZE as f32 / source_height as f32);
    let target_width = ((source_width as f32 * scale).round() as u32).max(1);
    let target_height = ((source_height as f32 * scale).round() as u32).max(1);

    let resized = imageops::resize(&cropped, target_width, target_height, FilterType::Lanczos3);
    let mut canvas = RgbaImage::from_pixel(ICON_CANVAS_SIZE, ICON_CANVAS_SIZE, Rgba([0, 0, 0, 0]));

    let offset_x = (ICON_CANVAS_SIZE - target_width) / 2;
    let offset_y = (ICON_CANVAS_SIZE - target_height) / 2;
    imageops::overlay(
        &mut canvas,
        &resized,
        i64::from(offset_x),
        i64::from(offset_y),
    );

    canvas
}

fn crop_transparent_padding(icon: &RgbaImage) -> RgbaImage {
    let (width, height) = icon.dimensions();
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found_visible_pixel = false;

    for (x, y, pixel) in icon.enumerate_pixels() {
        if pixel[3] == 0 {
            continue;
        }

        found_visible_pixel = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    if !found_visible_pixel {
        return icon.clone();
    }

    let cropped =
        imageops::crop_imm(icon, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1).to_image();

    if cropped.dimensions() == (width, height) && has_transparent_corners(icon) {
        return crop_full_canvas_icon(icon);
    }

    cropped
}

fn has_transparent_corners(icon: &RgbaImage) -> bool {
    let (width, height) = icon.dimensions();
    let corners = [
        icon.get_pixel(0, 0),
        icon.get_pixel(width - 1, 0),
        icon.get_pixel(0, height - 1),
        icon.get_pixel(width - 1, height - 1),
    ];

    corners.iter().all(|pixel| pixel[3] == 0)
}

fn crop_full_canvas_icon(icon: &RgbaImage) -> RgbaImage {
    let (width, height) = icon.dimensions();
    let inset = (width.min(height) / FULL_CANVAS_INSET_DIVISOR).max(1);

    imageops::crop_imm(
        icon,
        inset,
        inset,
        width.saturating_sub(inset * 2),
        height.saturating_sub(inset * 2),
    )
    .to_image()
}
