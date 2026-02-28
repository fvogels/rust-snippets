use std::borrow::Borrow;

use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::Style, text::Line, widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget}};
use crate::{snippets::Library, ui::{SearchParameters, state::Mode, syntax::SyntaxHighlighter, view_mode::ViewMode, widgets::{snippet_view::{SnippetView, SnippetViewState}, tags_view::{TagsView, TagsViewState}, tree_view::TreeViewState}}};


pub(super) struct TagSearchMode {
    pub(super) library: Box<Library>,
    pub(super) syntax_highlighter: Box<SyntaxHighlighter>,
    pub(super) snippet_list: Vec<usize>,
    pub(super) description_list_state: ListState,
    pub(super) snippet_view_state: SnippetViewState,
    pub(super) search_parameters: SearchParameters,
    pub(super) tag_input: String,
    pub(super) tags_view_state: TagsViewState,
}

impl TagSearchMode {
    fn handle_key_event(mut self, key_event: KeyEvent) -> Mode {
        match key_event.code {
            KeyCode::Up => {
                self.tags_view_state.select_previous();
                Mode::TagSearch(self)
            },
            KeyCode::Down => {
                self.tags_view_state.select_next();
                Mode::TagSearch(self)
            },
            KeyCode::Tab => {
                self.snippet_view_state.select_next();
                Mode::TagSearch(self)
            },
            KeyCode::BackTab => {
                self.snippet_view_state.select_previous();
                Mode::TagSearch(self)
            },
            KeyCode::Esc => {
                let selected_nodes = self.library.snippets().collect();
                self.description_list_state.select_first();

                Mode::View(ViewMode{
                    library: self.library,
                    syntax_highlighter: self.syntax_highlighter,
                    snippet_list: selected_nodes,
                    description_list_state: self.description_list_state,
                    snippet_view_state: self.snippet_view_state,
                    search_parameters: self.search_parameters,
                })
            },
            KeyCode::Enter => {
                Mode::View(ViewMode{
                    library: self.library,
                    syntax_highlighter: self.syntax_highlighter,
                    snippet_list: self.snippet_list,
                    description_list_state: self.description_list_state,
                    snippet_view_state: self.snippet_view_state,
                    search_parameters: self.search_parameters,
                })
            },
            KeyCode::Backspace => {
                if self.tag_input.len() > 0 {
                    self.tag_input.truncate(self.tag_input.len() - 1);
                    self.filter_snippets();
                    self.ensure_snippet_selection();
                }
                Mode::TagSearch(self)
            },
            KeyCode::Char(char) if valid_filter_character(char) => {
                self.tag_input.push(char.to_ascii_lowercase());
                self.filter_snippets();
                self.ensure_snippet_selection();
                Mode::TagSearch(self)
            },
            _ => Mode::TagSearch(self)
        }
    }

    fn filter_snippets(&mut self) {
        let keywords = self.tag_input.split(' ');
        let filtered_snippets = self.library.search(keywords, self.search_parameters.tags.iter().map(|s| s.as_str()));

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

    fn render_tag_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let selected_tags = &self.search_parameters.tags;
        let available_tags = self.library.tags();

        let tag_view = TagsView::new(selected_tags.iter().map(Borrow::borrow), available_tags.iter().map(Borrow::borrow));
        let block = Block::new().borders(Borders::ALL).border_type(BorderType::Double);

        let block_inner_area = block.inner(area);
        block.render(area, buffer);
        tag_view.render(block_inner_area, buffer, &mut self.tags_view_state);
    }

    fn render_input_field(&mut self, area: Rect, buffer: &mut Buffer) {
        let mut contents = String::from("> ");
        contents.push_str(&self.tag_input);
        Paragraph::new(contents).render(area, buffer);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_event(self, event: Event) -> Mode {
        match event {
            Event::Key(key_event) if key_event.is_press() => self.handle_key_event(key_event),
            _ => Mode::TagSearch(self),
        }
    }
}

impl Widget for &mut TagSearchMode {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [upper_area, input_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        let [tag_list_area, right_area] = Layout::horizontal([Constraint::Length(40), Constraint::Fill(1)]).areas(upper_area);
        let [snippet_list_area, snippet_area] = Layout::vertical([Constraint::Length(15), Constraint::Fill(1)]).areas(right_area);

        self.render_snippet_list(snippet_list_area, buffer);
        self.render_selected_snippet(snippet_area, buffer);
        self.render_input_field(input_area, buffer);
        self.render_tag_list(tag_list_area, buffer);
    }
}

fn valid_filter_character(c: char) -> bool {
    c.is_ascii_graphic() || c == ' '
}