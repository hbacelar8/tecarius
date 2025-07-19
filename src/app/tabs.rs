use super::create_block;
use crate::{config::Colors, pacman::PackageData};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::{Paragraph, Widget},
};
use strum_macros::{Display, EnumIter, FromRepr};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum DependenciesTabs {
    #[default]
    #[strum(to_string = "dependencies")]
    Deps,
    #[strum(to_string = "optional deps")]
    OptDeps,
    #[strum(to_string = "conflicts with")]
    Conflics,
    #[strum(to_string = "replaces")]
    Replaces,
}

impl DependenciesTabs {
    /// Render tabs.
    pub fn render(self, area: Rect, buf: &mut Buffer, package: &PackageData, colors: &Colors) {
        match self {
            Self::Deps => self.render_deps_tab(area, buf, package, colors),
            Self::OptDeps => self.render_opt_deps_tab(area, buf, package, colors),
            Self::Conflics => self.render_conflicts_tab(area, buf, package, colors),
            Self::Replaces => self.render_replaces_tab(area, buf, package, colors),
        }
    }

    /// Get previous tab.
    pub fn previous(self) -> Self {
        let current_index = self as usize;
        let previous_index = current_index.saturating_sub(1);

        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get next tab.
    pub fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);

        Self::from_repr(next_index).unwrap_or(self)
    }

    /// Get tab title as a Line.
    pub fn title(self) -> Line<'static> {
        format!(" {self} ").into()
    }

    fn render_deps_tab(self, area: Rect, buf: &mut Buffer, package: &PackageData, colors: &Colors) {
        let block = create_block(None, Some(" ⇄ (tab / shift+tab) ".to_string()), colors);

        let deps_lines: Vec<Line> = package
            .dependencies
            .iter()
            .map(|dep| Line::from(dep.to_string()))
            .collect();

        Paragraph::new(deps_lines)
            .block(block)
            .bg(Color::from_u32(colors.ui.background))
            .fg(Color::from_u32(colors.text.text))
            .render(area, buf);
    }

    fn render_opt_deps_tab(
        self,
        area: Rect,
        buf: &mut Buffer,
        package: &PackageData,
        colors: &Colors,
    ) {
        let block = create_block(None, Some(" ⇄ (tab / shift+tab) ".to_string()), colors);

        let opt_deps_lines: Vec<Line> = package
            .optional_dependencies
            .iter()
            .map(|dep| Line::from(dep.to_string()))
            .collect();

        Paragraph::new(opt_deps_lines)
            .block(block)
            .bg(Color::from_u32(colors.ui.background))
            .fg(Color::from_u32(colors.text.text))
            .render(area, buf);
    }

    fn render_conflicts_tab(
        self,
        area: Rect,
        buf: &mut Buffer,
        package: &PackageData,
        colors: &Colors,
    ) {
        let block = create_block(None, Some(" ⇄ (tab / shift+tab) ".to_string()), colors);

        let conflicts_lines: Vec<Line> = package
            .conflicts
            .iter()
            .map(|dep| Line::from(dep.to_string()))
            .collect();

        Paragraph::new(conflicts_lines)
            .block(block)
            .bg(Color::from_u32(colors.ui.background))
            .fg(Color::from_u32(colors.text.text))
            .render(area, buf);
    }

    fn render_replaces_tab(
        self,
        area: Rect,
        buf: &mut Buffer,
        package: &PackageData,
        colors: &Colors,
    ) {
        let block = create_block(None, Some(" ⇄ (tab / shift+tab) ".to_string()), colors);

        let replaces_lines: Vec<Line> = package
            .replaces
            .iter()
            .map(|dep| Line::from(dep.to_string()))
            .collect();

        Paragraph::new(replaces_lines)
            .block(block)
            .bg(Color::from_u32(colors.ui.background))
            .fg(Color::from_u32(colors.text.text))
            .render(area, buf);
    }
}
