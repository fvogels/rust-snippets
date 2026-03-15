use std::{io, mem};

use ratatui::{DefaultTerminal, Frame, crossterm::event};

use crate::{snippets::Library, ui::Mode};

pub struct Application {
    mode: Mode,
}

impl Application {
    pub fn new(library: Library) -> Self {
        Application{
            mode: Mode::default(library),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.mode.is_running() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.mode.draw(frame)
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        let current_mode = mem::replace(&mut self.mode, Mode::Terminated);
        self.mode = current_mode.handle_event(event);
        Ok(())
    }
}
