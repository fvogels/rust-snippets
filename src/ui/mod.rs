mod state;
mod view_mode;
mod search_mode;
mod tag_search_mode;
mod widgets;
mod application;

use state::Mode;
use application::Application;

use std::{io, mem};

use ratatui::{DefaultTerminal, Frame, crossterm::event};

use crate::{snippets::Library};

pub fn start_ui(library: Library) {
    match ratatui::run(|terminal| Application::new(library).run(terminal)) {
        Ok(()) => {
            println!("TUI successfully terminated")
        },
        Err(err) => {
            println!("An error occurred: {}", err)
        }
    }
}
