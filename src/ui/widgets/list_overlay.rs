use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::Stylize, text::{Line, Span, Text}, widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding}};


pub struct Widget<'a> {
    title: &'a str,
    lines: Vec<Line<'a>>,
}

impl<'a> Widget<'a> {
    pub fn new(title: &'a str, lines: Vec<Line<'a>>) -> Self {
        Widget{
            title,
            lines,
        }
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block_title = self.title;

        let list_items = self.lines.into_iter().enumerate().map(|(index, mut line)| {
            let index_string = format!("[{}] ", index + 1);
            let index_span = Span::default().content(index_string);
            line.spans.insert(0, index_span);

            let mut text = Text::default();
            text.push_line(line);
            ListItem::new(text)
        }).collect::<Vec<_>>();
        let longest_list_item = list_items.iter().map(ListItem::width).max().unwrap();
        let required_width = *[longest_list_item + 4, block_title.len() + 4].iter().max().unwrap();
        let required_height = list_items.len() + 4;

        let overlay_area = area.centered(Constraint::Length(required_width as u16), Constraint::Length(required_height as u16));
        let block = Block::new().borders(Borders::ALL).border_type(BorderType::Double).title(block_title).padding(Padding::uniform(1)).on_dark_gray();
        let block_inner_area = block.inner(overlay_area);
        Clear::default().render(overlay_area, buffer);
        block.render(overlay_area, buffer);

        let list = List::new(list_items);

        list.render(block_inner_area, buffer);
    }
}
