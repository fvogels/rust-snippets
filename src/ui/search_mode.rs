use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::{Color, Stylize}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}};
use crate::ui::{state::{Mode, State}, view_mode::ViewMode, widgets::{description_list, snippet_view::{SnippetView, SnippetViewState}, tags_view::TagsView}};


pub(super) struct SearchMode {
    pub state: State,
    pub(super) description_list_state: description_list::State,
    pub(super) snippet_view_state: SnippetViewState,
}

impl SearchMode {
    fn handle_key_event(mut self, key_event: KeyEvent) -> Mode {
        match key_event.code {
            KeyCode::Up => self.select_previous_description(),
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
                self.state.clear_keywords();
                self.state.refresh();
                self.description_list_state.select_first();
                Mode::View(ViewMode::init(self.state, self.description_list_state, self.snippet_view_state))
            },
            KeyCode::Enter => {
                Mode::View(ViewMode::init(self.state, self.description_list_state, self.snippet_view_state))
            },
            KeyCode::Backspace => {
                if let Some(mut last_keyword) = self.state.keywords.pop() {
                    if last_keyword.is_empty() {
                        self.state.keywords.pop();
                    }
                    else {
                        last_keyword.truncate(last_keyword.len() - 1);
                        self.state.keywords.push(last_keyword);
                    }

                    self.state.refresh();
                }
                Mode::Search(self)
            },
            KeyCode::Char(' ') => {
                match self.state.keywords.last() {
                    None => {
                        // No keywords entered, nothing needs to be done
                    },
                    Some(keyword) => {
                        if !keyword.is_empty() {
                            self.state.keywords.push(String::from(""));
                        }
                    }
                }

                Mode::Search(self)
            },
            KeyCode::Char(char) if valid_filter_character(char) => {
                let new_keyword = match self.state.pop_keyword() {
                    Some(keyword) => {
                        let mut keyword = keyword;
                        keyword.push(char);
                        keyword
                    },
                    None => {
                        String::from(char)
                    }
                };
                self.state.keywords.push(new_keyword);
                self.state.refresh();
                Mode::Search(self)
            },
            _ => Mode::Search(self)
        }
    }

    fn select_previous_description(mut self) -> Mode {
        self.description_list_state.select_previous();
        Mode::Search(self)
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let descriptions = self.state.visible_snippet_descriptions();
        let description_list_view = description_list::Widget::new(descriptions, true);
        StatefulWidget::render(description_list_view, area, buffer, &mut self.description_list_state);
    }

    fn render_selected_snippet(&mut self, area: Rect, buffer: &mut Buffer) {
        match self.description_list_state.selected() {
            None => {},
            Some(selected_snippet_index) => {
                let snippet = self.state.library.snippet(self.state.visible_snippets[selected_snippet_index]);
                let snippet_view = SnippetView::new(snippet);
                snippet_view.render(area, buffer, &mut self.snippet_view_state);
            }
        }
    }

    fn render_input_field(&mut self, area: Rect, buffer: &mut Buffer) {
        let keyword_spans = self.state.keywords.iter().filter(|keyword| !keyword.is_empty()).map(|keyword| {
            Span::default().content(format!(" {} ", keyword)).on_gray()
        });
        let separator_span = Span::default().content(" ");
        let spans = itertools::intersperse(keyword_spans, separator_span);
        let line = Line::default().spans(spans);
        let paragraph = Paragraph::new(line).bg(Color::DarkGray);

        paragraph.render(area, buffer);
    }

    fn render_tag_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let selected_tags = self.state.selected_tags.iter().map(String::as_str);
        let available_tags = self.state.visible_tags.iter().map(String::as_str);
        let tag_list = TagsView::new(selected_tags, available_tags);

        let block = Block::new().borders(Borders::all()).title("Tags");
        let tag_list_area = block.inner(area);

        block.render(area, buffer);
        Widget::render(tag_list, tag_list_area, buffer);
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
    c.is_ascii_graphic()
}
