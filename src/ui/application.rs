use std::{collections::HashSet, io, mem};

use ratatui::{DefaultTerminal, Frame, buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers}, layout::{Constraint, Layout, Margin, Rect}, style::Stylize, widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Widget}};

use crate::{external, snippets::{Library, snippets::{Page, Snippet, Tag}}, ui::{application::mode::ViewOverlay, widgets}, util::Cyclic};

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
            AppStateMode::KeywordSearch(state) => state.draw(frame),
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
            AppStateMode::KeywordSearch(state) => state.handle_event(event),
        };
        Ok(())
    }
}

enum AppStateMode {
    Quit,
    View(AppState<mode::View>),
    TagSearch(AppState<mode::TagSearch>),
    KeywordSearch(AppState<mode::KeywordSearch>),
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
}

mod mode {
    use crate::{snippets::snippets::Tag, ui::widgets, util::Cyclic};

    pub trait Mode { }

    pub struct View {
        pub(super) active_tags: Vec<Tag>,
        pub(super) snippets: Vec<usize>,
        pub(super) tags: Vec<Tag>,
        pub(super) highlighted_snippet_index: Option<Cyclic>,
        pub(super) shown_snippet: usize,
        pub(super) snippet_list_state: widgets::description_list::State,
        pub(super) snippet_viewer_state: widgets::snippet_view::State,
        pub(super) page_size: Option<usize>,
        pub(super) overlay: ViewOverlay,
    }

    pub enum ViewOverlay {
        None,
        CopySnippet,
    }

    impl View {
        pub fn initial(snippets: Vec<usize>, tags: Vec<Tag>) -> Self {
            assert!(!snippets.is_empty(), "no snippets");

            View {
                active_tags: Vec::new(),
                highlighted_snippet_index: Cyclic::new(0, snippets.len()).into(),
                shown_snippet: snippets[0],
                snippets,
                tags,
                snippet_list_state: widgets::description_list::State::default(),
                snippet_viewer_state: widgets::snippet_view::State::new(),
                page_size: None,
                overlay: ViewOverlay::None,
            }
        }

        pub fn new(snippets: Vec<usize>, tags: Vec<Tag>, active_tags: Vec<Tag>, highlighted_snippet_index: Option<Cyclic>, shown_snippet: usize) -> Self {
            assert!(!snippets.is_empty(), "no snippets");

            View {
                active_tags,
                highlighted_snippet_index,
                shown_snippet,
                snippets,
                tags,
                snippet_list_state: widgets::description_list::State::default(),
                snippet_viewer_state: widgets::snippet_view::State::new(),
                page_size: None,
                overlay: ViewOverlay::None,
            }
        }
    }

    impl Mode for View { }

    pub struct TagSearch {
        /// List of snippets before the search started
        pub(super) snippets: Vec<usize>,

        /// Tags that were listed before the search started
        pub(super) original_tags: Vec<Tag>,

        /// Tags that were activated before the search started
        pub(super) active_tags: Vec<Tag>,

        /// Edited by user, used to filter tags
        pub(super) tag_input: String,

        /// Tags from original_tags that match tag_input
        pub(super) tags: Vec<Tag>,

        /// Index of the highlighted tag
        pub(super) highlighted_tag_index: Option<Cyclic>,

        /// State of the tag list
        pub(super) tag_list_state: widgets::tags_view::State,

        /// Size of a page that fits on the screen; used by the PgUp and PgDown handlers
        pub(super) page_size: Option<usize>,
    }

    impl TagSearch {
        pub fn new(snippets: Vec<usize>, tag_list: Vec<Tag>, active_tags: Vec<Tag>) -> Self {
            let highlighted_tag_index = {
                if tag_list.is_empty() {
                    None
                }
                else {
                    Some(Cyclic::new(0, tag_list.len()))
                }
            };

            TagSearch {
                tag_list_state: widgets::tags_view::State::new(),
                highlighted_tag_index,
                tag_input: String::new(),
                tags: tag_list.clone(),
                active_tags,
                snippets,
                original_tags: tag_list,
                page_size: None,
            }
        }
    }

