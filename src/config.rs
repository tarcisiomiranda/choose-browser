use crate::browser::Browser;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub browsers: BrowserPreferences,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrowserPreferences {
    #[serde(default)]
    pub show_only: Vec<String>,
    #[serde(default)]
    pub hidden: Vec<String>,
    #[serde(default)]
    pub order: Vec<String>,
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("choose-browser")
        .join("config.toml")
}

pub fn load_config() -> Result<AppConfig, String> {
    let path = config_path();

    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    toml::from_str(&contents)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

pub fn init_config_file(browsers: &[Browser]) -> Result<PathBuf, String> {
    let path = config_path();

    if path.exists() {
        return Ok(path);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create config directory {}: {error}",
                parent.display()
            )
        })?;
    }

    fs::write(&path, default_config_contents(browsers))
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;

    Ok(path)
}

pub fn apply_browser_config(mut browsers: Vec<Browser>, config: &AppConfig) -> Vec<Browser> {
    let show_only = normalize_rules(&config.browsers.show_only);
    let hidden = normalize_rules(&config.browsers.hidden);
    let order = normalize_rules(&config.browsers.order);

    if !show_only.is_empty() {
        browsers.retain(|browser| matches_rules(browser, &show_only));
    }

    if !hidden.is_empty() {
        browsers.retain(|browser| !matches_rules(browser, &hidden));
    }

    browsers.sort_by(|a, b| {
        browser_order_index(a, &order)
            .cmp(&browser_order_index(b, &order))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    browsers
}

fn default_config_contents(browsers: &[Browser]) -> String {
    let mut contents = String::from(
        "# Choose Browser configuration\n\
         # Use `choose-browser --list-all` to see the available browser IDs.\n\
         # You can reference either the desktop file ID or the visible browser name.\n\
         \n\
         [browsers]\n\
         # If empty, all detected browsers may appear.\n\
         show_only = []\n\
         \n\
         # Browsers that should not appear in the chooser.\n\
         hidden = []\n\
         \n\
         # Order shown in the chooser. Unlisted browsers stay after these.\n\
         order = []\n\
         \n\
         # Available browsers on this machine:\n",
    );

    for browser in browsers {
        contents.push_str(&format!(
            "# - {} = {}\n",
            browser.desktop_file, browser.name
        ));
    }

    contents
}

fn normalize_rules(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| normalize_value(value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn browser_order_index(browser: &Browser, order: &[String]) -> usize {
    order
        .iter()
        .position(|rule| browser_matches_rule(browser, rule))
        .unwrap_or(usize::MAX)
}

fn matches_rules(browser: &Browser, rules: &[String]) -> bool {
    rules.iter().any(|rule| browser_matches_rule(browser, rule))
}

fn browser_matches_rule(browser: &Browser, rule: &str) -> bool {
    let desktop_file = normalize_value(&browser.desktop_file);
    let name = normalize_value(&browser.name);

    rule == desktop_file || rule == name
}

fn normalize_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
