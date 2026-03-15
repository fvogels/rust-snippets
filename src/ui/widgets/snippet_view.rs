use std::borrow::Cow;

use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}};

use crate::{document::{self, Document}, snippets::snippets::{Part, Snippet}};

pub struct SnippetView<'a> {
    snippet: &'a Snippet,
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
    pub fn new(snippet: &'a Snippet) -> Self {
        SnippetView{
            snippet,
        }
    }

    fn selected_snippet_part(&self, state: &mut SnippetViewState) -> (usize, &'a Part) {
        let snippet = self.snippet;

        state.ensure_within_bounds(snippet.parts.len());
        let selected_part_index = state.selected_part;

        (selected_part_index, &snippet.parts[selected_part_index])
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
        let paragraph = render_document_as_paragraph(document, line_width).block(snippet_caption_block);

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

    if let Some(b) = style.bold {
        if b {
            result = result.bold();
        }
        else {
            result = result.not_bold();
        }
    }

    if let Some(b) = style.underline {
        if b {
            result = result.underlined();
        }
        else {
            result = result.not_underlined();
        }
    }

    if let Some(b) = style.italic {
        if b {
            result = result.italic();
        }
        else {
            result = result.not_italic();
        }
    }

    result
}

fn translate_span<'a>(span: &document::Span, style_base: &document::Style) -> ratatui::text::Span<'a> {
    let combined_style = span.style.combine(style_base);

    ratatui::text::Span::default().content(Cow::from(span.text.clone())).style(translate_style(&combined_style))
}

fn render_document_as_paragraph<'a>(document: &Document, line_width: usize) -> Paragraph<'a> {
    let mut lines = Vec::new();
    let mut code_block_index = 1;

    for fragment in document {
        match fragment {
            document::Fragment::Wrapping{words, style} => {
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
                            // Add separating space between words
                            spans.push(Span::default().content(" ").style(translate_style(style)))
                        }
                        for span in word.spans() {
                            spans.push(translate_span(span, &style));
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
                if !lines.is_empty() {
                    lines.push(Line::default());
                }

                let code_block_caption = {
                    let mut caption =
                        if let Some(language) = language {
                            format!("  Code snippet #{} ({})", code_block_index, language)
                        }
                        else {
                            format!("  Code snippet #{}", code_block_index)
                        };
                    while caption.len() < line_width {
                        caption.push(' ');
                    }
                    let style = document::Style::default().background(document::Color::gray(64));
                    Line::raw(caption).style(translate_style(&style))
                };
                lines.push(code_block_caption);

                let line_style = document::Style::default();
                for line in code_lines {
                    let indentation_span = translate_span(&document::Span { text: "  ".to_owned(), style: line_style }, &line_style );
                    let mut translated_spans = vec![ indentation_span ];

                    line.spans().iter().for_each(|span| {
                        let translated_span = translate_span(span, &line_style);
                        translated_spans.push(translated_span);
                    });

                    lines.push(Line::default().spans(translated_spans));
                }

                lines.push(Line::default());
                code_block_index += 1;
            }
        }
    }

    Paragraph::new(lines)
}