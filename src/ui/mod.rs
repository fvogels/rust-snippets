mod state;
mod view_mode;
mod search_mode;
mod tag_search_mode;
mod syntax;
mod widgets;

use state::Mode;

use std::{io, mem, path::PathBuf};

use ratatui::{DefaultTerminal, Frame, crossterm::event};

use crate::{snippets::Library, ui::syntax::SyntaxHighlighter};

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
        let syntax_highlighter = create_syntax_highlighter();

        Application{
            mode: Mode::default(library, syntax_highlighter),
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

fn create_syntax_highlighter() -> SyntaxHighlighter {
    let mut syntax_highlighter = SyntaxHighlighter::new();

    syntax_highlighter.add_alias("bash", "Bourne Again Shell (bash)");

    for language in syntax_highlighter.supported_languages() {
        syntax_highlighter.add_alias(language.to_lowercase().as_str(), language.as_str());
    }

    syntax_highlighter
}