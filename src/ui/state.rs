use ratatui::{Frame, crossterm::event::{Event}};

use crate::{snippets::Library, ui::{syntax::SyntaxHighlighter, viewmode::ViewMode}};


pub struct State {
    mode: Box<dyn Mode>,
}

impl State {
    pub fn new(library: Library) -> Self {
        let boxed_library = Box::new(library);
        let boxed_highlighter = Box::new(SyntaxHighlighter::new());

        State{
            mode: Box::new(ViewMode::new(boxed_library, boxed_highlighter)),
        }
    }

    pub fn is_running(&self) -> bool {
        !self.mode.exit()
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.mode.draw(frame);
    }

    pub fn handle_event(&mut self, event: Event) {
        self.mode.handle_event(event);
    }
}

pub(super) trait Mode {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_event(&mut self, event: Event);
    fn exit(&self) -> bool;
}
