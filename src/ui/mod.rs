mod state;
mod view_mode;
mod search_mode;
mod tag_search_mode;
mod syntax;
mod widgets;
mod tree_adapter;
mod search_parameters;

pub use search_parameters::SearchParameters;

use state::Mode;

use std::{io, mem, path::PathBuf};

use ratatui::{DefaultTerminal, Frame, crossterm::event};

use crate::snippets::Library;

pub fn start_ui() {
    match ratatui::run(|terminal| Application::new().run(terminal)) {
        Ok(()) => {
            println!("TUI successfully terminated")
        },
        Err(err) => {
            println!("An error occurred: {}", err)
        }
    }
}

struct Application {
    mode: Mode,
}

impl Application {
    fn new() -> Self {
        let path = PathBuf::from("../data/snippets");
        let library = Library::load(&path).unwrap();

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
        let mut current_mode = mem::replace(&mut self.mode, Mode::Terminated);
        self.mode = current_mode.handle_event(event);
        Ok(())
    }
}
