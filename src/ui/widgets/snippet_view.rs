use std::{borrow::Cow};

use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}};

use crate::{document::{self, Document, Style}, snippets::snippets::{Part, Snippet}};

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

    pub fn select_first(&mut self) {
        self.selected_part = 0
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

        let bottom_title = {
            let one_based_index = selected_part_index + 1;
            let part_count = self.snippet.parts.len();

            let caption = match selected_part.caption() {
                Some(caption) => format!(" {}/{} {} ", one_based_index, part_count, caption),
                None => format!(" {}/{} ", one_based_index, part_count),
            };

            Line::raw(caption)
        };
        let mut snippet_caption_block = Block::new().title_bottom(bottom_title).borders(Borders::ALL);
        if let Some(language) = selected_part.language() {
            snippet_caption_block = snippet_caption_block.title_top(language);
        }

        let document = selected_part.document();
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

struct DocumentRenderer<'a> {
    line_width: usize,
    lines: Vec<Line<'a>>,
    code_block_index: i32,
}

impl<'a> DocumentRenderer<'a> {
    // Adds a new empty line if the most recently added line is not empty.
    // If no lines have been generated yet, does nothing.
    fn add_separating_line(&mut self) {
        if let Some(last_line) = self.lines.last() && last_line.width() != 0 {
            self.add_empty_line();
        }
    }

    fn add_empty_line(&mut self) {
        let empty_line = Line::default();
        self.lines.push(empty_line);
    }

    fn add_wrapped(&mut self, words: &Vec<document::Word>, style: &Style) {
        let mut spans = Vec::new();
        let mut acc = 0;

        for word in words {
            let is_fresh_line = acc == 0;
            let separator_size = if is_fresh_line { 0 } else { 1 };
            if acc + word.len() + separator_size > self.line_width && acc > 0 {
                // New line has to be started
                let line = Line::default().spans(spans);
                self.lines.push(line);
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
            self.lines.push(line);
        }
    }

    fn render_code_fragment(&mut self, language: &Option<String>, code_lines: &Vec<document::Line>) {
        self.add_separating_line();

        let style_base = document::Style::default().background(document::Color::gray(32));
        let indentation_style = document::Style::default();
        let caption_style = document::Style::default().background(document::Color::gray(128));
        let margin_size = 2;
        let left_margin_span = translate_span(&document::Span { text: " ".repeat(margin_size), style: indentation_style }, &indentation_style);
        let code_block_width = self.line_width - 2 * margin_size;

        // Add line for code block caption
        let code_block_caption = {
            let mut spans = vec![ left_margin_span.clone() ];

            let caption =
                if let Some(language) = language {
                    format!(" Code snippet #{} ({})", self.code_block_index, language)
                }
                else {
                    format!(" Code snippet #{}", self.code_block_index)
                };
            let caption = format!("{caption:<width$}", caption=caption, width=code_block_width);

            let caption_span = translate_span(&document::Span { text: caption, style: caption_style }, &style_base);
            spans.push(caption_span);
            Line::default().spans(spans)
        };
        self.lines.push(code_block_caption);

        // Add code block lines
        for line in code_lines {
            let mut translated_spans = vec![ left_margin_span.clone() ];
            let mut accumulated_length = 0;

            line.spans().iter().for_each(|span| {
                accumulated_length += span.len();
                let translated_span = translate_span(span, &style_base);
                translated_spans.push(translated_span);
            });

            // Add padding if necessary
            if accumulated_length < code_block_width {
                let padding_length = code_block_width - accumulated_length;
                let padding_string = " ".repeat(padding_length);
                let padding_span = document::Span { text: padding_string, style: style_base };
                translated_spans.push(translate_span(&padding_span, &style_base));
            }

            self.lines.push(Line::default().spans(translated_spans));
        }

        self.add_empty_line();
        self.code_block_index += 1;
    }

    fn render(mut self, document: &Document) -> Paragraph<'a> {
        for fragment in document {
            match fragment {
                document::Fragment::Wrapping{words, style} => {
                    self.add_wrapped(words, style);
                },
                document::Fragment::Heading{words, style, depth: _} => {
                    self.add_separating_line();
                    self.add_wrapped(words, style);
                },
                &document::Fragment::Code { ref language, highlighted_lines: ref code_lines, original: _ } => {
                    self.render_code_fragment(language, code_lines);
                },
                document::Fragment::Verbatim { lines: verbatim_lines} => {
                    let style_base = Style::default();

                    for verbatim_line in verbatim_lines {
                        let translated_spans = verbatim_line.spans().iter().map(|span| translate_span(span, &style_base));
                        let translated_line = Line::default().spans(translated_spans);
                        self.lines.push(translated_line);
                    }
                }
            }
        }

        Paragraph::new(self.lines)
    }
}

fn render_document_as_paragraph(document: &Document, line_width: usize) -> Paragraph<'_> {
    let renderer = DocumentRenderer{
        line_width,
        lines: Vec::new(),
        code_block_index: 1,
    };

    renderer.render(document)
}
