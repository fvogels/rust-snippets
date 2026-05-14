use ratatui::{style::{Color, Stylize}, widgets::{List, ListItem}};

pub struct Widget {
    categories: Vec<Category>,
}

pub struct Category {
    pub caption: String,
    pub entries: Vec<String>,
}

impl Widget {
    pub fn new(categories: Vec<Category>) -> Self {
        Widget { categories }
    }
}

impl ratatui::widgets::Widget for Widget {
    fn render(self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
        let mut list_items = Vec::new();

        for category in self.categories {
            if !list_items.is_empty() {
                list_items.push(ListItem::new(""));
            }

            let caption_item = ListItem::new(category.caption).underlined().bg(Color::Rgb(96, 96, 255));
            list_items.push(caption_item);

            for entry in category.entries {
                let entry_item = ListItem::new(entry);
                list_items.push(entry_item);
            }
        }

        let list = List::new(list_items).bg(ratatui::style::Color::Rgb(16, 16, 16));
        list.render(area, buffer);
    }
}
