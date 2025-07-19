use crate::{config::Colors, error, pacman::Pacman, utils::create_block};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{
        HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, StatefulWidget, Tabs,
        Widget,
    },
};
use std::ops::Not;
use strum::IntoEnumIterator;
use tabs::DependenciesTabs;
use tui_input::{Input, backend::crossterm::EventHandler};

mod tabs;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

pub struct App {
    exit: bool,
    filter_upgradables: bool,
    colors: Colors,
    pacman: Pacman,
    state: ListState,
    dependencies_tabs: DependenciesTabs,
    input: Input,
    input_mode: InputMode,
    search_matcher: SkimMatcherV2,
    selected_packages: Vec<String>,
}

impl App {
    pub fn new(pacman: Pacman, colors: Colors) -> Self {
        Self {
            exit: false,
            filter_upgradables: false,
            colors,
            pacman,
            state: Default::default(),
            dependencies_tabs: Default::default(),
            input: Default::default(),
            input_mode: Default::default(),
            search_matcher: Default::default(),
            selected_packages: Vec::new(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> error::Result<()> {
        self.state.select_first();

        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_event(event::read()?);
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }

            match self.input_mode {
                InputMode::Normal => match (key.modifiers, key.code) {
                    (_, KeyCode::Char('j')) | (_, KeyCode::Down) => self.state.select_next(),
                    (_, KeyCode::Char('k')) | (_, KeyCode::Up) => self.state.select_previous(),
                    (_, KeyCode::Char('g')) | (_, KeyCode::Home) => self.state.select_first(),
                    (_, KeyCode::Char('G')) | (_, KeyCode::End) => self.state.select_last(),
                    (KeyModifiers::CONTROL, KeyCode::Char('u')) => self.jump_up(),
                    (KeyModifiers::CONTROL, KeyCode::Char('d')) => self.jump_down(),
                    (KeyModifiers::ALT, KeyCode::Char('u')) => {
                        self.filter_upgradables = self.filter_upgradables.not();
                    }
                    (_, KeyCode::Char('x')) => self.toggle_package_selection(),
                    (_, KeyCode::Char('q')) => self.exit = true,
                    (_, KeyCode::Char('/')) => self.start_editing(),
                    (_, KeyCode::Tab) => self.next_tab(),
                    (_, KeyCode::BackTab) => self.previous_tab(),
                    (_, KeyCode::Esc) => {
                        if self.input.value() != "" {
                            self.input.reset();
                        } else {
                            self.exit = true
                        }
                    }
                    _ => (),
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => self.stop_editing(),
                    KeyCode::Esc => self.flush_editing(),
                    _ => {
                        self.input.handle_event(&event);
                    }
                },
            }
        }
    }

    fn jump_up(&mut self) {
        self.state.scroll_up_by(25);
    }

    fn jump_down(&mut self) {
        self.state.scroll_down_by(25);
    }

    fn start_editing(&mut self) {
        self.input_mode = InputMode::Editing;
    }

    fn stop_editing(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    fn flush_editing(&mut self) {
        self.stop_editing();
        self.input.reset();
    }

    fn toggle_package_selection(&mut self) {
        if let Some(selected_index) = self.state.selected() {
            let package_name = self
                .pacman
                .packages()
                .filter(|pkg| {
                    let search = self
                        .search_matcher
                        .fuzzy_match(pkg.name, self.input.value())
                        .is_some();
                    let filter = if self.filter_upgradables {
                        pkg.new_version.is_some()
                    } else {
                        true
                    };

                    search && filter
                })
                .enumerate()
                .find(|(index, _)| *index == selected_index)
                .map(|(_, pkg)| pkg.name.to_string())
                .unwrap();

            if self.selected_packages.contains(&package_name) {
                self.selected_packages.retain(|name| *name != package_name)
            } else {
                self.selected_packages.push(package_name);
            }
        }
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let block = create_block(None, None, &self.colors);

        Paragraph::new("Tecarius - Pacman Librarian 󱉟 ")
            .block(block)
            .bg(Color::from_u32(self.colors.ui.background))
            .fg(Color::from_u32(self.colors.text.title))
            .bold()
            .italic()
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let total_packages = self.pacman.packages().count();

        let packages_names: Vec<ListItem> = match self.filter_upgradables {
            true => self
                .pacman
                .packages()
                .filter_map(|pkg| {
                    if self
                        .search_matcher
                        .fuzzy_match(pkg.name, self.input.value())
                        .is_some()
                        && pkg.new_version.is_some()
                    {
                        if self.selected_packages.contains(&pkg.name.to_string()) {
                            Some(ListItem::from(format!("  {}  ", pkg.name)))
                        } else {
                            Some(ListItem::from(format!("{}  ", pkg.name)))
                        }
                    } else {
                        None
                    }
                })
                .collect(),
            false => self
                .pacman
                .packages()
                .filter_map(|pkg| {
                    if self
                        .search_matcher
                        .fuzzy_match(pkg.name, self.input.value())
                        .is_some()
                    {
                        match (
                            pkg.new_version.is_some(),
                            self.selected_packages.contains(&pkg.name.to_string()),
                        ) {
                            (true, true) => Some(ListItem::from(format!("  {}  ", pkg.name))),
                            (true, false) => Some(ListItem::from(format!("{}  ", pkg.name))),
                            (false, true) => Some(ListItem::from(format!("  {}", pkg.name))),
                            (false, false) => Some(ListItem::from(pkg.name)),
                        }
                    } else {
                        None
                    }
                })
                .collect(),
        };

        let upgradable_count = self
            .pacman
            .packages()
            .filter(|pkg| pkg.new_version.is_some())
            .count();

        let block = create_block(
            Some(format!(
                " packages   ({} 󰏖  {}  ) ",
                total_packages, upgradable_count
            )),
            Some("↑↓ (k/j) (g/G) (c-d/c-u) | filter (alt+u)".to_string()),
            &self.colors,
        );

        let name_list = List::new(packages_names)
            .block(block)
            .bg(Color::from_u32(self.colors.ui.background))
            .fg(Color::from_u32(self.colors.text.text))
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(" → ")
            .highlight_style(Style::new().fg(Color::from_u32(self.colors.ui.key)));

        StatefulWidget::render(name_list, area, buf, &mut self.state);
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let block = create_block(None, Some(" search (/) ".to_string()), &self.colors)
            .padding(Padding::horizontal(3));
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let block = match self.input_mode {
            InputMode::Normal => block.border_style(Color::from_u32(self.colors.ui.border)),
            InputMode::Editing => block.border_style(Color::from_u32(self.colors.ui.key)),
        };
        let style = match self.input_mode {
            InputMode::Normal => Color::from_u32(self.colors.input.normal),
            InputMode::Editing => Color::from_u32(self.colors.input.editing),
        };

        Paragraph::new(self.input.value())
            .block(block)
            .scroll((0, scroll as u16))
            .style(style)
            .render(area, buf);
    }

    fn render_general_info(&mut self, area: Rect, buf: &mut Buffer) {
        let block = create_block(Some(" package info  ".to_string()), None, &self.colors);

        if let Some(selected_index) = self.state.selected() {
            let package = self
                .pacman
                .packages()
                .filter(|pkg| {
                    let search = self
                        .search_matcher
                        .fuzzy_match(pkg.name, self.input.value())
                        .is_some();
                    let filter = if self.filter_upgradables {
                        pkg.new_version.is_some()
                    } else {
                        true
                    };

                    search && filter
                })
                .enumerate()
                .find(|(index, _)| *index == selected_index)
                .unwrap()
                .1;
            let color = Color::from_u32(self.colors.ui.key);
            let mut lines: Vec<Line> = Vec::new();

            lines.push(Line::from(vec!["Name: ".fg(color), package.name.into()]));
            lines.push(Line::from(vec![
                "Version: ".fg(color),
                package.version.to_string().into(),
            ]));
            if let Some(desc) = package.description {
                lines.push(Line::from(vec!["Description: ".fg(color), desc.into()]));
            }
            if let Some(arch) = package.architecture {
                lines.push(Line::from(vec!["Architecture: ".fg(color), arch.into()]));
            }
            if let Some(url) = package.url {
                lines.push(Line::from(vec!["Url: ".fg(color), url.into()]));
            }
            lines.push(Line::from(vec![
                "Size: ".fg(color),
                package.size.to_string().into(),
            ]));

            if let Some(updated_at) = package.install_date {
                lines.push(Line::from(vec![
                    "Updated at: ".fg(color),
                    updated_at.format("%a %d %h %Y %H:%M:%S").to_string().into(),
                ]));
            }

            if let Some(new_version) = package.new_version {
                lines.push(Line::from(vec![
                    "New version available: ".fg(color),
                    package.version.to_string().into(),
                    " → ".into(),
                    new_version.to_string().into(),
                ]));
            }

            Paragraph::new(lines)
                .block(block)
                .bg(Color::from_u32(self.colors.ui.background))
                .fg(Color::from_u32(self.colors.text.text))
                .render(area, buf);
        } else {
            block.render(area, buf);
        }
    }

    fn render_tabs(&self, header_area: Rect, inner_area: Rect, buf: &mut Buffer) {
        let titles = DependenciesTabs::iter().map(DependenciesTabs::title);

        Tabs::new(titles)
            .select(self.dependencies_tabs as usize)
            .bg(Color::from_u32(self.colors.ui.background))
            .fg(Color::from_u32(self.colors.text.title))
            .bold()
            .italic()
            .divider("")
            .render(header_area, buf);

        if let Some(selected_index) = self.state.selected() {
            let package = self
                .pacman
                .packages()
                .enumerate()
                .find(|(index, _)| *index == selected_index)
                .unwrap()
                .1;

            self.dependencies_tabs
                .render(inner_area, buf, &package, &self.colors);
        }
    }

    fn next_tab(&mut self) {
        self.dependencies_tabs = self.dependencies_tabs.next();
    }

    fn previous_tab(&mut self) {
        self.dependencies_tabs = self.dependencies_tabs.previous();
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [header_area, main_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
                .margin(1)
                .areas(area);

        let [left_area, info_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(main_area);

        let [list_area, input_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(left_area);

        let [general_info_area, dependencies_area] =
            Layout::vertical([Constraint::Min(10), Constraint::Fill(2)]).areas(info_area);

        let [tabs_header_area, tabs_inner_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(dependencies_area);

        self.render_header(header_area, buf);
        self.render_list(list_area, buf);
        self.render_input(input_area, buf);
        self.render_general_info(general_info_area, buf);
        self.render_tabs(tabs_header_area, tabs_inner_area, buf);
    }
}
