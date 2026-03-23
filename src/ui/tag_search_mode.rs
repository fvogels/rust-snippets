use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::Style, text::Line, widgets::{Block, BorderType, Borders, List, ListState, Paragraph, StatefulWidget, Widget}};
use crate::ui::{state::{Mode, State}, view_mode::ViewMode, widgets::{description_list, snippet_view::{SnippetView, SnippetViewState}, tags_view::{TagsView, TagsViewState}}};


pub(super) struct TagSearchMode {
    pub state: State,
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
            KeyCode::Esc => {
                Mode::View(ViewMode::init(self.state, description_list::State::default(), SnippetViewState::new()))
            },
            KeyCode::Enter => {
                if let Some(index) = self.tags_view_state.selected() {
                    let selected_tag = &self.state.visible_tags[index];
                    self.state.select_tag(selected_tag.clone());
                    self.state.refresh();
                }

                Mode::View(ViewMode::init(self.state, description_list::State::default(), SnippetViewState::new()))
            },
            KeyCode::Char(' ') => {
                if let Some(index) = self.tags_view_state.selected() {
                    let selected_tag = &self.state.visible_tags[index];
                    self.state.select_tag(selected_tag.clone());
                    self.state.refresh();
                }

                Mode::TagSearch(self)
            },
            KeyCode::Backspace => {
                if let Some(mut s) = self.state.tag_input {
                    if s.len() > 1 {
                        s.truncate(s.len() - 1);
                        self.state.tag_input = Some(s);
                    }
                    else {
                        self.state.tag_input = None;
                    }

                    self.state.refresh();
                    self.tags_view_state.ensure_selection();
                }

                Mode::TagSearch(self)
            },
            KeyCode::Char(mut char) if valid_filter_character(char) => {
                char = char.to_ascii_lowercase();

                match self.state.tag_input {
                    Some(mut s) => {
                        s.push(char);
                        self.state.tag_input = Some(s);
                    }
                    None => {
                        self.state.tag_input = Some(String::from(char));
                    }
                }

                self.state.refresh();

                Mode::TagSearch(self)
            },
            _ => Mode::TagSearch(self)
        }
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let descriptions = self.state.visible_snippet_descriptions();
        let description_list_view = description_list::Widget::new(descriptions, false);
        Widget::render(description_list_view, area, buffer);
    }

    fn render_selected_snippet(&mut self, area: Rect, buffer: &mut Buffer) {
        let snippet_id = self.state.visible_snippets.get(0);

        if let Some(snippet_id) = snippet_id {
            let snippet = self.state.library.snippet(*snippet_id);
            let snippet_view = SnippetView::new(snippet);

            let mut snippet_view_state = SnippetViewState::new();
            snippet_view.render(area, buffer, &mut snippet_view_state);
        }
    }

    fn render_tag_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let selected_tags = self.state.selected_tags.iter().map(String::as_str);
        let available_tags = self.state.visible_tags.iter().map(String::as_str);
        let tag_list = TagsView::new(selected_tags, available_tags);

        let block = Block::new().borders(Borders::all()).border_type(BorderType::Double).title("Tags");
        let tag_list_area = block.inner(area);

        block.render(area, buffer);
        StatefulWidget::render(tag_list, tag_list_area, buffer, &mut self.tags_view_state);
    }

    fn render_input_field(&mut self, area: Rect, buffer: &mut Buffer) {
        let mut contents = String::from("#");
        if let Some(tag_input) = &self.state.tag_input {
            contents.push_str(tag_input.as_str());
        }
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
    c.is_ascii_graphic()
}