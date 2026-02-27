use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::Style, text::{Line}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget}};
use crate::{snippets::Library, ui::{state::Mode, syntax::SyntaxHighlighter, viewmode::ViewMode, widgets::snippet_view::{SnippetView, SnippetViewState}}};


pub(super) struct SearchMode {
    pub(super) library: Box<Library>,
    pub(super) syntax_highlighter: Box<SyntaxHighlighter>,
    pub(super) snippet_list: Vec<usize>,
    pub(super) description_list_state: ListState,
    pub(super) snippet_view_state: SnippetViewState,
    pub(super) filter: String,
}

impl SearchMode {
    fn handle_key_event(mut self, key_event: KeyEvent) -> Mode {
        match key_event.code {
            KeyCode::Up => {
                self.description_list_state.select_previous();
                Mode::Search(self)
            },
            KeyCode::Down => {
                self.description_list_state.select_next();
                Mode::Search(self)
            },
            KeyCode::Tab => {
                self.snippet_view_state.select_next();
                Mode::Search(self)
            },
            KeyCode::BackTab => {
                self.snippet_view_state.select_previous();
                Mode::Search(self)
            },
            KeyCode::Esc => {
                let selected_nodes = self.library.snippet_indices().collect();
                self.description_list_state.select_first();

                Mode::View(ViewMode{
                    library: self.library,
                    syntax_highlighter: self.syntax_highlighter,
                    snippet_list: selected_nodes,
                    description_list_state: self.description_list_state,
                    snippet_view_state: self.snippet_view_state,
                })
            },
            KeyCode::Enter => {
                Mode::View(ViewMode{
                    library: self.library,
                    syntax_highlighter: self.syntax_highlighter,
                    snippet_list: self.snippet_list,
                    description_list_state: self.description_list_state,
                    snippet_view_state: self.snippet_view_state,
                })
            },
            KeyCode::Backspace => {
                if self.filter.len() > 0 {
                    self.filter.truncate(self.filter.len() - 1);
                    self.filter_snippets();
                    self.ensure_snippet_selection();
                }
                Mode::Search(self)
            },
            KeyCode::Char(char) if valid_filter_character(char) => {
                self.filter.push(char.to_ascii_lowercase());
                self.filter_snippets();
                self.ensure_snippet_selection();
                Mode::Search(self)
            },
            _ => Mode::Search(self)
        }
    }

    fn filter_snippets(&mut self) {
        let keywords = self.filter.split(' ');
        let filtered_snippets = self.library.search(keywords);

        self.snippet_list = filtered_snippets;
    }

    fn ensure_snippet_selection(&mut self) {
        if !self.snippet_list.is_empty() {
            match self.description_list_state.selected() {
                Some(_) => {}
                None => self.description_list_state.select_first(),
            }
        }
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        let descriptions = self.snippet_list.iter().copied().map(|index| ListItem::new(self.library.snippet(index).description.as_str()) );
        let list_block = Block::new().title(Line::raw("Snippets")).borders(Borders::ALL).title_bottom(Line::raw(format!("{} snippets", descriptions.len())).right_aligned());
        let list = List::new(descriptions).highlight_style(highlight_style).block(list_block);

        StatefulWidget::render(list, area, buffer, &mut self.description_list_state);
    }

    fn render_selected_snippet(&mut self, area: Rect, buffer: &mut Buffer) {
        match self.description_list_state.selected() {
            None => {},
            Some(selected_snippet_index) => {
                let snippet = self.library.snippet(selected_snippet_index);
                let snippet_view = SnippetView::new(snippet, &self.syntax_highlighter);
                snippet_view.render(area, buffer, &mut self.snippet_view_state);
            }
        }
    }

    fn render_input_field(&mut self, area: Rect, buffer: &mut Buffer) {
        let mut contents = String::from("> ");
        contents.push_str(&self.filter);
        Paragraph::new(contents).render(area, buffer);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_event(self, event: Event) -> Mode {
        match event {
            Event::Key(key_event) if key_event.is_press() => self.handle_key_event(key_event),
            _ => Mode::Search(self),
        }
    }
}

impl Widget for &mut SearchMode {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [snippet_list_area, snippet_area, input_area] = Layout::vertical([Constraint::Length(15), Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        self.render_snippet_list(snippet_list_area, buffer);
        self.render_selected_snippet(snippet_area, buffer);
        self.render_input_field(input_area, buffer);
    }
}

fn valid_filter_character(c: char) -> bool {
    c.is_ascii_graphic() || c == ' '
}