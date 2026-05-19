use ratatui::{style::{Color, Stylize}, text::{Line, Span}, widgets::{List, ListItem, Paragraph}};

use crate::snippets::{Library, snippets::Snippet};

pub struct Widget<'a> {
    library: &'a Library,
    snippet: &'a Snippet,
    lines: Vec<Line<'a>>,
}

impl<'a> Widget<'a> {
    pub fn new(library: &'a Library, snippet: &'a Snippet) -> Self {
        Widget { library, snippet, lines: Vec::new() }
    }

    fn render_caption(&mut self, caption: &'a str) {
       let line = Span::default().content(caption).into_left_aligned_line().underlined();

       self.lines.push(line);
    }

    fn render_pages(&mut self) {
        self.render_caption("Pages");

        for page in &self.snippet.pages {
            let page_caption = page.caption.as_ref().map(String::as_str).unwrap_or("untitled page");
            let line = Span::default().content(page_caption).into_left_aligned_line();
            self.lines.push(line);
        }
    }

    fn render_links(&mut self) {
        self.render_caption("Links");

        for link in &self.snippet.links {
            let description_of_linked_snippet = self.library.snippet(*link).description.as_str();
            let line = Span::default().content(description_of_linked_snippet).into_left_aligned_line();
            self.lines.push(line);
        }
    }

    fn render_blank_line(&mut self) {
        let line = Line::default();
        self.lines.push(line);
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(mut self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
        self.render_pages();
        self.render_blank_line();
        self.render_links();
        // pages
        // links
        // urls

        let paragraph = Paragraph::new(self.lines);
        paragraph.render(area, buffer);
    }
}
