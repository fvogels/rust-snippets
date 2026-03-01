use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{Style, Stylize}, widgets::{List, ListItem, ListState, StatefulWidget, Widget}};


pub struct TagsView<'a> {
    selected_tags: Vec<&'a str>,
    available_tags: Vec<&'a str>,
}

pub struct TagsViewState {
    available_tag_list_state: ListState,
}

impl TagsViewState {
    pub fn new() -> Self {
        TagsViewState{
            available_tag_list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn select_next(&mut self) {
        self.available_tag_list_state.select_next();
    }

    pub fn select_previous(&mut self) {
        self.available_tag_list_state.select_previous();
    }

    pub fn ensure_selection(&mut self) {
        if self.available_tag_list_state.selected().is_none() {
            self.available_tag_list_state = self.available_tag_list_state.with_selected(Some(0));
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.available_tag_list_state.selected()
    }
}

impl<'a> TagsView<'a> {
    pub fn new(selected_tags: impl Iterator<Item=&'a str>, available_tags: impl Iterator<Item=&'a str>) -> Self {
        TagsView{
            selected_tags: selected_tags.collect(),
            available_tags: available_tags.collect(),
        }
    }
}

impl<'a> StatefulWidget for TagsView<'a> {
    type State = TagsViewState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let [selected_tags_area, available_tags_area] = Layout::vertical([Constraint::Length(self.selected_tags.len().try_into().unwrap()), Constraint::Fill(1)]).areas(area);

        let selected_tags_items = self.selected_tags.iter().copied().map(|tag| ListItem::new(tag).white().on_blue());
        let selected_tags_list = List::new(selected_tags_items);

        let available_tags_items = self.available_tags.iter().copied().map(|tag| ListItem::new(tag));
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        let available_tags_list = List::new(available_tags_items).highlight_style(highlight_style);

        Widget::render(selected_tags_list, selected_tags_area, buffer);
        StatefulWidget::render(available_tags_list, available_tags_area, buffer, &mut state.available_tag_list_state);
    }
}
