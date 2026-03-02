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
        Mode::View(ViewMode::new(library, SyntaxHighlighter::new()))
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
    pub tag_input: Option<String>,
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
            tag_input: None,
        }
    }

    pub fn pop_keyword(&mut self) -> Option<String> {
        self.keywords.pop()
    }

    pub fn clear_keywords(&mut self) {
        self.keywords.clear();
    }

    pub fn select_tag(&mut self, tag: String) {
        self.selected_tags.push(tag);
        self.tag_input = None;
    }

    pub fn pop_selected_tag(&mut self) {
        self.selected_tags.pop();
    }

    pub fn refresh(&mut self) {
        self.update_visible_snippets();
        self.update_visible_tags();
    }

    fn update_visible_snippets(&mut self) {
        self.visible_snippets = self.library.search(self.keywords.iter().map(String::as_str), self.selected_tags.iter().map(String::as_str));
    }

    fn update_visible_tags(&mut self) {
        let mut tags: HashSet<&str> = HashSet::new();

        // Find the union of the tags of all visible snippets
        for snippet_id in self.visible_snippets.iter().copied() {
            let snippet = self.library.snippet(snippet_id);
            let snippet_tags = &snippet.tags;

            for snippet_tag in snippet_tags.iter() {
                if let Some(prefix) = &self.tag_input {
                    if snippet_tag.starts_with(prefix.as_str()) {
                        tags.insert(snippet_tag.as_str());
                    }
                }
                else {
                    tags.insert(snippet_tag.as_str());
                }
            }
        }

        // Remove the already selected tags
        for selected_tag in self.selected_tags.iter() {
            tags.remove(selected_tag.as_str());
        }

        self.visible_tags = tags.into_iter().map(String::from).collect();
        self.visible_tags.sort();
    }

    pub fn visible_snippet_descriptions<'a>(&'a self) -> impl Iterator<Item=&'a str> {
        self.visible_snippets.iter().copied().map(|id| self.library.snippet(id).description.as_str())
    }
}
