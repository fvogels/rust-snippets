use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, widgets::{Block, Borders, StatefulWidget, Widget}};
use crate::{snippets::Library, ui::{search_mode::SearchMode, state::{Mode, SnippetLayout, State}, tag_search_mode::TagSearchMode, widgets::{description_list, keybindings_view::{Binding, KeybindingsView}, snippet_view::{SnippetView, SnippetViewState}, tags_view::{TagsView, TagsViewState}}}};


pub(super) struct ViewMode {
    pub state: State,
    pub(super) description_list_state: description_list::State,
    pub(super) snippet_view_state: SnippetViewState,
    description_list_page_size: u16, // Amount to jump when pressing page up/page down
}

impl ViewMode {
    pub(super) fn new(library: Library) -> Self {
        ViewMode {
            state: State::new(library),
            description_list_state: description_list::State::default(),
            snippet_view_state: SnippetViewState::new(),
            description_list_page_size: 10,
        }
    }

    pub(super) fn init(state: State, description_list_state: description_list::State, snippet_view_state: SnippetViewState) -> Self {
        ViewMode {
            state,
            description_list_state,
            snippet_view_state,
            description_list_page_size: 10,
        }
    }

    fn handle_key_event(mut self, key_event: KeyEvent) -> Mode {
        match key_event.code {
            KeyCode::Char('q') => {
                Mode::Terminated
            },
            KeyCode::Char('/') => {
                Mode::Search(SearchMode { state: self.state, description_list_state: self.description_list_state, snippet_view_state: self.snippet_view_state })
            },
            KeyCode::Char('#') => {
                Mode::TagSearch(TagSearchMode {
                    state: self.state,
                    tags_view_state: TagsViewState::new(),
                    tag_page_size: None,
                })
            },
            KeyCode::Up => {
                let previously_selected = self.description_list_state.selected();
                self.description_list_state.select_previous();
                if self.description_list_state.selected() != previously_selected {
                    self.snippet_view_state.select_first();
                }

                self.remain_in_view_mode()
            },
            KeyCode::Down => {
                let previously_selected = self.description_list_state.selected();
                self.description_list_state.select_next();
                if self.description_list_state.selected() != previously_selected {
                    self.snippet_view_state.select_first();
                }

                self.remain_in_view_mode()
            },
            KeyCode::PageUp => {
                let previously_selected = self.description_list_state.selected();
                self.description_list_state.scroll_up_by(self.description_list_page_size);
                if self.description_list_state.selected() != previously_selected {
                    self.snippet_view_state.select_first();
                }

                self.remain_in_view_mode()
            },
            KeyCode::PageDown => {
                let previously_selected = self.description_list_state.selected();
                self.description_list_state.scroll_down_by(self.description_list_page_size);
                if self.description_list_state.selected() != previously_selected {
                    self.snippet_view_state.select_first();
                }

                self.remain_in_view_mode()
            },
            KeyCode::Home => {
                let previously_selected = self.description_list_state.selected();
                self.description_list_state.select_first();
                if self.description_list_state.selected() != previously_selected {
                    self.snippet_view_state.select_first();
                }

                self.remain_in_view_mode()
            },
            KeyCode::End => {
                let previously_selected = self.description_list_state.selected();
                self.description_list_state.select_last();
                if self.description_list_state.selected() != previously_selected {
                    self.snippet_view_state.select_first();
                }

                self.remain_in_view_mode()
            },
            KeyCode::Tab => {
                self.snippet_view_state.select_next();
                self.remain_in_view_mode()
            },
            KeyCode::BackTab => {
                self.snippet_view_state.select_previous();
                self.remain_in_view_mode()
            },
            KeyCode::Char('?') => {
                self.state.clear_keywords();
                self.description_list_state.select_first();
                self.state.refresh();
                self.remain_in_view_mode()
            },
            KeyCode::Delete => {
                self.state.pop_selected_tag();
                self.state.refresh();
                self.remain_in_view_mode()
            },
            KeyCode::Char('+') => self.increase_snippet_list_size(),
            KeyCode::Char('-') => self.decrease_snippet_list_size(),
            KeyCode::Char('*') => self.toggle_maximize_snippet_list(),
            KeyCode::Char('.') => self.toggle_maximize_snippet_view(),
            KeyCode::Char(digit) if digit.is_ascii_digit() => {
                if let Some(index) = self.description_list_state.selected() {
                    let snippet_id = &self.state.visible_snippets[index];
                    let snippet = self.state.library.snippet(*snippet_id);
                    if let Some(one_based_index) = digit.to_digit(10) {
                        let index = one_based_index - 1;
                        if let Some(part) = snippet.parts.get(self.snippet_view_state.selected()) {
                            if let Some(source_code) = part.find_code_block_with_index(index as usize) {
                                if let Err(error) = cli_clipboard::set_contents(source_code.to_owned()) {
                                    panic!("failed to copy snippet to clipboard: {}", error)
                                }
                            }
                        }
                    }
                }
                self.remain_in_view_mode()
            },
            _ => self.remain_in_view_mode()
        }
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let descriptions = self.state.visible_snippet_descriptions();
        let description_list_view = description_list::Widget::new(descriptions, false);
        StatefulWidget::render(description_list_view, area, buffer, &mut self.description_list_state);

        if area.height >= 2 {
            self.description_list_page_size = area.height - 2;
        }
        else {
            self.description_list_page_size = 1;
        }
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

    fn render_tag_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let selected_tags = self.state.selected_tags.iter().map(String::as_str);
        let available_tags = self.state.visible_tags.iter().map(String::as_str);
        let tag_list = TagsView::new(selected_tags, available_tags);

        let block = Block::new().borders(Borders::all()).title("Tags");
        let tag_list_area = block.inner(area);

        block.render(area, buffer);
        Widget::render(tag_list, tag_list_area, buffer);
    }

    fn render_keybindings(&self, area: Rect, buffer: &mut Buffer) {
        let bindings = vec![
            Binding::new("q", "quit"),
            Binding::new("#", "add tag"),
            Binding::new("del", "pop tag"),
            Binding::new("/", "search"),
            Binding::new("?", "reset search"),
        ];
        let keybindings_view = KeybindingsView::new(bindings);

        keybindings_view.render(area, buffer);
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

    fn remain_in_view_mode(self) -> Mode {
        Mode::View(self)
    }

    fn increase_snippet_list_size(mut self) -> Mode {
        self.state.snippet_layout.increase_list_height();

        self.remain_in_view_mode()
    }

    fn decrease_snippet_list_size(mut self) -> Mode {
        self.state.snippet_layout.decrease_list_height();

        self.remain_in_view_mode()
    }

    fn toggle_maximize_snippet_list(mut self) -> Mode {
        self.state.snippet_layout.toggle_maximize_list();

        self.remain_in_view_mode()
    }

    fn toggle_maximize_snippet_view(mut self) -> Mode {
        self.state.snippet_layout.toggle_maximize_snippet();

        self.remain_in_view_mode()
    }
}

impl Widget for &mut ViewMode {
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

        self.render_keybindings(bottom_line_area, buffer);
        self.render_tag_list(tag_list_area, buffer);
    }
}