    impl Mode for TagSearch { }

    pub struct KeywordSearch {
        /// Original list of snippets, pre-search
        pub(super) snippets: Vec<usize>,

        /// List of tags
        pub(super) tags: Vec<Tag>,

        /// List of active tags
        pub(super) active_tags: Vec<Tag>,

        /// Keywords used in filtering
        pub(super) keywords: Vec<String>,

        /// Subset of snippets that match keywords
        pub(super) filtered_snippets: Vec<usize>,

        /// Highlighted snippet pre-search; restored if user cancels search
        pub(super) original_highlighted_snippet_index: Option<Cyclic>,

        /// Index of highlighted snippet, can be moved up and down
        pub(super) highlighted_snippet_index: Option<Cyclic>,

        /// Snippet being shown in snippet viewer
        pub(super) shown_snippet: usize,

        /// State of snippet list widget
        pub(super) snippet_list_state: widgets::description_list::State,

        /// State of snippet viewer widget
        pub(super) snippet_viewer_state: widgets::snippet_view::State,

        /// Page size to be used when user presses pgup/pgdown
        pub(super) page_size: Option<usize>,
    }

    impl KeywordSearch {
        pub fn new(snippets: Vec<usize>, tags: Vec<Tag>, active_tags: Vec<Tag>, highlighted_snippet_index: Option<Cyclic>) -> Self {
            KeywordSearch {
                keywords: vec![String::new()],
                original_highlighted_snippet_index: highlighted_snippet_index,
                highlighted_snippet_index: highlighted_snippet_index,
                shown_snippet: snippets[0],
                snippet_list_state: widgets::description_list::State::default(),
                snippet_viewer_state: widgets::snippet_view::State::new(),
                filtered_snippets: snippets.clone(),
                tags,
                active_tags,
                snippets,
                page_size: None,
            }
        }
    }

    impl Mode for KeywordSearch { }
}

/// Recomputes the list of snippets and the list of tags.
fn apply_filters<'a, 'b>(library: &Library, filtering_keywords: impl Iterator<Item=&'a str>, filtering_tags: impl Iterator<Item=&'b Tag>) -> (Vec<usize>, Vec<Tag>) {
    // We need to iterate twice over the tags, so we need to make a copy
    let filtering_tags = filtering_tags.collect::<Vec<_>>();

    // Ask library for snippets that match the given keywords and tags
    let snippets = {
        let tag_names = filtering_tags.iter().map(|tag| tag.name.as_str());
        library.search(filtering_keywords, tag_names)
    };

    // Collect union of the tags of snippets
    let mut tags: HashSet<&Tag> = HashSet::new();
    for snippet_id in snippets.iter().copied() {
        let snippet = library.snippet(snippet_id);
        let snippet_tags = &snippet.tags;

        for snippet_tag in snippet_tags.iter() {
            tags.insert(&snippet_tag);
        }
    }

    // Remove the already selected tags
    for selected_tag in filtering_tags {
        tags.remove(&selected_tag);
    }

    let mut tags = tags.into_iter().cloned().collect::<Vec<_>>();
    tags.sort_by(|tag1, tag2| tag1.name.to_lowercase().cmp(&tag2.name.to_lowercase()));

    (snippets, tags)
}

impl<Mode: mode::Mode> AppState<Mode> {
    fn quit(self) -> AppStateMode {
        AppStateMode::Quit
    }

    fn find_snippet_in_list(&self, snippet_id: usize, list: &Vec<usize>) -> Option<usize> {
        match list.binary_search(&snippet_id) {
            Ok(index) => {
                debug_assert!(list[index] == snippet_id, "binary search failed to do its job; was the list correctly sorted?");
                Some(index)
            },
            Err(_) => None
        }
    }
}

impl AppState<mode::View> {
    pub fn initial(library: Library) -> Self {
        let listed_tags = library.tags().clone();
        let listed_snippets = library.snippets().collect::<Vec<_>>();

        AppState {
            library,
            mode: mode::View::initial(listed_snippets, listed_tags),
        }
    }

