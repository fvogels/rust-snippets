use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::{Style}, text::{Line}, widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget}};
use crate::{snippets::Library, ui::{searchmode::SearchMode, state::Mode, syntax::SyntaxHighlighter, widgets::snippet_view::{SnippetView, SnippetViewState}}};


pub(super) struct ViewMode {
    pub(super) library: Box<Library>,
    pub(super) syntax_highlighter: Box<SyntaxHighlighter>,
    pub(super) snippet_list: Vec<usize>,
    pub(super) description_list_state: ListState,
    pub(super) snippet_view_state: SnippetViewState,
}

impl ViewMode {
    pub(super) fn new(library: Box<Library>, syntax_highlighter: Box<SyntaxHighlighter>) -> Self {
        ViewMode {
            snippet_list: library.snippet_indices().collect(),
            library: library,
            syntax_highlighter: syntax_highlighter,
            description_list_state: ListState::default().with_selected(Some(0)),
            snippet_view_state: SnippetViewState::new(),
        }
    }

    fn handle_key_event(mut self, key_event: KeyEvent) -> Mode {
        match key_event.code {
            KeyCode::Char('q') => {
                Mode::Terminated
            },
            KeyCode::Char('/') => {
                Mode::Search(SearchMode{
                    library: self.library,
                    syntax_highlighter: self.syntax_highlighter,
                    snippet_list: self.snippet_list,
                    description_list_state: self.description_list_state,
                    snippet_view_state: self.snippet_view_state,
                    filter: String::new(),
                })
            },
            KeyCode::Up => {
                self.description_list_state.select_previous();
                Mode::View(self)
            },
            KeyCode::Down => {
                self.description_list_state.select_next();
                Mode::View(self)
            },
            KeyCode::Tab => {
                self.snippet_view_state.select_next();
                Mode::View(self)
            },
            KeyCode::Esc => {
                self.snippet_list = self.library.snippet_indices().collect();
                Mode::View(self)
            },
            _ => Mode::View(self)
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

    pub fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_event(self, event: Event) -> Mode {
        match event {
            Event::Key(key_event) if key_event.is_press() => {
                self.handle_key_event(key_event)
            },
            _ => Mode::View(self),
        }
    }
}

impl Widget for &mut ViewMode {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [snippet_list_area, snippet_area] = Layout::vertical([Constraint::Length(15), Constraint::Fill(1)]).areas(area);

        self.render_snippet_list(snippet_list_area, buffer);
        self.render_selected_snippet(snippet_area, buffer);
    }
}
