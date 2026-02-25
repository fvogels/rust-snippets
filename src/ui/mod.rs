mod state;

use state::State;

use std::io;

use ratatui::{DefaultTerminal, Frame, crossterm::event};

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
    state: State,
}

impl Application {
    fn new() -> Self {
        Application{
            state: State::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.state.is_running() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.state.draw(frame)
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        self.state.handle_event(event);
        Ok(())
    }
}
