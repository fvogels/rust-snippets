mod state;
mod view_mode;
mod search_mode;
mod tag_search_mode;
mod widgets;
mod application;

use state::Mode;
use application::Application;

use crate::{snippets::Library};

pub fn start_ui(library: Library) -> anyhow::Result<()> {
    ratatui::run(|terminal| Application::new(library).run(terminal))
}
