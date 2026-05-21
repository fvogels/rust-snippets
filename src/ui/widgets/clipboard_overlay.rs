use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::Stylize, text::Line, widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding}};

use crate::{document::Code, ui::widgets};


pub struct Widget<'a> {
    code_blocks: Vec<&'a Code>,
}

impl<'a> Widget<'a> {
    pub fn new(code_blocks: Vec<&'a Code>) -> Self {
        Widget{
            code_blocks
        }
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block_title = " Copy snippet to clipboard ";

        let lines = self.code_blocks.into_iter().map(|code| {
            let language = code.language.as_ref().map(String::as_str).unwrap_or("unknown language");
            let caption = code.metadata.as_ref().map(String::as_str).unwrap_or("Code snippet");
            let line = format!("{caption} ({language})", caption=caption, language=language);

            Line::raw(line)
        }).collect::<Vec<_>>();

        let widget = widgets::list_overlay::Widget::new(block_title, lines);
        widget.render(area, buffer);
    }
}
