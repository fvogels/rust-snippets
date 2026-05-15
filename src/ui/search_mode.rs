use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::{Color, Stylize}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}};
use crate::ui::{state::{Mode, SnippetLayout, State}, view_mode::ViewMode, widgets::{description_list, snippet_view::{SnippetView, SnippetViewState}, tags_view::TagsView}};


pub(super) struct SearchMode {
    pub state: State,
    pub(super) description_list_state: description_list::State,
    pub(super) snippet_view_state: SnippetViewState,
}

impl SearchMode {
    fn handle_key_event(self, key_event: KeyEvent) -> Mode {
        match key_event.code {
            KeyCode::Up => self.select_previous_description(),
            KeyCode::Down => self.select_next_description(),
            KeyCode::Tab => self.select_next_snippet_part(),
            KeyCode::BackTab => self.select_previous_snippet_part(),
            KeyCode::Esc => self.cancel_search(),
            KeyCode::Enter => self.switch_to_view_mode(),
            KeyCode::Backspace => self.remove_last_char_or_keyword(),
            KeyCode::Char(' ') => self.start_new_keyword(),
            KeyCode::Char(char) if valid_filter_character(char) => self.add_char_to_keyword(char),
            _ => self.remain_in_search_mode()
        }
    }

    fn select_previous_description(mut self) -> Mode {
        self.description_list_state.select_previous();
        self.remain_in_search_mode()
    }

    fn select_next_description(mut self) -> Mode {
        self.description_list_state.select_next();
        self.remain_in_search_mode()
    }

    fn select_next_snippet_part(mut self) -> Mode {
        self.snippet_view_state.select_next();
        self.remain_in_search_mode()
    }

    fn select_previous_snippet_part(mut self) -> Mode {
        self.snippet_view_state.select_previous();
        self.remain_in_search_mode()
    }

    fn cancel_search(mut self) -> Mode {
        self.state.clear_keywords();
        self.state.refresh();
        self.description_list_state.select_first();
        self.switch_to_view_mode()
    }

    // Removes the last char of the last keyword.
    // If the last keyword is empty, remove the entire keyword that precedes it.
    fn remove_last_char_or_keyword(mut self) -> Mode {
        if let Some(mut last_keyword) = self.state.keywords.pop() {
            if last_keyword.is_empty() {
                self.state.keywords.pop();

                // Without this, we would end up editing the last keyword instead
                // of starting a new one
                self.state.keywords.push("".to_owned());
            }
            else {
                last_keyword.truncate(last_keyword.len() - 1);
                self.state.keywords.push(last_keyword);
            }

            self.state.refresh();
        }

        self.remain_in_search_mode()
    }

    fn start_new_keyword(mut self) -> Mode {
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

        self.remain_in_search_mode()
    }

    fn add_char_to_keyword(mut self, char: char) -> Mode {
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

        self.remain_in_search_mode()
    }

    fn remain_in_search_mode(self) -> Mode {
        Mode::Search(self)
    }

    fn switch_to_view_mode(self) -> Mode {
        Mode::View(ViewMode::init(self.state, self.description_list_state, self.snippet_view_state))
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
                let snippet_view = SnippetView::new(snippet, &self.state.library);
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
        let (tag_list_area, snippet_list_area, snippet_area, bottom_line_area) = {
            let [upper_area, bottom_line_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
            let [tag_list_area, right_area] = Layout::horizontal([Constraint::Length(self.state.tag_view_width), Constraint::Fill(1)]).areas(upper_area);

            let (snippet_list_area, snippet_area) = {
                match self.state.snippet_layout {
                    SnippetLayout::OnlyList => {
                        (Some(right_area), None)
                    },
                    SnippetLayout::OnlySnippet => {
                        (None, Some(right_area))
                    },
                    SnippetLayout::Share(list_height) => {
                        let [snippet_list_area, snippet_area] = Layout::vertical([Constraint::Length(list_height), Constraint::Fill(1)]).areas(right_area);
                        (Some(snippet_list_area), Some(snippet_area))
                    }
                }
            };

            (tag_list_area, snippet_list_area, snippet_area, bottom_line_area)
        };

        if let Some(snippet_list_area) = snippet_list_area {
            self.render_snippet_list(snippet_list_area, buffer);
        }

        if let Some(snippet_area) = snippet_area {
            self.render_selected_snippet(snippet_area, buffer);
        }

        self.render_input_field(bottom_line_area, buffer);
        self.render_tag_list(tag_list_area, buffer);
    }
}

fn valid_filter_character(c: char) -> bool {
    c.is_ascii_graphic()
}
