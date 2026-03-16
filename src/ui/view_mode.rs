use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::Style, text::Line, widgets::{Block, BorderType, Borders, List, ListState, StatefulWidget, Widget}};
use crate::{snippets::Library, ui::{search_mode::SearchMode, state::{Mode, State}, tag_search_mode::TagSearchMode, widgets::{keybindings_view::{Binding, KeybindingsView}, snippet_view::{SnippetView, SnippetViewState}, tags_view::{TagsView, TagsViewState}}}};


pub(super) struct ViewMode {
    pub state: State,
    pub(super) description_list_state: ListState,
    pub(super) snippet_view_state: SnippetViewState,
    description_list_page_size: u16,
}

impl ViewMode {
    pub(super) fn new(library: Library) -> Self {
        ViewMode {
            state: State::new(library),
            description_list_state: ListState::default().with_selected(Some(0)),
            snippet_view_state: SnippetViewState::new(),
            description_list_page_size: 10,
        }
    }

    pub(super) fn init(state: State, description_list_state: ListState, snippet_view_state: SnippetViewState) -> Self {
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
            KeyCode::PageUp => {
                self.description_list_state.scroll_up_by(self.description_list_page_size);
                Mode::View(self)
            },
            KeyCode::PageDown => {
                self.description_list_state.scroll_down_by(self.description_list_page_size);
                Mode::View(self)
            },
            KeyCode::Tab => {
                self.snippet_view_state.select_next();
                Mode::View(self)
            },
            KeyCode::BackTab => {
                self.snippet_view_state.select_previous();
                Mode::View(self)
            },
            KeyCode::Char('?') => {
                self.state.clear_keywords();
                self.state.refresh();
                Mode::View(self)
            },
            KeyCode::Delete => {
                self.state.pop_selected_tag();
                self.state.refresh();
                Mode::View(self)
            },
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
                Mode::View(self)
            },
            // KeyCode::Char('c') => {
                // if let Some(index) = self.description_list_state.selected() {
                //     let snippet_id = &self.state.visible_snippets[index];
                //     let snippet = self.state.library.snippet(*snippet_id);

                //     if let Some(part) = snippet.parts.get(self.snippet_view_state.selected()) {
                //         let lines = &part.lines;
                //         let text = lines.join("\n");

                //         if let Err(error) = cli_clipboard::set_contents(text) {
                //             panic!("failed to copy snippet to clipboard: {}", error)
                //         }
                //     }
                // }
                // Mode::View(self)
            // },
            _ => Mode::View(self)
        }
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        let descriptions = self.state.visible_snippet_descriptions().collect::<Vec<_>>();
        let list_block = Block::new().title(Line::raw("Snippets")).borders(Borders::ALL).title_bottom(Line::raw(format!(" {} snippets ", descriptions.len())).right_aligned()).border_type(BorderType::Double);
        let list = List::new(descriptions).highlight_style(highlight_style).block(list_block);

        if area.height >= 2 {
            self.description_list_page_size = area.height - 2;
        }
        else {
            self.description_list_page_size = 1;
        }

        StatefulWidget::render(list, area, buffer, &mut self.description_list_state);
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
}

impl Widget for &mut ViewMode {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [upper_area, bottom_line_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        let [tag_list_area, right_area] = Layout::horizontal([Constraint::Length(40), Constraint::Fill(1)]).areas(upper_area);
        let [snippet_list_area, snippet_area] = Layout::vertical([Constraint::Length(15), Constraint::Fill(1)]).areas(right_area);

        self.render_snippet_list(snippet_list_area, buffer);
        self.render_selected_snippet(snippet_area, buffer);
        self.render_keybindings(bottom_line_area, buffer);
        self.render_tag_list(tag_list_area, buffer);
    }
}