    fn render_tag_list(&self, area: Rect, buffer: &mut Buffer) {
        let border = Block::new().borders(Borders::ALL).border_type(BorderType::Plain);
        let inner_area = border.inner(area);

        let tag_list = {
            let selected_tags = self.mode.active_tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();
            let listed_tags = &self.mode.tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();

            widgets::tags_view::Widget::new(selected_tags.iter().copied(), listed_tags.iter().copied())
        };

        ratatui::widgets::Widget::render(border, area, buffer);
        ratatui::widgets::Widget::render(tag_list, inner_area, buffer);
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        self.mode.page_size = Some(area.height as usize);

        let snippet_list = {
            let items = self.mode.snippets.iter().map(|id| self.library.snippet(*id).description.as_str());

            widgets::description_list::Widget::new(items, true)
        };
        let snippet_list_state = &mut self.mode.snippet_list_state;
        snippet_list_state.select(self.mode.highlighted_snippet_index.map(|c| c.value()));

        ratatui::widgets::StatefulWidget::render(snippet_list, area, buffer, snippet_list_state);
    }

    fn render_snippet(&mut self, area: Rect, buffer: &mut Buffer) {
        let library = &self.library;
        let snippet = library.snippet(self.mode.shown_snippet);
        let snippet_viewer = widgets::snippet_view::Widget::new(snippet, library);
        let snippet_viewer_state = &mut self.mode.snippet_viewer_state;

        ratatui::widgets::StatefulWidget::render(snippet_viewer, area, buffer, snippet_viewer_state);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.assert_invariant();

        let area = frame.area();

        let (tag_list_area, snippet_list_area, snippet_viewer_area) = {
            let [ tag_list_area, right_area ] = Layout::horizontal([ Constraint::Length(20), Constraint::Fill(1) ]).areas(area);
            let [ snippet_list_area, snippet_view_area ] = Layout::vertical([ Constraint::Length(15), Constraint::Fill(1) ]).areas(right_area);

            ( tag_list_area, snippet_list_area, snippet_view_area )
        };

        let buffer = frame.buffer_mut();
        self.render_tag_list(tag_list_area, buffer);
        self.render_snippet_list(snippet_list_area, buffer);
        self.render_snippet(snippet_viewer_area, buffer);

        match self.mode.overlay {
            ViewOverlay::None => { },
            ViewOverlay::CopySnippet => self.render_copy_snippet_overlay(area, buffer),
        }

        self.assert_invariant();
    }

    fn currently_shown_snippet(&self) -> &Snippet {
        let snippet_id = self.mode.shown_snippet;

        self.library.snippet(snippet_id)
    }

    fn currently_shown_page(&self) -> &Page {
        let snippet = self.currently_shown_snippet();
        let pages = &snippet.pages;
        let shown_page = &pages[0];

        shown_page
    }

