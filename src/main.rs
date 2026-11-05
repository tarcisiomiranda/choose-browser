#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod browser;
mod config;
mod platform;

use browser::{detect_browsers, Browser};
use config::{apply_browser_config, config_path, init_config_file, load_config};
use iced::widget::{
    button, column, container, horizontal_rule, image, row, scrollable, text, Space,
};
use iced::{
    border, keyboard, window, Alignment, Background, Color, Element, Length, Padding, Shadow, Size,
    Subscription, Task as IcedTask, Theme,
};
use std::env;

fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();
    #[cfg(target_os = "windows")]
    if args.iter().skip(1).any(|arg| arg.starts_with("--")) {
        platform::attach_console();
    }
    let all_browsers = detect_browsers();
    let config = load_config_or_default();
    let browsers = apply_browser_config(all_browsers.clone(), &config);

    if args.len() > 1 {
        match args[1].as_str() {
            "--install" => {
                run_registration(platform::install);
                return Ok(());
            }
            "--uninstall" => {
                run_registration(platform::uninstall);
                return Ok(());
            }
            "--config-path" => {
                println!("{}", config_path().display());
                return Ok(());
            }
            "--init-config" => {
                match init_config_file(&all_browsers) {
                    Ok(path) => println!("Config file ready at: {}", path.display()),
                    Err(error) => eprintln!("Failed to initialize config: {error}"),
                }
                return Ok(());
            }
            "--list-all" => {
                print_browser_list("Detected browsers", &all_browsers);
                return Ok(());
            }
            "--list" => {
                print_browser_list("Browsers after config", &browsers);
                return Ok(());
            }
            _ => {}
        }
    }

    let url = args
        .iter()
        .skip(1)
        .find(|a| a.starts_with("http://") || a.starts_with("https://") || a.starts_with("file://"))
        .cloned()
        .unwrap_or_default();

    if browsers.is_empty() {
        eprintln!(
            "No browsers available after applying the config. Check: {}",
            config_path().display()
        );
        return Ok(());
    }

    if browsers.len() == 1 {
        if let Err(error) = browsers[0].open_url(&url) {
            eprintln!("Failed to open browser: {error}");
            std::process::exit(1);
        }
        return Ok(());
    }

    let settings = window::Settings {
        size: Size::new(300.0, compute_window_height(browsers.len())),
        resizable: false,
        decorations: true,
        level: window::Level::AlwaysOnTop,
        position: window::Position::Centered,
        ..Default::default()
    };

    iced::application("Choose Browser", App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(settings)
        .run_with(move || (App { url, browsers }, IcedTask::none()))
}

const MAX_VISIBLE: usize = 5;

fn compute_window_height(count: usize) -> f32 {
    let header = 40.0;
    let visible = count.min(MAX_VISIBLE);
    let per_item = 52.0;
    let padding = 24.0;
    header + (visible as f32 * per_item) + padding
}

#[derive(Debug, Clone)]
enum Message {
    BrowserSelected(usize),
    CopyUrl,
    CloseRequested,
}

struct App {
    url: String,
    browsers: Vec<Browser>,
}

