use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{Style, Stylize}, widgets::{List, ListItem, ListState}};


pub struct Widget<'a> {
    selected_tags: Vec<&'a str>,
    available_tags: Vec<&'a str>,
}

pub struct State {
    available_tag_list_state: ListState,
}

impl State {
    pub fn new() -> Self {
        State{
            available_tag_list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.available_tag_list_state.select(index);
    }
}

impl<'a> Widget<'a> {
    pub fn new(selected_tags: impl Iterator<Item=&'a str>, available_tags: impl Iterator<Item=&'a str>) -> Self {
        Widget{
            selected_tags: selected_tags.collect(),
            available_tags: available_tags.collect(),
        }
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [selected_tags_area, available_tags_area] = Layout::vertical([Constraint::Length(self.selected_tags.len().try_into().unwrap()), Constraint::Fill(1)]).areas(area);

        let selected_tags_items = self.selected_tags.iter().copied().map(|tag| ListItem::new(tag).white().on_blue());
        let selected_tags_list = List::new(selected_tags_items);

        let available_tags_items = self.available_tags.iter().copied().map(|tag| ListItem::new(tag));
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        let available_tags_list = List::new(available_tags_items).highlight_style(highlight_style);

        ratatui::widgets::Widget::render(selected_tags_list, selected_tags_area, buffer);
        ratatui::widgets::Widget::render(available_tags_list, available_tags_area, buffer);
    }
}


impl<'a> ratatui::widgets::StatefulWidget for Widget<'a> {
    type State = State;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let [selected_tags_area, available_tags_area] = Layout::vertical([Constraint::Length(self.selected_tags.len().try_into().unwrap()), Constraint::Fill(1)]).areas(area);

        let selected_tags_items = self.selected_tags.iter().copied().map(|tag| ListItem::new(tag).white().on_blue());
        let selected_tags_list = List::new(selected_tags_items);

        let available_tags_items = self.available_tags.iter().copied().map(|tag| ListItem::new(tag));
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        let available_tags_list = List::new(available_tags_items).highlight_style(highlight_style);

        ratatui::widgets::Widget::render(selected_tags_list, selected_tags_area, buffer);
        ratatui::widgets::StatefulWidget::render(available_tags_list, available_tags_area, buffer, &mut state.available_tag_list_state);
    }
}
