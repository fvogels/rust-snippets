use ratatui::widgets::ListState;

pub struct State {
    pub exit: bool,
    pub description_list: Vec<String>,
    pub description_list_state: ListState,
    pub mode: Mode,
}

pub enum Mode {
    ViewMode,
    SearchMode,
}

impl State {
    pub fn new() -> Self {
        State {
            exit: false,
            description_list: vec!["a", "b", "c", "d"].into_iter().map(|s| s.to_owned()).collect(),
            description_list_state: ListState::default().with_selected(Some(0)),
            mode: Mode::ViewMode,
        }
    }

    pub fn select_next(&mut self) {
        self.description_list_state.select_next()
    }

    pub fn select_previous(&mut self) {
        self.description_list_state.select_previous()
    }
}