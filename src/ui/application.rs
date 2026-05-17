use std::{collections::HashSet, io, mem};

use ratatui::{DefaultTerminal, Frame, buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::{Style, Stylize}, widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget}};

use crate::{snippets::{Library, snippets::Tag}, timing, ui::{Mode, widgets}, util::Cyclic};

pub struct Application {
    active_mode: AppStateMode,
}

impl Application {
    pub fn new(library: Library) -> Self {
        Application{
            active_mode: AppStateMode::initial(library)
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while self.active_mode.is_running() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match &mut self.active_mode {
            AppStateMode::Quit => panic!("should not happen"),
            AppStateMode::View(state) => state.draw(frame),
            AppStateMode::TagSearch(state) => state.draw(frame),
        }
        // let (_, duration) = timing::measure(|| self.mode.draw(frame));
        // log::info!("Rendering frame took {}ms", duration.as_millis());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        let active_mode = mem::replace(&mut self.active_mode, AppStateMode::Quit);
        self.active_mode = match active_mode {
            AppStateMode::Quit => panic!("should not happen"),
            AppStateMode::View(state) => state.handle_event(event),
            AppStateMode::TagSearch(state) => state.handle_event(event),
        };
        Ok(())
    }
}

enum AppStateMode {
    Quit,
    View(AppState<mode::View>),
    TagSearch(AppState<mode::TagSearch>),
}

impl AppStateMode {
    pub fn initial(library: Library) -> Self {
        AppStateMode::View(AppState::initial(library))
    }

    pub fn is_running(&self) -> bool {
        match self {
            AppStateMode::Quit => false,
            _ => true,
        }
    }
}

struct AppState<Mode: mode::Mode> {
    library: Library,
    mode: Mode,
    filtering_tags: Vec<Tag>,
    filtering_keywords: Vec<String>,
    listed_snippets: Vec<usize>,
    listed_tags: Vec<Tag>,
    shown_snippet: Cyclic,
}

mod mode {
    use crate::{snippets::snippets::Tag, ui::widgets, util::Cyclic};

    pub trait Mode { }

    pub struct View {
        pub snippet_list_state: widgets::description_list::State,
        pub snippet_viewer_state: widgets::snippet_view::SnippetViewState,
    }

    impl View {
        pub fn initial() -> Self {
            View {
                snippet_list_state: widgets::description_list::State::default(),
                snippet_viewer_state: widgets::snippet_view::SnippetViewState::new(),
            }
        }
    }

    impl Mode for View { }

    pub struct TagSearch {
        pub tag_list_state: widgets::tags_view::TagsViewState,
        pub selected_tag_index: Cyclic,
        pub tag_input: String,
        pub leftover_tags: Vec<Tag>,
    }

    impl TagSearch {
        pub fn new(tag_list: Vec<Tag>) -> Self {
            TagSearch {
                tag_list_state: widgets::tags_view::TagsViewState::new(),
                selected_tag_index: Cyclic::new(0, tag_list.len() as u64),
                tag_input: String::new(),
                leftover_tags: tag_list,
            }
        }
    }

    impl Mode for TagSearch { }
}

impl<Mode: mode::Mode> AppState<Mode> {
    fn quit(self) -> AppStateMode {
        AppStateMode::Quit
    }

    fn assert_invariant(&self) {
        debug_assert_eq!(self.shown_snippet.modulo() as usize, self.listed_snippets.len());
    }

    fn add_tag_to_filter(&mut self, tag: Tag) {
        self.filtering_tags.push(tag);
        self.refresh();
    }

    /// Recomputes the list of snippets and the list of tags.
    fn refresh(&mut self) {
        self.listed_snippets = self.library.search(self.filtering_keywords.iter().map(String::as_str), self.filtering_tags.iter().map(|tag| tag.name.as_str()));

        let mut tags: HashSet<&Tag> = HashSet::new();

        // Find the union of the tags of all visible snippets
        for snippet_id in self.listed_snippets.iter().copied() {
            let snippet = self.library.snippet(snippet_id);
            let snippet_tags = &snippet.tags;

            for snippet_tag in snippet_tags.iter() {
                tags.insert(&snippet_tag);
            }
        }

        // Remove the already selected tags
        for selected_tag in self.filtering_tags.iter() {
            tags.remove(selected_tag);
        }

        self.listed_tags = tags.into_iter().cloned().collect();
        self.listed_tags.sort_by(|tag1, tag2| tag1.name.cmp(&tag2.name));
        self.shown_snippet = Cyclic::new(0, self.listed_snippets.len() as u64);
    }
}

impl AppState<mode::View> {
    pub fn initial(library: Library) -> Self {
        let listed_tags = library.tags().clone();
        let listed_snippets = library.snippets().collect::<Vec<_>>();

        AppState {
            library,
            shown_snippet: Cyclic::new(0, listed_snippets.len() as u64),
            mode: mode::View::initial(),
            filtering_keywords: Vec::new(),
            filtering_tags: Vec::new(),
            listed_snippets,
            listed_tags,
        }
    }