impl App {
    fn update(&mut self, message: Message) -> IcedTask<Message> {
        match message {
            Message::BrowserSelected(idx) => {
                if let Some(browser) = self.browsers.get(idx) {
                    if let Err(error) = browser.open_url(&self.url) {
                        eprintln!("Failed to open browser: {error}");
                    }
                }
                iced::exit()
            }
            Message::CopyUrl => {
                #[cfg(target_os = "windows")]
                {
                    iced::clipboard::write(self.url.clone()).chain(iced::exit())
                }
                #[cfg(target_os = "linux")]
                {
                    platform::copy_to_clipboard(&self.url);
                    iced::exit()
                }
            }
            Message::CloseRequested => iced::exit(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let url_display = if self.url.is_empty() {
            "No URL".to_string()
        } else if self.url.len() > 42 {
            format!("{}...", self.url.chars().take(39).collect::<String>())
        } else {
            self.url.clone()
        };

        let copy_btn = button(
            text("Copy (c)")
                .size(11)
                .color(Color::from_rgba8(180, 180, 180, 0.8)),
        )
        .on_press(Message::CopyUrl)
        .style(copy_button_style)
        .padding(Padding::from([2, 8]));

        let header = row![
            text(url_display)
                .size(11)
                .color(Color::from_rgba8(180, 180, 180, 0.8)),
            Space::new(Length::Fill, 0),
            copy_btn,
        ]
        .align_y(Alignment::Center)
        .padding(Padding::from([6, 12]));

        let mut browser_list = column![].spacing(2).width(Length::Fill);

        for (idx, browser) in self.browsers.iter().enumerate() {
            let icon_element: Element<Message> = if let Some(icon) = browser.icon.clone() {
                image::Image::new(icon).width(36).height(36).into()
            } else {
                Space::new(36, 36).into()
            };

            let label = text(browser.name.clone()).size(16);

            let shortcut_element: Element<Message> = if let Some(shortcut) = shortcut_label(idx) {
                text(format!("{shortcut}"))
                    .size(14)
                    .color(Color::from_rgba8(180, 180, 180, 0.7))
                    .into()
            } else {
                Space::new(0, 0).into()
            };

            let row_content = row![
                icon_element,
                label,
                Space::new(Length::Fill, 0),
                shortcut_element,
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .padding(Padding::from([6, 12]));

            let btn = button(row_content)
                .on_press(Message::BrowserSelected(idx))
                .width(Length::Fill)
                .style(browser_button_style);

            browser_list = browser_list.push(btn);
        }

        let list = browser_list.padding(6);

        let content: Element<Message> = if self.browsers.len() > MAX_VISIBLE {
            scrollable(list)
                .height(Length::Fill)
                .width(Length::Fill)
                .into()
        } else {
            list.into()
        };

        let page = column![header, horizontal_rule(1), content,].width(Length::Fill);

        container(page)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(shortcut_message)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn browser_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let transparent = Color::TRANSPARENT;
    let highlight = Color::from_rgba8(60, 120, 220, 0.85);
    let pressed = Color::from_rgba8(45, 100, 195, 0.9);
    let white = Color::WHITE;

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(transparent)),
            text_color: white,
            border: border::rounded(8),
            shadow: Shadow::default(),
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(highlight)),
            text_color: white,
            border: border::rounded(8),
            shadow: Shadow::default(),
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(pressed)),
            text_color: white,
            border: border::rounded(8),
            shadow: Shadow::default(),
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(transparent)),
            text_color: Color::from_rgba8(180, 180, 180, 0.5),
            border: border::rounded(8),
            shadow: Shadow::default(),
        },
    }
}

fn copy_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let base = Color::from_rgba8(180, 180, 180, 0.15);
    let hover = Color::from_rgba8(180, 180, 180, 0.3);

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(base)),
            text_color: Color::WHITE,
            border: border::rounded(4),
            shadow: Shadow::default(),
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(hover)),
            text_color: Color::WHITE,
            border: border::rounded(4),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(base)),
            text_color: Color::WHITE,
            border: border::rounded(4),
            shadow: Shadow::default(),
        },
    }
}

fn shortcut_message(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
    if modifiers.alt() || modifiers.command() || modifiers.logo() {
        return None;
    }

    match key.as_ref() {
        keyboard::Key::Character("1") => Some(Message::BrowserSelected(0)),
        keyboard::Key::Character("2") => Some(Message::BrowserSelected(1)),
        keyboard::Key::Character("3") => Some(Message::BrowserSelected(2)),
        keyboard::Key::Character("4") => Some(Message::BrowserSelected(3)),
        keyboard::Key::Character("5") => Some(Message::BrowserSelected(4)),
        keyboard::Key::Character("6") => Some(Message::BrowserSelected(5)),
        keyboard::Key::Character("7") => Some(Message::BrowserSelected(6)),
        keyboard::Key::Character("8") => Some(Message::BrowserSelected(7)),
        keyboard::Key::Character("9") => Some(Message::BrowserSelected(8)),
        keyboard::Key::Character("c") => Some(Message::CopyUrl),
        keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::CloseRequested),
        _ => None,
    }
}

fn shortcut_label(index: usize) -> Option<usize> {
    if index < 9 {
        Some(index + 1)
    } else {
        None
    }
}

fn load_config_or_default() -> config::AppConfig {
    match load_config() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Warning: {error}");
            config::AppConfig::default()
        }
    }
}

fn print_browser_list(title: &str, browsers: &[Browser]) {
    println!("{title}:");
    for browser in browsers {
        println!(
            "  {} ({}) [icon: {}]",
            browser.name,
            browser.desktop_file,
            browser
                .icon_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "none".to_string())
        );
    }
}

fn run_registration(action: fn() -> std::io::Result<()>) {
    if let Err(error) = action() {
        eprintln!("Browser registration failed: {error}");
        std::process::exit(1);
    }
}
