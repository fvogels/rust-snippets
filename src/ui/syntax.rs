use std::collections::HashMap;

use ratatui::{text::{Line, Span}, widgets::Paragraph};
use syntect::{easy::HighlightLines, highlighting::{FontStyle, Theme, ThemeSet}, parsing::{SyntaxReference, SyntaxSet}};


pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    aliases: HashMap<String, String>,
    theme: Theme,
}

fn create_theme() -> Theme {
    ThemeSet::load_defaults().themes["base16-eighties.dark"].clone()
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_nonewlines();
        let theme = create_theme();

        SyntaxHighlighter {
            syntax_set,
            theme,
            aliases: HashMap::new(),
        }
    }

    pub fn supported_languages(&self) -> Vec<String> {
        let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
        syntax_set.syntaxes().iter().map(|s| s.name.to_owned()).collect()
    }

    pub fn add_alias(&mut self, alias: &str, language: &str) {
        self.aliases.insert(alias.to_owned(), language.to_owned());
    }

    pub fn highlight_lines<'a>(&self, language: Option<&str>, lines: impl Iterator<Item=&'a str>) -> impl Iterator<Item=Line<'a>> {
        let syntax = self.get_syntax_reference(language);
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        lines.map(move |line| self.highlight_line(line, &mut highlighter))
    }

    fn get_syntax_reference(&self, language: Option<&str>) -> &SyntaxReference {
        if let Some(language) = language {
            let language = self.aliases.get(language).map(String::as_str).unwrap_or(language);

            match self.syntax_set.find_syntax_by_name(language) {
                Some(s) => s,
                None => self.syntax_set.find_syntax_plain_text(),
            }
        }
        else {
            self.syntax_set.find_syntax_plain_text()
        }
    }

    fn highlight_line<'a>(&self, line: &'a str, highlighter: &mut HighlightLines) -> Line<'a> {
        let spans=
            highlighter.highlight_line(line, &self.syntax_set)
                       .unwrap()
                       .into_iter()
                       .map(|segment| convert_to_span(segment.0, segment.1))
                       .collect::<Vec<_>>();

        Line::from(spans)
    }
}


fn convert_to_span<'a>(syntect_style: syntect::highlighting::Style, content: &'a str) -> Span<'a> {
    let ratatui_style = translate_syntect_style(syntect_style);

    Span::styled(content, ratatui_style)
}

fn translate_syntect_style(syntect_style: syntect::highlighting::Style) -> ratatui::style::Style {
    let foreground = translate_syntect_color(syntect_style.foreground);
    // let background = translate_syntect_color(syntect_style.background);

    let syntect_font_style = syntect_style.font_style;
    let is_bold = !syntect_font_style.intersection(FontStyle::BOLD).is_empty();
    let is_italic = !syntect_font_style.intersection(FontStyle::ITALIC).is_empty();
    let is_underlined = !syntect_font_style.intersection(FontStyle::UNDERLINE).is_empty();

    let mut result = ratatui::style::Style::default().fg(foreground);
    if is_bold {
        result = result.bold()
    }
    if is_italic {
        result = result.italic()
    }
    if is_underlined {
        result = result.underlined()
    }

    result
}

fn translate_syntect_color(syntect_color: syntect::highlighting::Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(syntect_color.r, syntect_color.g, syntect_color.b)
}