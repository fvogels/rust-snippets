use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget}};
use crate::ui::{state::{Mode, State}, view_mode::ViewMode, widgets::{description_list, snippet_view::SnippetViewState, tags_view::{TagsView, TagsViewState}}};


pub(super) struct TagSearchMode {
    pub state: State,
    pub(super) tags_view_state: TagsViewState,
    pub(super) tag_page_size: Option<usize>,
}

impl TagSearchMode {
    fn handle_key_event(mut self, key_event: KeyEvent) -> Mode {
        match key_event.code {
            KeyCode::Up => {
                self.tags_view_state.select_previous();
                self.remain_in_tag_search_mode()
            },
            KeyCode::Down => {
                self.tags_view_state.select_next();
                self.remain_in_tag_search_mode()
            },
            KeyCode::Home => {
                self.tags_view_state.select_first();
                self.remain_in_tag_search_mode()
            },
            KeyCode::End => {
                self.tags_view_state.select_last();
                self.remain_in_tag_search_mode()
            },
            KeyCode::PageDown => {
                self.tags_view_state.select_page_down(self.tag_page_size.unwrap_or(10));
                self.remain_in_tag_search_mode()
            },
            KeyCode::PageUp => {
                self.tags_view_state.select_page_up(self.tag_page_size.unwrap_or(10));
                self.remain_in_tag_search_mode()
            },
            KeyCode::Esc => {
                self.state.tag_input = None;
                self.state.refresh();
                Mode::View(ViewMode::init(self.state, description_list::State::default(), SnippetViewState::new()))
            },
            KeyCode::Enter => {
                if let Some(index) = self.tags_view_state.selected() {
                    let selected_tag = &self.state.visible_tags[index];
                    self.state.select_tag(selected_tag.clone());
                    self.state.refresh();
                }
                else {
                    self.state.tag_input = None;
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

                self.remain_in_tag_search_mode()
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

                self.remain_in_tag_search_mode()
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

                self.remain_in_tag_search_mode()
            },
            _ => self.remain_in_tag_search_mode()
        }
    }

    fn remain_in_tag_search_mode(self) -> Mode {
        Mode::TagSearch(self)
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let descriptions = self.state.visible_snippet_descriptions();
        let description_list_view = description_list::Widget::new(descriptions, false);
        Widget::render(description_list_view, area, buffer);
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
        let (tag_list_area, snippet_list_area, bottom_line_area) = {
            let [upper_area, bottom_line_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
            let [tag_list_area, snippet_list_area] = Layout::horizontal([Constraint::Length(self.state.tag_view_width), Constraint::Fill(1)]).areas(upper_area);

            (tag_list_area, snippet_list_area, bottom_line_area)
        };

        self.tag_page_size = Some((tag_list_area.height - 2) as usize);

        self.render_snippet_list(snippet_list_area, buffer);
        self.render_input_field(bottom_line_area, buffer);
        self.render_tag_list(tag_list_area, buffer);
    }
}

fn valid_filter_character(c: char) -> bool {
    c.is_ascii_graphic()
}