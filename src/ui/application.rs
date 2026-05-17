use std::{io, mem};

use ratatui::{DefaultTerminal, Frame, buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent}, layout::{Constraint, Layout, Rect}, style::Style, widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget}};

use crate::{snippets::{Library, snippets::Tag}, timing, ui::{Mode, widgets}, util::Cyclic};

pub struct Application {
    active_mode: AppStateMode,
}

impl Application {
    pub fn new(library: Library) -> Self {
        Application{
            active_mode: AppStateMode::initial(library)
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while self.active_mode.is_running() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match &mut self.active_mode {
            AppStateMode::Quit => panic!("should not happen"),
            AppStateMode::View(state) => state.draw(frame),
        }
        // let (_, duration) = timing::measure(|| self.mode.draw(frame));
        // log::info!("Rendering frame took {}ms", duration.as_millis());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        let active_mode = mem::replace(&mut self.active_mode, AppStateMode::Quit);
        self.active_mode = match active_mode {
            AppStateMode::Quit => panic!("should not happen"),
            AppStateMode::View(state) => state.handle_event(event),
        };
        Ok(())
    }
}

enum AppStateMode {
    Quit,
    View(AppState<mode::View>),
}

impl AppStateMode {
    pub fn initial(library: Library) -> Self {
        AppStateMode::View(AppState::initial(library))
    }

    pub fn is_running(&self) -> bool {
        match self {
            AppStateMode::Quit => false,
            _ => true,
        }
    }
}

struct AppState<Mode> {
    library: Library,
    mode: Mode,
    tag_based_filter: Vec<Tag>,
    keyword_based_filter: Vec<String>,
    listed_snippets: Vec<usize>,
    listed_tags: Vec<Tag>,
    shown_snippet: Cyclic,
}

mod mode {
    use crate::ui::widgets;

    pub struct View {
        pub snippet_list_state: widgets::description_list::State,
        pub snippet_viewer_state: widgets::snippet_view::SnippetViewState,
    }

    impl View {
        pub fn initial() -> Self {
            View {
                snippet_list_state: widgets::description_list::State::default(),
                snippet_viewer_state: widgets::snippet_view::SnippetViewState::new(),
            }
        }
    }
}

impl<Mode> AppState<Mode> {
    fn quit(self) -> AppStateMode {
        AppStateMode::Quit
    }

    fn assert_invariant(&self) {
        debug_assert_eq!(self.shown_snippet.modulo() as usize, self.listed_snippets.len());
    }
}

impl AppState<mode::View> {
    pub fn initial(library: Library) -> Self {
        let listed_tags = library.tags().clone();
        let listed_snippets = library.snippets().collect::<Vec<_>>();

        AppState {
            library,
            shown_snippet: Cyclic::new(0, listed_snippets.len() as u64),
            mode: mode::View::initial(),
            keyword_based_filter: Vec::new(),
            tag_based_filter: Vec::new(),
            listed_snippets,
            listed_tags,
        }
    }

    fn render_tag_list(&self, area: Rect, buffer: &mut Buffer) {
        let border = Block::new().borders(Borders::ALL).border_type(BorderType::Plain);
        let inner_area = border.inner(area);

        let tag_list = {
            let selected_tags = Vec::<&str>::new();
            let listed_tags = &self.listed_tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>();

            widgets::tags_view::TagsView::new(selected_tags.iter().copied(), listed_tags.iter().copied())
        };

        ratatui::widgets::Widget::render(border, area, buffer);
        ratatui::widgets::Widget::render(tag_list, inner_area, buffer);
    }

    fn render_snippet_list(&mut self, area: Rect, buffer: &mut Buffer) {
        let snippet_list = {
            let items = self.listed_snippets.iter().map(|id| self.library.snippet(*id).description.as_str());

            widgets::description_list::Widget::new(items, false)
        };
        let snippet_list_state = &mut self.mode.snippet_list_state;
        snippet_list_state.select(self.shown_snippet.into());

        ratatui::widgets::StatefulWidget::render(snippet_list, area, buffer, snippet_list_state);
    }

    fn render_snippet(&mut self, area: Rect, buffer: &mut Buffer) {
        let library = &self.library;
        let snippet = library.snippet(self.shown_snippet.into());
        let snippet_viewer = widgets::snippet_view::SnippetView::new(snippet, library);
        let snippet_viewer_state = &mut self.mode.snippet_viewer_state;

        ratatui::widgets::StatefulWidget::render(snippet_viewer, area, buffer, snippet_viewer_state);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.assert_invariant();

        let (tag_list_area, snippet_list_area, snippet_viewer_area) = {
            let area = frame.area();
            let [ tag_list_area, right_area ] = Layout::horizontal([ Constraint::Length(20), Constraint::Fill(1) ]).areas(area);
            let [ snippet_list_area, snippet_view_area ] = Layout::vertical([ Constraint::Length(15), Constraint::Fill(1) ]).areas(right_area);

            ( tag_list_area, snippet_list_area, snippet_view_area )
        };

        let buffer = frame.buffer_mut();
        self.render_tag_list(tag_list_area, buffer);
        self.render_snippet_list(snippet_list_area, buffer);
        self.render_snippet(snippet_viewer_area, buffer);

        self.assert_invariant();
    }

    pub fn handle_event(self, event: Event) -> AppStateMode {
        self.assert_invariant();

        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            _ => self.remain_in_view_mode()
        }
    }

    fn handle_key_event(self, event: KeyEvent) -> AppStateMode {
        if event.is_press() {
            match event.code {
                KeyCode::Char('q') => self.quit(),
                KeyCode::Up => self.highlight_previous_snippet(),
                KeyCode::Down => self.highlight_next_snippet(),
                _ => self.remain_in_view_mode(),
            }
        }
        else {
            self.remain_in_view_mode()
        }
    }

    fn remain_in_view_mode(self) -> AppStateMode {
        self.assert_invariant();

        self.into()
    }

    fn highlight_previous_snippet(mut self) -> AppStateMode {
        self.shown_snippet = self.shown_snippet.sub(1);

        self.remain_in_view_mode()
    }

    fn highlight_next_snippet(mut self) -> AppStateMode {
        self.shown_snippet = self.shown_snippet.add(1);

        self.remain_in_view_mode()
    }
}

impl From<AppState<mode::View>> for AppStateMode {
    fn from(state: AppState<mode::View>) -> Self {
        AppStateMode::View(state)
    }
}
