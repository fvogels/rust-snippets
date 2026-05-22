use ratatui::{buffer::Buffer, layout::{Margin, Rect}, style::{Style, Stylize}, text::Line, widgets::{Block, Borders, Padding}};

use crate::{snippets::snippets::{Page, Snippet, WebLink}, ui::widgets};


pub struct Widget<'a> {
    page: &'a Page,
}

impl<'a> Widget<'a> {
    pub fn new(page: &'a Page) -> Self {
        Widget{
            page
        }
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let area = area.inner(Margin::new(4, 4));
        ratatui::widgets::Clear::default().render(area, buffer);

        let block = Block::default().border_type(ratatui::widgets::BorderType::Double).borders(Borders::ALL).padding(Padding::uniform(1));
        let document_area = block.inner(area);
        block.render(area, buffer);

        let document = self.page.document();
        let document_viewer = widgets::document_view::Widget::new(document);
        document_viewer.render(document_area, buffer);
    }
}
