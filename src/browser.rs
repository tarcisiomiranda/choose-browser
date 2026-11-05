use std::io;
use std::path::PathBuf;

use ::image::imageops::{self, FilterType};
use ::image::{Rgba, RgbaImage};
use iced::widget::image as iced_image;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
pub use crate::platform::detect_browsers;
#[cfg(target_os = "linux")]
pub use linux::detect_browsers;

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
    pub fn open_url(&self, url: &str) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        {
            crate::platform::open_url(&self.exec, url)
        }
        #[cfg(target_os = "linux")]
        {
            let exec = self
                .exec
                .replace("%u", url)
                .replace("%U", url)
                .replace("%f", url)
                .replace("%F", url);
            let parts: Vec<&str> = exec.split_whitespace().collect();
            if let Some((cmd, args)) = parts.split_first() {
                std::process::Command::new(cmd).args(args).spawn()?;
            }
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
fn load_icon_handle(path: &std::path::Path) -> Option<iced_image::Handle> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    if matches!(extension.as_deref(), Some("svg" | "xpm")) {
        return None;
    }

    let icon = ::image::ImageReader::open(path)
        .ok()?
        .decode()
        .ok()?
        .into_rgba8();
    let normalized = normalize_icon(icon);

    Some(iced_image::Handle::from_rgba(
        ICON_CANVAS_SIZE,
        ICON_CANVAS_SIZE,
        normalized.into_raw(),
    ))
}

pub(crate) fn normalize_icon(icon: RgbaImage) -> RgbaImage {
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
