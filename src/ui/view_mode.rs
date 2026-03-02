use ratatui::{Frame, buffer::Buffer, crossterm::event::{Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::Style, text::Line, widgets::{Block, BorderType, Borders, List, ListItem, ListState, StatefulWidget, Widget}};
use crate::{snippets::Library, ui::{SearchParameters, search_mode::SearchMode, state::{Mode, State}, syntax::SyntaxHighlighter, tag_search_mode::TagSearchMode, tree_adapter::TreeAdapter, widgets::{snippet_view::{SnippetView, SnippetViewState}, tags_view::{TagsView, TagsViewState}, tree_view::{TreeView, TreeViewState}}}};


pub(super) struct ViewMode {
    pub state: State,
    pub(super) description_list_state: ListState,
    pub(super) snippet_view_state: SnippetViewState,

    // pub(super) library: Box<Library>,
    // pub(super) syntax_highlighter: Box<SyntaxHighlighter>,
    // pub(super) snippet_list: Vec<usize>,
    // pub(super) search_parameters: SearchParameters,
}

impl ViewMode {
    pub(super) fn new(library: Library, syntax_highlighter: SyntaxHighlighter) -> Self {
        ViewMode {
            state: State::new(library, syntax_highlighter),
            description_list_state: ListState::default().with_selected(Some(0)),
            snippet_view_state: SnippetViewState::new(),
        }

        // ViewMode {
        //     snippet_list: library.snippets().collect(),
        //     library: library,
        //     syntax_highlighter: syntax_highlighter,
        //     description_list_state: ListState::default().with_selected(Some(0)),
        //     snippet_view_state: SnippetViewState::new(),
        //     search_parameters: SearchParameters::new(),
        // }
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
            KeyCode::Tab => {
                self.snippet_view_state.select_next();
                Mode::View(self)
            },
            KeyCode::BackTab => {
                self.snippet_view_state.select_previous();
                Mode::View(self)
            },
            KeyCode::Esc => {
                self.state.clear_keywords();
                self.state.refresh();
                Mode::View(self)
            },
            KeyCode::Delete => {
                self.state.pop_selected_tag();
                self.state.refresh();
                Mode::View(self)
            }
            _ => Mode::View(self)
        }
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        let descriptions = self.state.visible_snippet_descriptions().collect::<Vec<_>>();
        let list_block = Block::new().title(Line::raw("Snippets")).borders(Borders::ALL).title_bottom(Line::raw(format!("{} snippets", descriptions.len())).right_aligned());
        let list = List::new(descriptions).highlight_style(highlight_style).block(list_block);

        StatefulWidget::render(list, area, buffer, &mut self.description_list_state);

        // let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        // let descriptions = self.snippet_list.iter().copied().map(|index| ListItem::new(self.library.snippet(index).description.as_str()) );
        // let list_block = Block::new().title(Line::raw("Snippets")).borders(Borders::ALL).title_bottom(Line::raw(format!("{} snippets", descriptions.len())).right_aligned());
        // let list = List::new(descriptions).highlight_style(highlight_style).block(list_block);

        // StatefulWidget::render(list, area, buffer, &mut self.description_list_state);
    }

    fn render_selected_snippet(&mut self, area: Rect, buffer: &mut Buffer) {
        match self.description_list_state.selected() {
            None => {},
            Some(selected_snippet_index) => {
                let snippet = self.state.library.snippet(self.state.visible_snippets[selected_snippet_index]);
                let snippet_view = SnippetView::new(snippet, &self.state.syntax_highlighter);
                snippet_view.render(area, buffer, &mut self.snippet_view_state);
            }
        }
    }

    fn render_tag_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let selected_tags = self.state.selected_tags.iter().map(String::as_str);
        let available_tags = self.state.visible_tags.iter().map(String::as_str);
        let tag_list = TagsView::new(selected_tags, available_tags);

        let block = Block::new().borders(Borders::all()).border_type(BorderType::Double).title("Tags");
        let tag_list_area = block.inner(area);

        block.render(area, buffer);
        let mut tags_view_state = TagsViewState::new();
        tag_list.render(tag_list_area, buffer, &mut tags_view_state);
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
        let [hierarchy_area, right_area] = Layout::horizontal([Constraint::Length(40), Constraint::Fill(1)]).areas(area);
        let [snippet_list_area, snippet_area] = Layout::vertical([Constraint::Length(15), Constraint::Fill(1)]).areas(right_area);

        self.render_tag_list(hierarchy_area, buffer);
        self.render_snippet_list(snippet_list_area, buffer);
        self.render_selected_snippet(snippet_area, buffer);
    }
}
