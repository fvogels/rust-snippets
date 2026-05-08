use std::borrow::Cow;

use crate::document;


pub struct Widget<'a> {
    document: &'a document::Document,
}

impl<'a> Widget<'a> {
    pub fn new(document: &'a document::Document) -> Self {
        Widget{ document }
    }
}

impl<'a> ratatui::widgets::Widget for Widget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
        let line_width = area.width;
        let document = self.document;
        let paragraph = render_document_as_paragraph(document, line_width as usize);

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
    lines: Vec<ratatui::text::Line<'a>>,
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
        let empty_line = ratatui::text::Line::default();
        self.lines.push(empty_line);
    }

    fn render_wrapped(&mut self, words: &Vec<document::Word>, style: &document::Style) {
        let mut spans = Vec::new();
        let mut acc = 0;

        for word in words {
            let is_fresh_line = acc == 0;
            let separator_size = if is_fresh_line { 0 } else { 1 };
            if acc + word.len() + separator_size > self.line_width && acc > 0 {
                // New line has to be started
                let line = ratatui::text::Line::default().spans(spans);
                self.lines.push(line);
                spans = Vec::new();
            }
            else {
                // Word fits on current line
                if !is_fresh_line {
                    // Add separating space between words
                    spans.push(ratatui::text::Span::default().content(" ").style(translate_style(style)))
                }
                for span in word.spans() {
                    spans.push(translate_span(span, &style));
                }
                acc += word.len();
            }
        }

        if !spans.is_empty() {
            let line = ratatui::text::Line::default().spans(spans);
            self.lines.push(line);
        }
    }

    fn render_code_fragment(&mut self, language: &Option<String>, code_lines: &Vec<document::Line>) {
        let style_base = document::Style::default().background(document::Color::gray(32));
        let indentation_style = document::Style::default();
        let caption_style = document::Style::default().background(document::Color::gray(128));
        let margin_size = 2;
        let left_margin_span = translate_span(&document::Span::spaces(margin_size, document::Style::default()), &indentation_style);
        let code_block_width = {
            if self.line_width > 2 * margin_size {
                self.line_width - 2 * margin_size
            }
            else {
                10
            }
        };
        // Empty line to surround snippet with
        let separator_line = {
            let empty_span = document::Span::spaces(code_block_width, document::Style::default());
            let spans = vec![ left_margin_span.clone(), translate_span(&empty_span, &style_base) ];
            ratatui::text::Line::default().spans(spans)
        };
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
            ratatui::text::Line::default().spans(spans)
        };

        self.add_separating_line();
        self.lines.push(code_block_caption);
        self.lines.push(separator_line.clone());

        // Add code block lines
        for line in code_lines {
            let mut translated_spans = vec![ left_margin_span.clone() ];
            let mut accumulated_length = 0;

            line.padded_with_spaces(code_block_width).spans().iter().for_each(|span| {
                accumulated_length += span.len();
                let translated_span = translate_span(span, &style_base);
                translated_spans.push(translated_span);
            });

            self.lines.push(ratatui::text::Line::default().spans(translated_spans));
        }

        self.lines.push(separator_line);
        self.add_empty_line();
        self.code_block_index += 1;
    }

    fn render(mut self, document: &document::Document) -> ratatui::widgets::Paragraph<'a> {
        for fragment in document {
            match fragment {
                document::Fragment::Wrapping{words, style} => {
                    self.render_wrapped(words, style);
                },
                document::Fragment::Heading{words, style, depth} => {
                    self.render_heading_fragment(words, style, *depth);
                },
                &document::Fragment::Code { ref language, highlighted_lines: ref code_lines, original: _ } => {
                    self.render_code_fragment(language, code_lines);
                },
                document::Fragment::Verbatim { lines } => {
                    self.render_verbatim_fragment(lines);
                }
            }
        }

        ratatui::widgets::Paragraph::new(self.lines)
    }

    fn render_heading_fragment(&mut self, words: &Vec<document::Word>, style: &document::Style, depth: usize) {
        self.add_separating_line();
        self.render_wrapped(words, style);
        self.add_separating_line();
    }

    fn render_verbatim_fragment(&mut self, lines: &Vec<document::Line>) {
        let style_base = document::Style::default();

        self.add_separating_line();

        for verbatim_line in lines {
            let translated_spans = verbatim_line.spans().iter().map(|span| translate_span(span, &style_base));
            let translated_line = ratatui::text::Line::default().spans(translated_spans);
            self.lines.push(translated_line);
        }

        self.add_separating_line();
    }
}

fn render_document_as_paragraph(document: &document::Document, line_width: usize) -> ratatui::widgets::Paragraph<'_> {
    let renderer = DocumentRenderer{
        line_width,
        lines: Vec::new(),
        code_block_index: 1,
    };

    renderer.render(document)
}
