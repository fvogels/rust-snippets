mod style;
mod color;
mod theme;
mod syntax;

use rkyv::{Archive, Deserialize, Serialize};
pub use style::Style;
pub use color::Color;
pub use theme::Theme;
pub use syntax::SyntaxHighlighter;

use markdown::{ParseOptions, mdast::{AlignKind, Code, Heading, Node, Paragraph, Root, Table}, to_mdast};

pub type Document = Vec<Fragment>;

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum Fragment {
    Wrapping { words: Vec<Word>, style: Style },
    Code { language: Option<String>, original: String, highlighted_lines: Vec<Line> },
    Verbatim { lines: Vec<Line> },
}

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Line(Vec<Span>);

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct TableRow(Vec<Span>);

impl Line {
    pub fn spans(&self) -> &Vec<Span> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(|span| span.len()).sum()
    }

    fn indent(&mut self, indentation: usize) {
        let indentation_span = Span{ text: " ".repeat(indentation), style: Style::default() };
        self.0.insert(0, indentation_span);
    }

    fn pad_with_spaces(&mut self, target_length: usize) {
        let padding_size = target_length - self.len();
        let padding = " ".repeat(padding_size);
        let padding_span = Span{ text: padding, style: Style::default() };
        self.0.push(padding_span);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Word(Vec<Span>);

impl Word {
    pub fn len(&self) -> usize {
        self.0.iter().map(Span::len).sum()
    }

    pub fn spans(&self) -> impl Iterator<Item=&Span> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

impl Span {
    pub fn len(&self) -> usize {
        self.text.len()
    }
}

struct Converter<'a, 'b> {
    syntax_highlighter: &'a SyntaxHighlighter,
    fragments: Document,
    theme: &'b Theme,
}

impl<'a, 'b> Converter<'a, 'b> {
    fn new(syntax_highlighter: &'a SyntaxHighlighter, theme: &'b Theme) -> Self {
        Converter{
            fragments: Vec::new(),
            theme,
            syntax_highlighter,
        }
    }

    fn convert_root(&mut self, root: Root) {
        for child in root.children {
            self.convert_node(child);
        }
    }

    fn convert_node(&mut self, node: Node) {
        match node {
            Node::Paragraph(paragraph) => self.convert_paragraph(paragraph),
            Node::Heading(heading) => self.convert_heading(heading),
            Node::Code(code) => self.convert_code(code),
            Node::Table(table) => self.convert_table(table),
            _ => { panic!("unsupported node: {:?}", node); }
        }
    }

    fn convert_table(&mut self, table: Table) {
        let rows = table.children.into_iter().map(|child| {
            match child {
                Node::TableRow(row) => {
                    row.children.into_iter().map(|row_child| {
                        match row_child {
                            Node::TableCell(mut cell) => {
                                assert_eq!(1, cell.children.len(), "table cell should only contain one child");
                                let cell_child = cell.children.pop().unwrap();
                                match cell_child {
                                    Node::Text(text) => {
                                        text.value
                                    },
                                    _ => {
                                        panic!("table cell should contain text; found {:?}", cell)
                                    }
                                }
                            },
                            _ => {
                                panic!("table row children should be table cells; found {:?}", row_child)
                            }
                        }
                    }).collect::<Vec<_>>()
                },
                _ => { panic!("table children should be table rows; found {:?}", child) }
            }
        }).collect::<Vec<_>>();

        let alignments = table.align;
        let column_count = rows[0].len();
        let row_count = rows.len();
        let column_widths = (0..column_count).map(|column_index| {
            (0..row_count).map(|row_index| {
                rows[row_index][column_index].len()
            }).max().unwrap() + 2
        }).collect::<Vec<_>>();

        let lines = rows.into_iter().enumerate().map(|(row_index, row)| {
            let style = {
                if row_index == 0 {
                    // Header row
                    self.theme.table_heading
                }
                else if row_index % 2 == 0 {
                    self.theme.table_even_row
                }
                else {
                    self.theme.table_odd_row
                }
            };

            let spans = row.into_iter().enumerate().map(|(column_index, cell_contents)| {
                let column_width = column_widths[column_index];
                let padded_text = {
                    match alignments[column_index] {
                        AlignKind::Left | AlignKind::None => format!("{content:<width$}", content=cell_contents, width=column_width),
                        AlignKind::Right => format!("{content:>width$}", content=cell_contents, width=column_width),
                        AlignKind::Center => format!("{content:^width$}", content=cell_contents, width=column_width),
                    }
                };

                Span { text: padded_text, style: style }
            }).collect::<Vec<_>>();

            Line(spans)
        }).collect::<Vec<_>>();

        let fragment = Fragment::Verbatim { lines };

        self.fragments.push(fragment);
    }

    fn convert_code(&mut self, code: Code) {
        let language = code.lang;
        let lines = code.value.lines();
        let indentation = 0;
        let mut highlighted_lines = self.syntax_highlighter.highlight_lines(language.as_deref(), lines, indentation).collect::<Vec<_>>();
        let longest_line_length = highlighted_lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let target_line_length = longest_line_length + 2;

        // Indent and pad lines
        for highlighted_line in &mut highlighted_lines {
            highlighted_line.indent(1);
            highlighted_line.pad_with_spaces(target_line_length);
        }

        // Add top and bottom empty line
        let empty_line = {
            let span = Span { text: " ".repeat(target_line_length), style: Style::default() };
            Line(vec![span])
        };
        highlighted_lines.insert(0, empty_line.clone());
        highlighted_lines.push(empty_line);

        let fragment = Fragment::Code { language, original: code.value, highlighted_lines };

        self.fragments.push(fragment);
    }

    fn convert_heading(&mut self, heading: Heading) {
        let mut words = Vec::new();
        let level = (heading.depth - 1) as usize;
        let style = self.theme.headings[level];

        for child in heading.children {
            match child {
                Node::Text(text) => {
                    let string = text.value;

                    for word in split_string_into_words(string.as_str(), &style) {
                        words.push(word)
                    }
                },
                _ => { panic!("unsupported node: {:?}", child); }
            }
        }

        self.fragments.push(Fragment::Wrapping{ words, style: self.theme.headings[level] })
    }

    fn convert_paragraph(&mut self, paragraph: Paragraph) {
        let mut words = Vec::new();
        let mut spans = Vec::new();

        for child in paragraph.children {
            match child {
                Node::Text(text) => {
                    let mut span: String = String::new();

                    for char in text.value.chars() {
                        if char.is_whitespace() {
                            if !span.is_empty() {
                                spans.push(Span{text: span, style: self.theme.default});
                                span = String::new();
                            }

                            if !spans.is_empty() {
                                words.push(Word(spans));
                                spans = Vec::new();
                            }
                        }
                        else {
                            span.push(char);
                        }
                    }

                    if !span.is_empty() {
                        spans.push(Span{text: span, style: self.theme.default})
                    }
                },
                Node::InlineCode(inline_code) => {
                    let string = inline_code.value;
                    let span = Span{text: string, style: self.theme.inline_code};
                    spans.push(span);
                }
                _ => { panic!("unsupported node: {:?}", child); }
            }
        }

        if !spans.is_empty() {
            words.push(Word(spans))
        }

        self.fragments.push(Fragment::Wrapping { words, style: self.theme.default })
    }
}

pub fn parse(markdown: &str, syntax_highlighter: &SyntaxHighlighter, theme: &Theme) -> Document {
    // Enable GitHub flavored markdown (GFM) to enable tables
    let ast = to_mdast(markdown, &ParseOptions::gfm()).unwrap();

    match ast {
        Node::Root(root) => {
            let mut converter = Converter::new(syntax_highlighter, theme);
            converter.convert_root(root);
            converter.fragments
        },
        _ => {
            panic!("expected root node");
        }
    }
}

fn split_string_into_words(string: &str, style: &Style) -> Vec<Word> {
    string.split_ascii_whitespace().map(|part| {
        let span = Span{text: part.into(), style: style.clone()};
        Word(vec![span])
    }).collect()
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use super::*;

    fn word(s: &str, style: &Style) -> Word {
        Word(vec![Span{text: s.into(), style: style.clone()}])
    }

    fn words<'a>(strings: impl Iterator<Item=&'a str>, style: &Style) -> impl Iterator<Item=Word> {
        strings.map(|s| word(s, style))
    }

    #[test]
    fn wrapping_single_line() {
        let markdown = indoc! { r#"
        line of text
        "# };

        let syntax_highlighter = SyntaxHighlighter::new();
        let theme = Theme::default();
        let document = parse(markdown, &syntax_highlighter, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping{ words: ws, style: _ }  = &document[0] {
            let expected = words(["line", "of", "text"].into_iter(), &theme.default).collect::<Vec<_>>();
            assert_eq!(&expected, ws);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }

    #[test]
    fn wrapping_two_lines() {
        let markdown = indoc! { r#"
        line of text
        second line
        "# };

        let syntax_highlighter = SyntaxHighlighter::new();
        let theme = Theme::default();
        let document = parse(markdown, &syntax_highlighter, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping{ words: text, style: _ } = &document[0] {
            let expected = words(["line", "of", "text", "second", "line"].into_iter(), &theme.default).collect::<Vec<_>>();
            assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }

    #[test]
    fn inline_code() {
        let markdown = indoc! { r#"
        some `highlighted` word
        "# };

        let syntax_highlighter = SyntaxHighlighter::new();
        let theme = Theme::default();
        let document = parse(markdown, &syntax_highlighter, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping{ words: text, style: _ } = &document[0] {
            let expected = vec![word("some", &theme.default), word("highlighted", &theme.inline_code), word("word", &theme.default)];
            assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }

    #[test]
    fn single_word_heading() {
        let markdown = indoc! { r#"
        # Title
        "# };

        let syntax_highlighter = SyntaxHighlighter::new();
        let theme = Theme::default();
        let document = parse(markdown, &syntax_highlighter, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping{ words: text, style: _ } = &document[0] {
            let expected = vec![word("Title", &theme.headings[0])];
            assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }

    #[test]
    fn multiple_word_heading() {
        let markdown = indoc! { r#"
        # This is the title
        "# };

        let syntax_highlighter = SyntaxHighlighter::new();
        let theme = Theme::default();
        let document = parse(markdown, &syntax_highlighter, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping{ words: text, style: _ } = &document[0] {
            let expected = words(["This", "is", "the", "title"].into_iter(), &theme.headings[0]).collect::<Vec<_>>();
            assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }

    #[test]
    fn table() {
        let markdown = indoc! { r#"
        | a | b |
        | - | - |
        | 1 | 2 |
        "# };

        let syntax_highlighter = SyntaxHighlighter::new();
        let theme = Theme::default();
        let document = parse(markdown, &syntax_highlighter, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Verbatim{ lines } = &document[0] {
            // let expected = words(["This", "is", "the", "title"].into_iter(), &theme.headings[0]).collect::<Vec<_>>();
            // assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a table but was a {:?}", document[0]);
        }
    }
}