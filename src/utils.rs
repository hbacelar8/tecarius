use crate::config::Colors;
use ratatui::{
    layout::Alignment,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders},
};

const SUFFIX: [&str; 9] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
const UNIT: f64 = 1024.0;

/// Convert raw bytes to human readable size.
pub fn to_human_bytes<T: Into<f64>>(bytes: T) -> String {
    let size = bytes.into();

    if size <= 0.0 {
        return "0 B".to_string();
    }

    let base = size.log10() / UNIT.log10();

    let result = format!("{:.1}", UNIT.powf(base - base.floor()),)
        .trim_end_matches(".0")
        .to_owned();

    [&result, SUFFIX[base.floor() as usize]].join(" ")
}

/// Create a block widget with optionals title and legend.
pub fn create_block(
    title: Option<String>,
    legend: Option<String>,
    colors: &Colors,
) -> Block<'static> {
    let mut block = Block::new()
        .bg(Color::from_u32(colors.ui.background))
        .fg(Color::from_u32(colors.ui.border))
        .title_style(Style::new().fg(Color::from_u32(colors.text.title)))
        .title_alignment(Alignment::Center)
        .bold()
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL);

    if let Some(title) = title {
        block = block.title(title);
    }

    if let Some(legend) = legend {
        block = block.title_bottom(legend);
    }

    block
}