    fn render_copy_snippet_overlay(&mut self, area: Rect, buffer: &mut Buffer) {
        let block_caption = " Copy snippet to clipboard ";

        let shown_page = self.currently_shown_page();
        let code_blocks = shown_page.document().code_fragments();
        let list_items = code_blocks.enumerate().map(|(index, code)| {
            let language = code.language.as_ref().map(String::as_str).unwrap_or("unknown language");
            let caption = code.metadata.as_ref().map(String::as_str).unwrap_or("Code snippet");
            let list_item_content = format!("[{index}] {caption} ({language})", index=index+1, caption=caption, language=language);

            ListItem::new(list_item_content)
        }).collect::<Vec<_>>();
        let longest_list_item = list_items.iter().map(ListItem::width).max().unwrap();
        let required_width = *[longest_list_item + 4, block_caption.len() + 4].iter().max().unwrap();
        let required_height = list_items.len() + 4;

        let overlay_area = area.centered(Constraint::Length(required_width as u16), Constraint::Length(required_height as u16));
        let block = Block::new().borders(Borders::ALL).border_type(BorderType::Double).title(" Copy snippet to clipboard ").padding(Padding::uniform(1)).on_dark_gray();
        let block_inner_area = block.inner(overlay_area);
        Clear::default().render(overlay_area, buffer);
        block.render(overlay_area, buffer);

        let list = List::new(list_items);

        list.render(block_inner_area, buffer);
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
            match self.mode.overlay {
                ViewOverlay::None => {
                    match event.code {
                        KeyCode::Char('q') => self.quit(),
                        KeyCode::Char('#') => self.switch_to_tag_search_mode(),
                        KeyCode::Char('/') => self.switch_to_keyword_search_mode(),
                        KeyCode::Char('c') => self.show_copy_snippet_overlay(),
                        KeyCode::Char('C') => self.copy_first_to_clipboard(),
                        KeyCode::Delete => self.drop_filtering_tag(),
                        KeyCode::Up => self.highlight_previous_snippet(),
                        KeyCode::Down => self.highlight_next_snippet(),
                        KeyCode::PageUp => self.highlight_previous_page(),
                        KeyCode::PageDown => self.highlight_next_page(),
                        KeyCode::Home => self.highlight_first_snippet(),
                        KeyCode::End => self.highlight_last_snippet(),
                        _ => self.remain_in_view_mode(),
                    }
                },
                ViewOverlay::CopySnippet => {
                    match event.code {
                        KeyCode::Esc => self.remove_overlay(),
                        KeyCode::Char(char) if char.is_ascii_digit() => self.copy_code_to_clipboard(char),
                        _ => self.remain_in_view_mode(),
                    }
                }
            }
        }
        else {
            self.remain_in_view_mode()
        }
    }

    fn copy_first_to_clipboard(self) -> AppStateMode {
        let page = self.currently_shown_page();

        if let Some(code) = page.document().code_fragments().next() {
            let to_be_copied = code.original.clone();

            external::clipboard::copy_to_clipboard(to_be_copied).unwrap();
        }

        self.remain_in_view_mode()
    }

    fn show_copy_snippet_overlay(mut self) -> AppStateMode {
        // Find out if the snippet actually contains code blocks
        let page = self.currently_shown_page();
        let has_at_least_one_code_block = page.document().code_fragments().next().is_some();

        if has_at_least_one_code_block {
            self.mode.overlay = ViewOverlay::CopySnippet;
        }

        self.remain_in_view_mode()
    }

    fn remove_overlay(mut self) -> AppStateMode {
        self.mode.overlay = ViewOverlay::None;

        self.remain_in_view_mode()
    }

    fn copy_code_to_clipboard(mut self, char: char) -> AppStateMode {
        let page = self.currently_shown_page();
        let index = {
            let digit = char.to_digit(10).unwrap();
            if digit == 0 {
                9
            }
            else {
                digit - 1
            }
        } as usize;
        let block = page.document().code_fragments().nth(index);

        if let Some(code) = block {
            external::clipboard::copy_to_clipboard(code.original.clone()).unwrap();
            self.remove_overlay()
        }
        else {
            self.remain_in_view_mode()
        }
    }

    fn assert_invariant(&self) {
        if let Some(highlighted_snippet_index) = self.mode.highlighted_snippet_index {
            debug_assert_eq!(highlighted_snippet_index.modulo(), self.mode.snippets.len());
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
            mode: mode::TagSearch::new(self.mode.snippets, self.mode.tags, self.mode.active_tags),
        })
    }

    fn switch_to_keyword_search_mode(self) -> AppStateMode {
        self.assert_invariant();

        AppStateMode::KeywordSearch(AppState {
            library: self.library,
            mode: mode::KeywordSearch::new(self.mode.snippets, self.mode.tags, self.mode.active_tags, self.mode.highlighted_snippet_index),
        })
    }

    fn highlight_previous_snippet(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = index.sub(1);
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_view_mode()
    }

    fn highlight_next_snippet(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = index.add(1);
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_view_mode()
    }

    fn highlight_first_snippet(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = index.set(0);
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_view_mode()
    }

    fn highlight_last_snippet(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = index.set(index.modulo() - 1);
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_view_mode()
    }

    fn highlight_previous_page(mut self) -> AppStateMode {
        let page_size = self.mode.page_size.unwrap();

        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = {
                if index.value() < page_size {
                    index.set(0)
                }
                else {
                    index.sub(page_size)
                }
            };
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_view_mode()
    }

    fn highlight_next_page(mut self) -> AppStateMode {
        let page_size = self.mode.page_size.unwrap();

        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = {
                if index.value() + page_size < index.modulo() {
                    index.add(page_size)
                }
                else {
                    index.set(index.modulo() - 1)
                }
            };
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_view_mode()
    }

    fn drop_filtering_tag(mut self) -> AppStateMode {
        self.mode.active_tags.pop();
        self.refresh();

        self.remain_in_view_mode()
    }

    fn refresh(&mut self) {
        let (snippets, tags) = apply_filters(&self.library, Vec::new().into_iter(), self.mode.active_tags.iter());

        self.mode.highlighted_snippet_index = Cyclic::new(0, snippets.len()).into();
        self.mode.shown_snippet = snippets[0];
        self.mode.snippets = snippets;
        self.mode.tags = tags;
    }
}

