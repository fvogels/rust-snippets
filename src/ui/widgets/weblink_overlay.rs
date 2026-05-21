use ratatui::{buffer::Buffer, layout::Rect, text::Line};

use crate::{snippets::snippets::WebLink, ui::widgets};


pub struct Widget<'a> {
    web_links: &'a Vec<WebLink>
}

impl<'a> Widget<'a> {
    pub fn new(web_links: &'a Vec<WebLink>) -> Self {
        Widget{
            web_links,
        }
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block_title = " Web links ";
        let lines = self.web_links.iter().map(|web_link| {
            Line::raw(web_link.caption.as_str())
        }).collect();

        let widget = widgets::list_overlay::Widget::new(block_title, lines);
        widget.render(area, buffer);
    }
}
