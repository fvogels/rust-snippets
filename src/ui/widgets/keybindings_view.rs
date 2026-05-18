use itertools::enumerate;
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::{Line, Span}, widgets::Paragraph};


pub struct Widget {
    bindings: Vec<Binding>,
}

pub struct Binding {
    key: String,
    description: String,
}

impl Binding {
    pub fn new(key: &str, description: &str) -> Self {
        Binding { key: key.to_owned(), description: description.to_owned() }
    }
}

impl Widget {
    pub fn new(bindings: Vec<Binding>) -> Self {
        Widget{
            bindings,
        }
    }
}

impl ratatui::widgets::Widget for Widget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let mut spans = Vec::new();
        let separator_span = Span::default().content(" ");

        for (index, binding) in enumerate(self.bindings.into_iter()) {
            let key = format!(" {} ", binding.key);
            let description = format!(" {} ", binding.description);

            let key_span = Span::default().content(key).on_light_blue();
            let description_span = Span::default().content(description).on_dark_gray();

            if index > 0 {
                spans.push(separator_span.clone());
            }

            spans.push(key_span);
            spans.push(description_span);
        }

        let line = Line::default().spans(spans);
        let paragraph = Paragraph::new(line);

        paragraph.render(area, buffer);
    }
}