impl AppState<mode::TagSearch> {
    fn assert_invariant(&self) {
        // TODO
    }

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
        self.mode.tag_list_state.select(self.mode.highlighted_tag_index.map(|c| c.value()));

        let border = Block::new().borders(Borders::ALL).border_type(BorderType::Double);
        let inner_area = border.inner(area);

        self.mode.page_size = Some(inner_area.height as usize);

        let tag_list = {
            let active_tags = self.mode.active_tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();
            let listed_tags = self.mode.tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();

            widgets::tags_view::Widget::new(active_tags.iter().copied(), listed_tags.iter().copied())
        };

        ratatui::widgets::Widget::render(border, area, buffer);
        ratatui::widgets::StatefulWidget::render(tag_list, inner_area, buffer, &mut self.mode.tag_list_state);
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let snippet_list = {
            let items = self.mode.snippets.iter().map(|id| self.library.snippet(*id).description.as_str());

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
                KeyCode::Home => self.highlight_first_tag(),
                KeyCode::End => self.highlight_last_tag(),
                KeyCode::PageDown => self.highlight_page_down(),
                KeyCode::PageUp => self.highlight_page_up(),
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
        self.assert_invariant();

        let highlighted_snippet_index = Cyclic::new(0, self.mode.snippets.len());

        AppState {
            library: self.library,
            mode: mode::View::new(self.mode.snippets, self.mode.original_tags, self.mode.active_tags, highlighted_snippet_index.into(), 0),
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
        if let Some(index) = self.mode.highlighted_tag_index {
            self.mode.highlighted_tag_index = index.sub(1).into();
        }
        else {
            assert!(self.mode.tags.is_empty(), "missing highlighted tag index only allowed when there are no tags to highlight");
        }

        self.remain_in_tag_search_mode()
    }

    fn highlight_next_tag(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_tag_index {
            self.mode.highlighted_tag_index = index.add(1).into();
        }
        else {
            assert!(self.mode.tags.is_empty(), "missing highlighted tag index only allowed when there are no tags to highlight");
        }

        self.remain_in_tag_search_mode()
    }

    fn highlight_first_tag(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_tag_index {
            self.mode.highlighted_tag_index = index.set(0).into();
        }
        else {
            assert!(self.mode.tags.is_empty(), "missing highlighted tag index only allowed when there are no tags to highlight");
        }

        self.remain_in_tag_search_mode()
    }

    fn highlight_last_tag(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_tag_index {
            self.mode.highlighted_tag_index = index.set(self.mode.tags.len() - 1).into();
        }
        else {
            assert!(self.mode.tags.is_empty(), "missing highlighted tag index only allowed when there are no tags to highlight");
        }

        self.remain_in_tag_search_mode()
    }

    fn highlight_page_up(mut self) -> AppStateMode {
        let page_size = self.mode.page_size.unwrap();

        if let Some(index) = self.mode.highlighted_tag_index {
            if index.value() < page_size {
                self.mode.highlighted_tag_index = index.set(0).into();
            }
            else {
                self.mode.highlighted_tag_index = index.sub(self.mode.page_size.unwrap()).into();
            }
        }
        else {
            assert!(self.mode.tags.is_empty(), "missing highlighted tag index only allowed when there are no tags to highlight");
        }

        self.remain_in_tag_search_mode()
    }

    fn highlight_page_down(mut self) -> AppStateMode {
        let page_size = self.mode.page_size.unwrap();

        if let Some(index) = self.mode.highlighted_tag_index {
            if index.value() + page_size >= index.modulo() {
                self.mode.highlighted_tag_index = index.set(index.modulo() - 1).into();
            }
            else {
                self.mode.highlighted_tag_index = index.add(self.mode.page_size.unwrap()).into();
            }
        }
        else {
            assert!(self.mode.tags.is_empty(), "missing highlighted tag index only allowed when there are no tags to highlight");
        }

        self.remain_in_tag_search_mode()
    }

    fn refresh_tag_list(&mut self) {
        let lowercased = self.mode.tag_input.to_lowercase();

        self.mode.tags = self.mode.original_tags.iter().filter(|tag| tag.name.to_lowercase().starts_with(lowercased.as_str())).cloned().collect::<Vec<_>>();
        self.mode.highlighted_tag_index = if self.mode.tags.is_empty() { None } else { Cyclic::new(0, self.mode.tags.len()).into() };
    }

    fn select_highlighted_tag(self) -> AppStateMode {
        if let Some(highlighted_tag_index) = self.mode.highlighted_tag_index {
            self.assert_invariant();

            let updated_active_tags = {
                let selected_index = highlighted_tag_index.value();
                let selected_tag = self.mode.tags[selected_index].clone();
                let mut tags = self.mode.active_tags;
                tags.push(selected_tag);
                tags
            };

            let (snippets, tags) = apply_filters(&self.library, Vec::new().into_iter(), updated_active_tags.iter());

            let highlighted_snippet_index = Cyclic::new(0, snippets.len());
            let shown_snippet = snippets[0];

            AppState {
                library: self.library,
                mode: mode::View::new(snippets, tags, updated_active_tags, highlighted_snippet_index.into(), shown_snippet),
            }.into()
        }
        else {
            // User tried to select tag while none was highlighted
            self.remain_in_tag_search_mode()
        }
    }
}

impl AppState<mode::KeywordSearch> {
    fn assert_invariant(&self) {
        // TODO
    }

