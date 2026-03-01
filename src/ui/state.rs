use std::{collections::HashSet, mem};

use ratatui::{Frame, crossterm::event::{Event}};

use crate::{snippets::Library, ui::{search_mode::SearchMode, syntax::SyntaxHighlighter, tag_search_mode::TagSearchMode, view_mode::ViewMode}};


pub(super) enum Mode {
    View(ViewMode),
    Search(SearchMode),
    TagSearch(TagSearchMode),
    Terminated,
}

impl Mode {
    pub fn default(library: Library) -> Self {
        Mode::View(ViewMode::new(Box::new(library), Box::new(SyntaxHighlighter::new())))
    }

    pub fn is_running(&self) -> bool {
        match self {
            Mode::Terminated => false,
            _ => true,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match self {
            Mode::Terminated => panic!("should never occur"),
            Mode::View(view_mode) => view_mode.draw(frame),
            Mode::Search(search_mode) => search_mode.draw(frame),
            Mode::TagSearch(tag_search_mode) => tag_search_mode.draw(frame),
        }
    }

    pub fn handle_event(self, event: Event) -> Mode {
        match self {
            Mode::Terminated => panic!("should never occur"),
            Mode::View(view_mode) => view_mode.handle_event(event),
            Mode::Search(search_mode) => search_mode.handle_event(event),
            Mode::TagSearch(tag_search_mode) => tag_search_mode.handle_event(event),
        }
    }
}

pub struct State {
    pub library: Library,
    pub syntax_highlighter: SyntaxHighlighter,
    pub selected_tags: Vec<String>,
    pub keywords: Vec<String>,
    pub visible_snippets: Vec<usize>,
    pub visible_tags: Vec<String>,
}

impl State {
    pub fn new(library: Library, syntax_highlighter: SyntaxHighlighter) -> State {
        State {
            visible_snippets: library.snippets().collect(),
            visible_tags: library.tags().clone(),
            library: library,
            syntax_highlighter: syntax_highlighter,
            selected_tags: Vec::new(),
            keywords: Vec::new(),
        }
    }

    pub fn add_keyword(&mut self, keyword: String) {
        self.keywords.push(keyword);
    }

    pub fn select_tag(&mut self, tag: String) {
        self.selected_tags.push(tag);
    }

    pub fn refresh(&mut self) {
        self.update_visible_snippets();
        self.update_visible_tags();
    }

    fn update_visible_snippets(&mut self) {
        self.visible_snippets = self.library.search(self.keywords.iter().map(String::as_str), self.selected_tags.iter().map(String::as_str));
    }

    fn update_visible_tags(&mut self) {
        let mut tags: HashSet<String> = HashSet::new();

        for snippet_id in self.visible_snippets.iter().copied() {
            let snippet = self.library.snippet(snippet_id);
            let snippet_tags = &snippet.tags;

            tags.retain(|tag| snippet_tags.contains(tag));
        }

        for selected_tag in self.selected_tags.iter() {
            tags.remove(selected_tag);
        }

        self.visible_tags = tags.into_iter().collect();
        self.visible_tags.sort();
    }
}
