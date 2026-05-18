use ratatui::{buffer::Buffer, layout::Rect, style::Style, text::Line, widgets::{Block, BorderType, Borders, List, ListState}};


pub struct Widget<'a> {
    has_focus: bool,
    items: Vec<&'a str>,
}

pub struct State {
    list_state: ListState,
}

impl State {
    pub fn default() -> Self {
        State {
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }
}

impl<'a> Widget<'a> {
    pub fn new(items: impl Iterator<Item=&'a str>, has_focus: bool) -> Self {
        Widget{
            has_focus,
            items: items.collect(),
        }
    }
}

impl<'a> ratatui::widgets::StatefulWidget for Widget<'a> {
    type State = State;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut State) {
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        let descriptions = self.items;
        let list_block = {
            let title = Line::raw("Snippets");
            let bottom_title = {
                if let Some(selected) = state.list_state.selected() {
                    // checked_add is necessary: jumping to the last element sets the selected index to the maximum value, and doing +1 on this causes a panic
                    Line::raw(format!(" Snippet {}/{} ", selected.checked_add(1).unwrap_or(descriptions.len()), descriptions.len())).right_aligned()
                }
                else {
                    Line::raw(format!(" {} snippets ", descriptions.len())).right_aligned()
                }
            };
            Block::new().title(title).borders(Borders::ALL).title_bottom(bottom_title).border_type(BorderType::Double)
        };

        let list = List::new(descriptions).highlight_style(highlight_style).block(list_block);

        ratatui::widgets::StatefulWidget::render(list, area, buffer, &mut state.list_state);
    }
}


impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let highlight_style = Style::new().bg(ratatui::style::Color::LightGreen);
        let descriptions = self.items;
        let list_block = {
            let title = Line::raw("Snippets");
            let bottom_title = {
                Line::raw(format!(" {} snippets ", descriptions.len())).right_aligned()
            };
            let block = Block::new().title(title).borders(Borders::ALL).title_bottom(bottom_title);

            if self.has_focus {
                block.border_type(BorderType::Double)
            }
            else {
                block.border_type(BorderType::Plain)
            }
        };

        let list = List::new(descriptions).highlight_style(highlight_style).block(list_block);

        ratatui::widgets::Widget::render(list, area, buffer);
    }
}