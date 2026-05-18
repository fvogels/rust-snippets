mod widgets;
mod application;

use application::Application;

use crate::{snippets::Library};

pub fn start_ui(library: Library) -> anyhow::Result<()> {
    ratatui::run(|terminal| Application::new(library).run(terminal))
}
