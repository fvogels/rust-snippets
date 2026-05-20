use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::Stylize, widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding}};

use crate::{document::Code, snippets::snippets::WebLink};


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
        let block_title = " Copy snippet to clipboard ";

        let web_links = self.web_links;
        let list_items = web_links.iter().enumerate().map(|(index, web_link)| {
            let caption = &web_link.caption;
            let list_item_content = format!("[{index}] {caption}", index=index+1, caption=caption);

            ListItem::new(list_item_content)
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
