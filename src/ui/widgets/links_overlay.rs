use ratatui::{buffer::Buffer, layout::Rect, text::Line};

use crate::ui::widgets;


pub struct Widget<'a> {
    links: &'a Vec<String>,
}

impl<'a> Widget<'a> {
    pub fn new(links: &'a Vec<String>) -> Self {
        Widget{
            links
        }
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block_title = " Linked snippets ";
        let lines = self.links.iter().map(|link| {
            Line::raw(link)
        }).collect();

        let widget = widgets::list_overlay::Widget::new(block_title, lines);
        widget.render(area, buffer);
    }
}