    fn render_tag_list(&self, area: Rect, buffer: &mut Buffer) {
        let border = Block::new().borders(Borders::ALL).border_type(BorderType::Plain);
        let inner_area = border.inner(area);

        let tag_list = {
            let selected_tags = self.filtering_tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();
            let listed_tags = &self.listed_tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();

            widgets::tags_view::TagsView::new(selected_tags.iter().copied(), listed_tags.iter().copied())
        };

        ratatui::widgets::Widget::render(border, area, buffer);
        ratatui::widgets::Widget::render(tag_list, inner_area, buffer);
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let snippet_list = {
            let items = self.listed_snippets.iter().map(|id| self.library.snippet(*id).description.as_str());

            widgets::description_list::Widget::new(items, true)
        };
        let snippet_list_state = &mut self.mode.snippet_list_state;
        snippet_list_state.select(self.shown_snippet.into());

        ratatui::widgets::StatefulWidget::render(snippet_list, area, buffer, snippet_list_state);
    }

    fn render_snippet(&mut self, area: Rect, buffer: &mut Buffer) {
        let library = &self.library;
        let snippet = library.snippet(self.shown_snippet.into());
        let snippet_viewer = widgets::snippet_view::SnippetView::new(snippet, library);
        let snippet_viewer_state = &mut self.mode.snippet_viewer_state;

        ratatui::widgets::StatefulWidget::render(snippet_viewer, area, buffer, snippet_viewer_state);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.assert_invariant();

        let (tag_list_area, snippet_list_area, snippet_viewer_area) = {
            let area = frame.area();
            let [ tag_list_area, right_area ] = Layout::horizontal([ Constraint::Length(20), Constraint::Fill(1) ]).areas(area);
            let [ snippet_list_area, snippet_view_area ] = Layout::vertical([ Constraint::Length(15), Constraint::Fill(1) ]).areas(right_area);

            ( tag_list_area, snippet_list_area, snippet_view_area )
        };

        let buffer = frame.buffer_mut();
        self.render_tag_list(tag_list_area, buffer);
        self.render_snippet_list(snippet_list_area, buffer);
        self.render_snippet(snippet_viewer_area, buffer);

        self.assert_invariant();
    }

    pub fn handle_event(self, event: Event) -> AppStateMode {
        self.assert_invariant();

        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            _ => self.remain_in_view_mode()
        }
    }

    fn handle_key_event(self, event: KeyEvent) -> AppStateMode {
        if event.is_press() {
            match event.code {
                KeyCode::Char('q') => self.quit(),
                KeyCode::Char('#') => self.switch_to_tag_search_mode(),
                KeyCode::Up => self.highlight_previous_snippet(),
                KeyCode::Down => self.highlight_next_snippet(),
                _ => self.remain_in_view_mode(),
            }
        }
        else {
            self.remain_in_view_mode()
        }
    }

    fn remain_in_view_mode(self) -> AppStateMode {
        self.assert_invariant();

        self.into()
    }

    fn switch_to_tag_search_mode(self) -> AppStateMode {
        self.assert_invariant();

        AppStateMode::TagSearch(AppState {
            library: self.library,
            mode: mode::TagSearch::new(self.listed_tags.clone()),
            filtering_tags: self.filtering_tags,
            filtering_keywords: self.filtering_keywords,
            listed_snippets: self.listed_snippets,
            listed_tags: self.listed_tags,
            shown_snippet: self.shown_snippet,
        })
    }

    fn highlight_previous_snippet(mut self) -> AppStateMode {
        self.shown_snippet = self.shown_snippet.sub(1);

        self.remain_in_view_mode()
    }

    fn highlight_next_snippet(mut self) -> AppStateMode {
        self.shown_snippet = self.shown_snippet.add(1);

        self.remain_in_view_mode()
    }
}

impl AppState<mode::TagSearch> {
    pub fn draw(&mut self, frame: &mut Frame) {
        self.assert_invariant();

        let (search_bar_area, tag_list_area, snippet_list_area) = {
            let area = frame.area();
            let [ upper_area, search_bar_area ] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
            let [ tag_list_area, snippet_list_area ] = Layout::horizontal([ Constraint::Length(20), Constraint::Fill(1) ]).areas(upper_area);

            ( search_bar_area, tag_list_area, snippet_list_area )
        };

        let buffer = frame.buffer_mut();
        self.render_search_bar(search_bar_area, buffer);
        self.render_tag_list(tag_list_area, buffer);
        self.render_snippet_list(snippet_list_area, buffer);

        self.assert_invariant();
    }

