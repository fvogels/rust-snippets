use ratatui::{text::{Line, Span}, widgets::Paragraph};
use syntect::{easy::HighlightLines, highlighting::{Theme, ThemeSet}, parsing::SyntaxSet};


pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults().themes["base16-ocean.dark"].clone();

        SyntaxHighlighter { syntax_set, theme }
    }

    pub fn highlight_lines<'a>(&self, language: &str, lines: impl Iterator<Item=&'a str>) -> Option<Paragraph<'a>> {
        let syntax = self.syntax_set.find_syntax_by_name(language)?;
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let highlighted_lines: Vec<Line> = lines.map(|line| self.highlight_line(line, &mut highlighter)).collect();
        let paragraph = Paragraph::new(highlighted_lines);

        Some(paragraph)
    }

    fn highlight_line<'a>(&self, line: &'a str, highlighter: &mut HighlightLines) -> Line<'a> {
        let spans=
            highlighter.highlight_line(line, &self.syntax_set)
                       .unwrap()
                       .into_iter()
                       .map(|segment: (syntect::highlighting::Style, &str)| {
                            convert_to_span(segment.0, segment.1)
                        }).collect::<Vec<_>>();

        Line::from(spans)
    }
}


fn convert_to_span<'a>(syntect_style: syntect::highlighting::Style, content: &'a str) -> Span<'a> {
    let ratatui_style = translate_syntect_style(syntect_style);

    Span::styled(content, ratatui_style)
}

fn translate_syntect_style(syntect_style: syntect::highlighting::Style) -> ratatui::style::Style {
    let foreground = translate_syntect_color(syntect_style.foreground);
    let background = translate_syntect_color(syntect_style.background);

    ratatui::style::Style::default().fg(foreground).bg(background)
}

fn translate_syntect_color(syntect_color: syntect::highlighting::Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(syntect_color.r, syntect_color.g, syntect_color.b)
}