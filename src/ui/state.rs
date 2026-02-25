use ratatui::{Frame, buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent}, layout::Rect, widgets::{List, ListItem, ListState, StatefulWidget, Widget}};


pub struct State {
    mode: Box<dyn Mode>,
}

impl State {
    pub fn new() -> Self {
        State{
            mode: Box::new(ViewMode::new()),
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

trait Mode {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_event(&mut self, event: Event);
    fn exit(&self) -> bool;
}

struct ViewMode {
    exit: bool,
    description_list: Vec<String>,
    description_list_state: ListState,
}

impl ViewMode {
    fn new() -> Self {
        ViewMode {
            exit: false,
            description_list: vec!["a", "b", "c", "d"].into_iter().map(|x| x.to_owned()).collect(),
            description_list_state: ListState::default().with_selected(Some(0))
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit = true
            },
            KeyCode::Up => {
                self.description_list_state.select_previous();
            },
            KeyCode::Down => {
                self.description_list_state.select_next();
            },
            _ => { }
        }
    }
}

impl Mode for ViewMode {
    fn exit(&self) -> bool {
        return self.exit
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) if key_event.is_press() => self.handle_key_event(key_event),
            _ => {},
        }
    }
}

impl Widget for &mut ViewMode {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let items = self.description_list.iter().map(|item| ListItem::from(item.as_str())).collect::<Vec<_>>();
        let list = List::new(items).highlight_spacing(ratatui::widgets::HighlightSpacing::Always).highlight_symbol(">");

        StatefulWidget::render(list, area, buffer, &mut self.description_list_state);
    }
}
