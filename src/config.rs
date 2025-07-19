use crate::error::Result;
use cross_xdg::BaseDirs;
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

/// User configuration.
#[derive(Deserialize)]
pub struct UserConfig {
    theme: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            theme: "catppuccin-mocha".to_string(),
        }
    }
}

/// Colors configuration.
#[derive(Deserialize)]
pub struct Colors {
    pub ui: UI,
    pub text: Text,
    pub input: Input,
}

/// UI colors.
#[derive(Deserialize)]
pub struct UI {
    pub background: u32,
    pub border: u32,
    pub key: u32,
}

/// Text colors.
#[derive(Deserialize)]
pub struct Text {
    pub title: u32,
    pub text: u32,
}

/// Input colors.
#[derive(Deserialize)]
pub struct Input {
    pub typing: u32,
    pub normal: u32,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            ui: UI {
                background: 0x1e1e2e,
                border: 0x6c7086,
                key: 0x94e2d5,
            },
            text: Text {
                title: 0xa6e3a1,
                text: 0xf5e0dc,
            },
            input: Input {
                typing: 0x94e2d4,
                normal: 0x6c7086,
            },
        }
    }
}

/// Get the color configuration.
pub fn theme_colors() -> Result<Colors> {
    let user_config_path = BaseDirs::new()?.config_home().join("tecarius/config.toml");

    let user_config = fs::read_to_string(user_config_path)?;
    let user_config: UserConfig = toml::from_str(&user_config)?;

    let theme_path = &PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join(format!("themes/{}.toml", user_config.theme));

    let colors = fs::read_to_string(theme_path)?;
    let colors: Colors = toml::from_str(&colors)?;

    Ok(colors)
}
