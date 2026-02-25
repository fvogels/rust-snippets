use ratatui::{Frame, buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent}, layout::Rect, style::{Modifier, Style, palette::tailwind::SLATE}, widgets::{List, ListItem, ListState, StatefulWidget, Widget}};

use crate::{snippets::Library, ui::state::Mode};

pub(super) struct ViewMode {
    exit: bool,
    library: Box<Library>,
    description_list_state: ListState,
}

impl ViewMode {
    pub(super) fn new(library: Box<Library>) -> Self {
        ViewMode {
            exit: false,
            description_list_state: ListState::default().with_selected(Some(0)),
            library: library,
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
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen).add_modifier(Modifier::BOLD);
        let descriptions = self.library.snippets().iter().map(|item| ListItem::new(item.description.as_str()) );
        let list = List::new(descriptions).highlight_style(highlight_style);

        StatefulWidget::render(list, area, buffer, &mut self.description_list_state);
    }
}