    pub fn handle_event(self, event: Event) -> AppStateMode {
        self.assert_invariant();

        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            _ => self.remain_in_keyword_search_mode()
        }
    }

    fn handle_key_event(self, event: KeyEvent) -> AppStateMode {
        if event.is_press() {
            log::debug!("Key pressed: {:?}", event.code);
            match event.code {
                KeyCode::Esc => self.cancel_keyword_search(),
                KeyCode::Enter => self.switch_to_view_mode(),
                KeyCode::Up => self.highlight_previous_snippet(),
                KeyCode::Down => self.highlight_next_snippet(),
                KeyCode::PageUp => self.highlight_previous_page(),
                KeyCode::PageDown => self.highlight_next_page(),
                KeyCode::Home => self.highlight_first_snippet(),
                KeyCode::End => self.highlight_last_snippet(),
                KeyCode::Char(char) if self.is_valid_keyword_character(char) => self.add_char(char),
                KeyCode::Char(' ') => self.start_new_keyword(),
                KeyCode::Backspace => {
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.drop_last_keyword()
                    }
                    else {
                        self.drop_last_char()
                    }
                },
                _ => self.remain_in_keyword_search_mode(),
            }
        }
        else {
            self.remain_in_keyword_search_mode()
        }
    }

    fn is_valid_keyword_character(&self, char: char) -> bool {
        char.is_ascii_graphic()
    }

    fn add_char(mut self, char: char) -> AppStateMode {
        debug_assert!(self.mode.keywords.len() > 0, "this vec should always contain at least one element");

        self.mode.keywords.last_mut().unwrap().push(char);
        self.refresh_snippet_list();

        self.remain_in_keyword_search_mode()
    }

    fn start_new_keyword(mut self) -> AppStateMode {
        debug_assert!(self.mode.keywords.len() > 0, "this vec should always contain at least one element");

        if !self.mode.keywords.last().unwrap().is_empty() {
            self.mode.keywords.push(String::new());
        }

        self.remain_in_keyword_search_mode()
    }

    fn drop_last_char(mut self) -> AppStateMode {
        debug_assert!(self.mode.keywords.len() > 0, "this vec should always contain at least one element");

        if self.mode.keywords.last().unwrap().is_empty() {
            if self.mode.keywords.len() >= 2 {
                self.mode.keywords.pop();
                debug_assert!(self.mode.keywords.last().unwrap().len() > 0, "empty keywords should not have been allowed");
                self.mode.keywords.last_mut().unwrap().pop();
            }
        }
        else {
            self.mode.keywords.last_mut().unwrap().pop();
        }

        self.refresh_snippet_list();

        self.remain_in_keyword_search_mode()
    }

    fn drop_last_keyword(mut self) -> AppStateMode {
        debug_assert!(self.mode.keywords.len() > 0, "this vec should always contain at least one element");

        if self.mode.keywords.last().unwrap().is_empty() {
            self.mode.keywords.pop();
        }

        self.mode.keywords.pop();
        self.mode.keywords.push(String::new());

        self.refresh_snippet_list();

        self.remain_in_keyword_search_mode()
    }

    fn refresh_snippet_list(&mut self) {
        let snippet_ids = self.library.search(self.mode.keywords.iter().map(String::as_str), self.mode.active_tags.iter().map(|tag| tag.name.as_str()));
        self.mode.filtered_snippets = snippet_ids;

        if let Some(id) = self.mode.filtered_snippets.get(0) {
            self.mode.highlighted_snippet_index = Some(Cyclic::new(0, self.mode.filtered_snippets.len()));
            self.mode.shown_snippet = *id;
        }
        else {
            self.mode.highlighted_snippet_index = None
        }
    }

    fn highlight_previous_snippet(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = index.sub(1);
            self.mode.highlighted_snippet_index = Some(updated_index);
            self.mode.shown_snippet = self.mode.filtered_snippets[updated_index.value()];
        }

        self.remain_in_keyword_search_mode()
    }

    fn highlight_next_snippet(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = index.add(1);
            self.mode.highlighted_snippet_index = Some(updated_index);
            self.mode.shown_snippet = self.mode.filtered_snippets[updated_index.value()];
        }

        self.remain_in_keyword_search_mode()
    }

    fn highlight_first_snippet(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = index.set(0);
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_keyword_search_mode()
    }

    fn highlight_last_snippet(mut self) -> AppStateMode {
        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = index.set(index.modulo() - 1);
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_keyword_search_mode()
    }

    fn highlight_previous_page(mut self) -> AppStateMode {
        let page_size = self.mode.page_size.unwrap();

        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = {
                if index.value() < page_size {
                    index.set(0)
                }
                else {
                    index.sub(page_size)
                }
            };
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_keyword_search_mode()
    }

    fn highlight_next_page(mut self) -> AppStateMode {
        let page_size = self.mode.page_size.unwrap();

        if let Some(index) = self.mode.highlighted_snippet_index {
            let updated_index = {
                if index.value() + page_size < index.modulo() {
                    index.add(page_size)
                }
                else {
                    index.set(index.modulo() - 1)
                }
            };
            self.mode.highlighted_snippet_index = updated_index.into();
            self.mode.shown_snippet = self.mode.snippets[updated_index.value()];
        }

        self.remain_in_keyword_search_mode()
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.assert_invariant();

        let (search_bar_area, tag_list_area, snippet_list_area, snippet_viewer_area) = {
            let area = frame.area();
            let [ upper_area, search_bar_area ] = Layout::vertical([ Constraint::Fill(1), Constraint::Length(1) ]).areas(area);
            let [ tag_list_area, right_area ] = Layout::horizontal([ Constraint::Length(20), Constraint::Fill(1) ]).areas(upper_area);
            let [ snippet_list_area, snippet_viewer_area ] = Layout::vertical([ Constraint::Length(15), Constraint::Fill(1) ]).areas(right_area);

            (search_bar_area, tag_list_area, snippet_list_area, snippet_viewer_area)
        };

        let buffer = frame.buffer_mut();
        self.render_search_bar(search_bar_area, buffer);
        self.render_tag_list(tag_list_area, buffer);
        self.render_snippet_list(snippet_list_area, buffer);
        self.render_snippet(snippet_viewer_area, buffer);

        self.assert_invariant();
    }

    fn render_snippet(&mut self, area: Rect, buffer: &mut Buffer) {
        let library = &self.library;
        let snippet = library.snippet(self.mode.shown_snippet);
        let snippet_viewer = widgets::snippet_view::Widget::new(snippet, library);
        let snippet_viewer_state = &mut self.mode.snippet_viewer_state;

        ratatui::widgets::StatefulWidget::render(snippet_viewer, area, buffer, snippet_viewer_state);
    }

    fn render_tag_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let border = Block::new().borders(Borders::ALL).border_type(BorderType::Plain);
        let inner_area = border.inner(area);

        let tag_list = {
            let selected_tags = self.mode.active_tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();
            let listed_tags = self.mode.tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();

            widgets::tags_view::Widget::new(selected_tags.iter().copied(), listed_tags.iter().copied())
        };

        ratatui::widgets::Widget::render(border, area, buffer);
        ratatui::widgets::Widget::render(tag_list, inner_area, buffer);
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        self.mode.page_size = Some(area.height as usize);

        let snippet_list = {
            let items = self.mode.filtered_snippets.iter().map(|id| self.library.snippet(*id).description.as_str());

            widgets::description_list::Widget::new(items, false)
        };
        let snippet_list_state = &mut self.mode.snippet_list_state;
        snippet_list_state.select(self.mode.highlighted_snippet_index.map(|c| c.value()));

        ratatui::widgets::StatefulWidget::render(snippet_list, area, buffer, snippet_list_state);
    }

    fn render_search_bar(&mut self, area: Rect, buffer: &mut Buffer) {
        let prompt = self.mode.keywords.join(" ");
        let paragraph = ratatui::widgets::Paragraph::new(prompt).on_light_blue();

        paragraph.render(area, buffer);
    }

    fn remain_in_keyword_search_mode(self) -> AppStateMode {
        self.assert_invariant();

        self.into()
    }

    fn cancel_keyword_search(self) -> AppStateMode {
        self.assert_invariant();

        AppState {
            library: self.library,
            mode: mode::View::new(self.mode.snippets, self.mode.tags, self.mode.active_tags, self.mode.original_highlighted_snippet_index, self.mode.shown_snippet),
        }.into()
    }

    fn switch_to_view_mode(self) -> AppStateMode {
        self.assert_invariant();

        if let Some(_) = self.mode.highlighted_snippet_index {
            let shown_snippet_index = match self.find_snippet_in_list(self.mode.shown_snippet, &self.mode.snippets) {
                Some(index) => Cyclic::new(index, self.mode.snippets.len()),
                None => panic!("expected to be able to find shown snippet in list of snippets"),
            };

            AppState {
                library: self.library,
                mode: mode::View::new(self.mode.snippets, self.mode.tags, self.mode.active_tags, shown_snippet_index.into(), self.mode.shown_snippet),
            }.into()
        }
        else {
            self.remain_in_keyword_search_mode()
        }
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

impl From<AppState<mode::KeywordSearch>> for AppStateMode {
    fn from(state: AppState<mode::KeywordSearch>) -> Self {
        AppStateMode::KeywordSearch(state)
    }
}
