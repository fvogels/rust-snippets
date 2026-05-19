use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::Stylize, widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding}};

use crate::document::{Code};


pub struct Widget<'a> {
    code_blocks: &'a Vec<&'a Code>,
}

impl<'a> Widget<'a> {
    pub fn new(code_blocks: &'a Vec<&'a Code>) -> Self {
        Widget{
            code_blocks
        }
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block_caption = " Copy snippet to clipboard ";

        let code_blocks = self.code_blocks;
        let list_items = code_blocks.iter().enumerate().map(|(index, code)| {
            let language = code.language.as_ref().map(String::as_str).unwrap_or("unknown language");
            let caption = code.metadata.as_ref().map(String::as_str).unwrap_or("Code snippet");
            let list_item_content = format!("[{index}] {caption} ({language})", index=index+1, caption=caption, language=language);

            ListItem::new(list_item_content)
        }).collect::<Vec<_>>();
        let longest_list_item = list_items.iter().map(ListItem::width).max().unwrap();
        let required_width = *[longest_list_item + 4, block_caption.len() + 4].iter().max().unwrap();
        let required_height = list_items.len() + 4;

        let overlay_area = area.centered(Constraint::Length(required_width as u16), Constraint::Length(required_height as u16));
        let block = Block::new().borders(Borders::ALL).border_type(BorderType::Double).title(" Copy snippet to clipboard ").padding(Padding::uniform(1)).on_dark_gray();
        let block_inner_area = block.inner(overlay_area);
        Clear::default().render(overlay_area, buffer);
        block.render(overlay_area, buffer);

        let list = List::new(list_items);

        list.render(block_inner_area, buffer);
    }
}
