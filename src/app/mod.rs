use crate::{config::Colors, error, pacman::Pacman, utils::create_block};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{
        Clear, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, StatefulWidget,
        Tabs, Widget,
    },
};
use std::{io::Read, ops::Not, process::ChildStdout};
use strum::IntoEnumIterator;
use tabs::DependenciesTabs;
use tui_input::{Input, backend::crossterm::EventHandler};

mod tabs;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum State {
    Exiting,
    #[default]
    Normal,
    Typing,
    Upgrading,
}

pub struct App {
    state: State,
    filter_upgradables: bool,
    colors: Colors,
    pacman: Pacman,
    list_state: ListState,
    dependencies_tabs: DependenciesTabs,
    input: Input,
    search_matcher: SkimMatcherV2,
    selected_packages: Vec<String>,
    stdout: Option<ChildStdout>,
}

impl App {
    pub fn new(pacman: Pacman, colors: Colors) -> Self {
        Self {
            state: Default::default(),
            filter_upgradables: false,
            colors,
            pacman,
            list_state: Default::default(),
            dependencies_tabs: Default::default(),
            input: Default::default(),
            search_matcher: Default::default(),
            selected_packages: Vec::new(),
            stdout: None,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> error::Result<()> {
        self.list_state.select_first();

        while self.state != State::Exiting {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event(event::read()?);
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

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

        self.render_header(header_area, frame.buffer_mut());
        self.render_list(list_area, frame.buffer_mut());
        self.render_input(input_area, frame.buffer_mut());
        self.render_general_info(general_info_area, frame.buffer_mut());
        self.render_tabs(tabs_header_area, tabs_inner_area, frame.buffer_mut());

        if self.state == State::Upgrading {
            let popup_area = popup_area(area, 70, 70);
            frame.render_widget(Clear, popup_area);
            self.render_popup(popup_area, frame.buffer_mut());
        }
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }

            match self.state {
                State::Normal => match (key.modifiers, key.code) {
                    (_, KeyCode::Char('j')) | (_, KeyCode::Down) => self.list_state.select_next(),
                    (_, KeyCode::Char('k')) | (_, KeyCode::Up) => self.list_state.select_previous(),
                    (_, KeyCode::Char('g')) | (_, KeyCode::Home) => self.list_state.select_first(),
                    (_, KeyCode::Char('G')) | (_, KeyCode::End) => self.list_state.select_last(),
                    (KeyModifiers::CONTROL, KeyCode::Char('u')) => self.jump_up(),
                    (KeyModifiers::CONTROL, KeyCode::Char('d')) => self.jump_down(),
                    (KeyModifiers::ALT, KeyCode::Char('u')) => {
                        self.filter_upgradables = self.filter_upgradables.not();
                    }
                    (KeyModifiers::SHIFT, KeyCode::Char('S')) => self.upgrade_packages(),
                    (_, KeyCode::Char('x')) => self.toggle_package_selection(),
                    (_, KeyCode::Char('q')) => self.state = State::Exiting,
                    (_, KeyCode::Char('/')) => self.state = State::Typing,
                    (_, KeyCode::Tab) => self.next_tab(),
                    (_, KeyCode::BackTab) => self.previous_tab(),
                    (_, KeyCode::Esc) => {
                        if self.input.value() != "" {
                            self.input.reset();
                        } else {
                            self.state = State::Exiting
                        }
                    }
                    _ => (),
                },
                State::Typing => match key.code {
                    KeyCode::Enter => self.state = State::Normal,
                    KeyCode::Esc => self.flush_editing(),
                    _ => {
                        self.input.handle_event(&event);
                    }
                },
                State::Upgrading => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.state = State::Normal,
                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn jump_up(&mut self) {
        self.list_state.scroll_up_by(25);
    }

    fn jump_down(&mut self) {
        self.list_state.scroll_down_by(25);
    }

    fn flush_editing(&mut self) {
        self.state = State::Normal;
        self.input.reset();
    }

    fn next_tab(&mut self) {
        self.dependencies_tabs = self.dependencies_tabs.next();
    }

    fn previous_tab(&mut self) {
        self.dependencies_tabs = self.dependencies_tabs.previous();
    }

    fn upgrade_packages(&mut self) {
        self.state = State::Upgrading;
        self.stdout = self.pacman.upgrade(&self.selected_packages).ok();
    }

    fn toggle_package_selection(&mut self) {
        if let Some(selected_index) = self.list_state.selected() {
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
                        } else if !self.selected_packages.is_empty() {
                            Some(ListItem::from(format!("  {}  ", pkg.name)))
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
                    self.search_matcher
                        .fuzzy_match(pkg.name, self.input.value())
                        .map(|_| {
                            match (
                                pkg.new_version.is_some(),
                                self.selected_packages.contains(&pkg.name.to_string()),
                                self.selected_packages.is_empty(),
                            ) {
                                (true, true, _) => ListItem::from(format!("  {}  ", pkg.name)),
                                (true, false, true) => ListItem::from(format!("{}  ", pkg.name)),
                                (true, false, false) => {
                                    ListItem::from(format!("  {}  ", pkg.name))
                                }
                                (false, true, _) => ListItem::from(format!("  {}", pkg.name)),
                                (false, false, false) => ListItem::from(format!("  {}", pkg.name)),
                                (false, false, true) => ListItem::from(pkg.name),
                            }
                        })
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

        StatefulWidget::render(name_list, area, buf, &mut self.list_state);
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let block = create_block(None, Some(" search (/) ".to_string()), &self.colors)
            .padding(Padding::horizontal(3));
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let block = match self.state {
            State::Typing => block.border_style(Color::from_u32(self.colors.ui.key)),
            _ => block.border_style(Color::from_u32(self.colors.ui.border)),
        };
        let style = match self.state {
            State::Typing => Color::from_u32(self.colors.input.typing),
            _ => Color::from_u32(self.colors.input.normal),
        };

        Paragraph::new(self.input.value())
            .block(block)
            .scroll((0, scroll as u16))
            .style(style)
            .render(area, buf);
    }

    fn render_general_info(&mut self, area: Rect, buf: &mut Buffer) {
        let block = create_block(Some(" package info  ".to_string()), None, &self.colors);

        if let Some(selected_index) = self.list_state.selected() {
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

        if let Some(selected_index) = self.list_state.selected() {
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

    fn render_popup(&mut self, area: Rect, buf: &mut Buffer) {
        let block = create_block(Some(" upgrading ".to_string()), None, &self.colors);
        let mut st = String::new();
        if let Some(stdout) = self.stdout.as_mut() {
            stdout.read_to_string(&mut st).unwrap();
        }

        Paragraph::new(st)
            .block(block)
            .bg(Color::from_u32(self.colors.ui.background))
            .fg(Color::from_u32(self.colors.text.text))
            .render(area, buf);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
