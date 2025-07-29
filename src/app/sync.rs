use crate::{config::Colors, utils::create_block};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Stylize},
    text::Text,
    widgets::{Paragraph, Widget},
};

#[derive(Clone, Copy, Default)]
pub enum SyncStates {
    #[default]
    Confirmation,
    Syncing,
}

#[derive(Default)]
pub struct SyncWidget {
    state: SyncStates,
    vertical_scroll: i16,
}

impl SyncWidget {
    pub fn render<'a>(
        &self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        packages: impl IntoIterator<Item = &'a str>,
    ) {
        let [msg_area, log_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);

        self.render_msg_box(msg_area, buf, colors);
        self.render_log_box(log_area, buf, colors, packages);
    }

    pub fn area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);

        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        area
    }

    pub fn next(&mut self) {
        self.vertical_scroll = 1;
    }

    pub fn previous(&mut self) {
        self.vertical_scroll = -1;
    }

    pub fn start_sync(&mut self) {
        self.state = SyncStates::Syncing;
    }

    fn render_msg_box(&self, area: Rect, buf: &mut Buffer, colors: &Colors) {
        let block = create_block(None, None, colors);

        let message = match self.state {
            SyncStates::Confirmation => "Sync packages? [Enter/ESC]",
            SyncStates::Syncing => "Syncing",
        };

        Paragraph::new(message)
            .block(block)
            .centered()
            .bg(Color::from_u32(colors.ui.background))
            .fg(Color::from_u32(colors.text.text))
            .render(area, buf);
    }

    fn render_log_box<'a>(
        &self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        packages: impl IntoIterator<Item = &'a str>,
    ) {
        let block = create_block(None, None, colors);
        let packages = Text::from_iter(packages);
        let scroll = (packages.height() as u16).saturating_sub(area.height);
        let scroll = scroll.saturating_add_signed(self.vertical_scroll);

        Paragraph::new(packages)
            .block(block)
            .bg(Color::from_u32(colors.ui.background))
            .fg(Color::from_u32(colors.text.text))
            .scroll((scroll, 0))
            .render(area, buf);
    }
}
