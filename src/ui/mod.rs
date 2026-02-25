mod state;

use state::State;

use std::io;

use ratatui::{DefaultTerminal, Frame, buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent}, layout::Rect, text::Line, widgets::{Block, List, ListItem, StatefulWidget, Widget}};

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
        while !self.state.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.is_press() => self.handle_key_event(key_event),
            _ => Ok(())
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => {
                self.state.select_previous();
                Ok(())
            },
            KeyCode::Down => {
                self.state.select_next();
                Ok(())
            },
            _ => Ok(())
        }
    }

    fn exit(&mut self) -> io::Result<()> {
        self.state.exit = true;
        Ok(())
    }
}

impl Widget for &mut Application {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // let xs = Vec::<i32>::new();
        // Block::bordered().title(Line::from("Title")).render(area, buffer)

        let items = self.state.description_list.iter().map(|item| ListItem::from(item.as_str())).collect::<Vec<_>>();
        let list = List::new(items).highlight_spacing(ratatui::widgets::HighlightSpacing::Always).highlight_symbol(">");

        StatefulWidget::render(list, area, buffer, &mut self.state.description_list_state);
    }
}