    fn render_tag_list(&mut self, area: Rect, buffer: &mut Buffer) {
        self.mode.tag_list_state.select(self.mode.selected_tag_index.value() as usize);

        let border = Block::new().borders(Borders::ALL).border_type(BorderType::Double);
        let inner_area = border.inner(area);

        let tag_list = {
            let selected_tags = self.filtering_tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();
            let listed_tags = self.mode.leftover_tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();

            widgets::tags_view::TagsView::new(selected_tags.iter().copied(), listed_tags.iter().copied())
        };

        ratatui::widgets::Widget::render(border, area, buffer);
        ratatui::widgets::StatefulWidget::render(tag_list, inner_area, buffer, &mut self.mode.tag_list_state);
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let snippet_list = {
            let items = self.listed_snippets.iter().map(|id| self.library.snippet(*id).description.as_str());

            widgets::description_list::Widget::new(items, false)
        };

        ratatui::widgets::Widget::render(snippet_list, area, buffer);
    }

    fn render_search_bar(&mut self, area: Rect, buffer: &mut Buffer) {
        let prompt = format!("#{}", self.mode.tag_input);
        let paragraph = ratatui::widgets::Paragraph::new(prompt).on_light_blue();

        paragraph.render(area, buffer);
    }

    pub fn handle_event(self, event: Event) -> AppStateMode {
        self.assert_invariant();

        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            _ => self.remain_in_tag_search_mode()
        }
    }

    fn handle_key_event(self, event: KeyEvent) -> AppStateMode {
        if event.is_press() {
            match event.code {
                KeyCode::Esc => self.cancel_tag_search(),
                KeyCode::Up => self.highlight_previous_tag(),
                KeyCode::Down => self.highlight_next_tag(),
                KeyCode::Backspace => self.remove_char(),
                KeyCode::Enter => self.select_highlighted_tag(),
                KeyCode::Char(char) if self.is_valid_tag_character(char) => self.add_char(char),
                _ => self.remain_in_tag_search_mode(),
            }
        }
        else {
            self.remain_in_tag_search_mode()
        }
    }

    fn is_valid_tag_character(&self, char: char) -> bool {
        char.is_ascii_graphic()
    }

    fn remain_in_tag_search_mode(self) -> AppStateMode {
        self.assert_invariant();

        self.into()
    }

    fn cancel_tag_search(self) -> AppStateMode {
        self.switch_to_view_mode()
    }

    fn switch_to_view_mode(self) -> AppStateMode {
        self.assert_invariant();

        AppState {
            library: self.library,
            mode: mode::View::initial(),
            filtering_tags: self.filtering_tags,
            filtering_keywords: self.filtering_keywords,
            listed_snippets: self.listed_snippets,
            listed_tags: self.listed_tags,
            shown_snippet: self.shown_snippet,
        }.into()
    }

    fn add_char(mut self, char: char) -> AppStateMode {
        self.mode.tag_input.push(char);
        self.refresh_tag_list();

        self.remain_in_tag_search_mode()
    }

    fn remove_char(mut self) -> AppStateMode {
        self.mode.tag_input.pop();
        self.refresh_tag_list();

        self.remain_in_tag_search_mode()
    }

    fn highlight_previous_tag(mut self) -> AppStateMode {
        self.mode.selected_tag_index = self.mode.selected_tag_index.sub(1);

        self.remain_in_tag_search_mode()
    }

    fn highlight_next_tag(mut self) -> AppStateMode {
        self.mode.selected_tag_index = self.mode.selected_tag_index.add(1);

        self.remain_in_tag_search_mode()
    }

    fn refresh_tag_list(&mut self) {
        let lowercased = self.mode.tag_input.to_lowercase();

        self.mode.leftover_tags = self.listed_tags.iter().filter(|tag| tag.name.to_lowercase().starts_with(lowercased.as_str())).cloned().collect::<Vec<_>>();
        self.mode.selected_tag_index = Cyclic::new(0, self.mode.leftover_tags.len() as u64);
    }

    fn select_highlighted_tag(mut self) -> AppStateMode {
        let selected_index = self.mode.selected_tag_index.value() as usize;
        let selected_tag = self.mode.leftover_tags[selected_index].clone();
        self.add_tag_to_filter(selected_tag);

        self.switch_to_view_mode()
    }
}

impl From<AppState<mode::View>> for AppStateMode {
    fn from(state: AppState<mode::View>) -> Self {
        AppStateMode::View(state)
    }
}

impl From<AppState<mode::TagSearch>> for AppStateMode {
    fn from(state: AppState<mode::TagSearch>) -> Self {
        AppStateMode::TagSearch(state)
    }
}