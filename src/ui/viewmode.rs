use ratatui::{Frame, buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::{Style}, text::{Line}, widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget}};
use crate::{snippets::Library, ui::{searchmode::SearchMode, state::Mode, syntax::SyntaxHighlighter}};


pub(super) struct ViewMode {
    library: Box<Library>,
    syntax_highlighter: Box<SyntaxHighlighter>,
    snippet_list: Vec<usize>,
    description_list_state: ListState,
    selected_snippet_part: usize,
}

impl ViewMode {
    pub(super) fn new(library: Box<Library>, syntax_highlighter: Box<SyntaxHighlighter>) -> Self {
        ViewMode {
            snippet_list: library.snippet_indices().collect(),
            library: library,
            syntax_highlighter: syntax_highlighter,
            description_list_state: ListState::default().with_selected(Some(0)),
            selected_snippet_part: 0,
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
                    selected_snippet_part: self.selected_snippet_part,
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
                self.selected_snippet_part += 1;
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

                if self.selected_snippet_part >= snippet.parts.len() {
                    self.selected_snippet_part = 0;
                }

                let snippet_part = &snippet.parts[self.selected_snippet_part];

                let snippet_caption = match snippet_part.attributes.get("caption") {
                    Some(caption) => format!(" {}/{} {} ", self.selected_snippet_part + 1, snippet.parts.len(), caption),
                    None => format!(" {}/{} ", self.selected_snippet_part + 1, snippet.parts.len()),
                };

                let lines: Vec<&str> = snippet.parts[self.selected_snippet_part].lines.iter().map(|line| line.as_str()).collect();
                let snippet_caption_block = Block::new().title_bottom(Line::raw(snippet_caption)).borders(Borders::ALL);
                let paragraph = self.syntax_highlighter.highlight_lines("Go", lines.into_iter()).unwrap().block(snippet_caption_block);
                paragraph.render(area, buffer)
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
