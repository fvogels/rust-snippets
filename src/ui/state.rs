use std::mem;

use ratatui::{Frame, crossterm::event::{Event}};

use crate::{snippets::Library, ui::{search_mode::SearchMode, syntax::SyntaxHighlighter, tag_search_mode::TagSearchMode, view_mode::ViewMode}};


pub struct State {
    mode: Mode,
}

pub(super) enum Mode {
    View(ViewMode),
    Search(SearchMode),
    TagSearch(TagSearchMode),
    Terminated,
}

impl State {
    pub fn new(library: Library) -> Self {
        let boxed_library = Box::new(library);
        let boxed_highlighter = Box::new(SyntaxHighlighter::new());

        State{
            mode: Mode::View(ViewMode::new(boxed_library, boxed_highlighter)),
        }
    }

    pub fn is_running(&self) -> bool {
        match self.mode {
            Mode::Terminated => false,
            _ => true,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match &mut self.mode {
            Mode::Terminated => panic!("should never occur"),
            Mode::View(view_mode) => view_mode.draw(frame),
            Mode::Search(search_mode) => search_mode.draw(frame),
            Mode::TagSearch(tag_search_mode) => tag_search_mode.draw(frame),
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        let current_mode = mem::replace(&mut self.mode, Mode::Terminated);

        self.mode = match current_mode {
            Mode::Terminated => panic!("should never occur"),
            Mode::View(view_mode) => view_mode.handle_event(event),
            Mode::Search(search_mode) => search_mode.handle_event(event),
            Mode::TagSearch(tag_search_mode) => tag_search_mode.handle_event(event),
        }
    }
}
