use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}};

use crate::{document::{self, Document}, snippets::snippets::{Part, Snippet}, ui::syntax::SyntaxHighlighter};

pub struct SnippetView<'a> {
    snippet: &'a Snippet,
    syntax_highlighter: &'a SyntaxHighlighter,
}

pub struct SnippetViewState {
    selected_part: usize,
}

impl SnippetViewState {
    pub fn new() -> Self {
        SnippetViewState{
            selected_part: 0,
        }
    }

    pub fn select_next(&mut self) {
        self.selected_part += 1
    }

    pub fn select_previous(&mut self) {
        if self.selected_part >= 1 {
            self.selected_part -= 1
        }
    }

    fn ensure_within_bounds(&mut self, part_count: usize) {
        if self.selected_part >= part_count {
            self.selected_part = 0
        }
    }

    pub fn selected(&self) -> usize {
        self.selected_part
    }
}

impl<'a> SnippetView<'a> {
    pub fn new(snippet: &'a Snippet, syntax_highlighter: &'a SyntaxHighlighter) -> Self {
        SnippetView{
            snippet,
            syntax_highlighter,
        }
    }

    fn selected_snippet_part(&self, state: &mut SnippetViewState) -> (usize, &'a Part) {
        let snippet = self.snippet;

        state.ensure_within_bounds(snippet.parts.len());
        let selected_part_index = state.selected_part;

        (selected_part_index, &snippet.parts[selected_part_index])
    }

    fn render_document_as_paragraph<'b>(&self, document: &'b Document, line_width: usize) -> Paragraph<'b> {
        let mut lines = Vec::new();

        for fragment in document {
            match fragment {
                document::Fragment::Wrapping(words) => {
                    let mut spans = Vec::new();
                    let mut acc = 0;

                    for word in words {
                        let is_fresh_line = acc == 0;
                        let separator_size = if is_fresh_line { 0 } else { 1 };
                        if acc + word.len() + separator_size > line_width && acc > 0 {
                            // New line has to be started
                            let line = Line::default().spans(spans);
                            lines.push(line);
                            spans = Vec::new();
                        }
                        else {
                            // Word fits on current line
                            if !is_fresh_line {
                                spans.push(Span::default().content(" "))
                            }
                            for span in word.spans() {
                                spans.push(translate_span(span));
                            }
                            acc += word.len();
                        }
                    }

                    if !spans.is_empty() {
                        let line = Line::default().spans(spans);
                        lines.push(line);
                    }
                },
                &document::Fragment::Code { ref language, lines: ref code_lines } => {
                    let highlighted_lines = self.syntax_highlighter.highlight_lines(language.as_deref(), code_lines.iter().map(String::as_str));

                    for line in highlighted_lines {
                        lines.push(line);
                    }
                }
            }
        }

        Paragraph::new(lines)
    }
}

impl<'a> StatefulWidget for SnippetView<'a> {
    type State = SnippetViewState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let (selected_part_index, selected_part) = self.selected_snippet_part(state);
        let one_based_index = selected_part_index + 1;
        let part_count = self.snippet.parts.len();
        let part_caption = match selected_part.caption() {
            Some(caption) => format!(" {}/{} {} ", one_based_index, part_count, caption),
            None => format!(" {}/{} ", one_based_index, part_count),
        };
        let bottom_title = Line::raw(part_caption);
        let mut snippet_caption_block = Block::new().title_bottom(bottom_title).borders(Borders::ALL);
        if let Some(language) = selected_part.language() {
            snippet_caption_block = snippet_caption_block.title_top(language);
        }

        let document = &selected_part.contents;
        let line_width = snippet_caption_block.inner(area).width as usize;
        let paragraph = self.render_document_as_paragraph(document, line_width).block(snippet_caption_block);

        paragraph.render(area, buffer);
    }
}

fn translate_color(color: &document::Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(color.r, color.g, color.b)
}

fn translate_style(style: &document::Style) -> ratatui::style::Style {
    let mut result = ratatui::style::Style::default();

    if let Some(foreground_color) = style.foreground {
        result = result.fg(translate_color(&foreground_color));
    }

    if let Some(background_color) = style.background {
        result = result.bg(translate_color(&background_color));
    }

    if style.bold {
        result = result.bold();
    }

    if style.underline {
        result = result.underlined();
    }

    if style.italic {
        result = result.italic();
    }

    result
}

fn translate_span(span: &document::Span) -> ratatui::text::Span {
    ratatui::text::Span::default().content(span.text.as_str()).style(translate_style(&span.style))
}