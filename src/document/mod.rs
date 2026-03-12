mod style;
mod color;
mod theme;

pub use style::Style;
pub use color::Color;
pub use theme::Theme;

use markdown::{ParseOptions, mdast::{Heading, Node, Paragraph, Root}, to_mdast};

pub type Document = Vec<Fragment>;

pub enum Fragment {
    Wrapping(Vec<Word>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word(Vec<Span>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    text: String,
    style: Style,
}

struct Converter<'a> {
    fragments: Document,
    theme: &'a Theme,
}

impl<'a> Converter<'a> {
    fn new(theme: &'a Theme) -> Self {
        Converter{
            fragments: Vec::new(),
            theme: theme,
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
            _ => { panic!("unsupported node: {:?}", node); }
        }
    }

    fn convert_heading(&mut self, heading: Heading) {
        let mut words = Vec::new();
        let level = heading.depth - 1;
        let style = self.theme.headings[level as usize];

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

        self.fragments.push(Fragment::Wrapping(words))
    }

    fn convert_paragraph(&mut self, paragraph: Paragraph) {
        let mut words = Vec::new();

        for child in paragraph.children {
            match child {
                Node::Text(text) => {
                    let string = text.value;

                    for word in split_string_into_words(string.as_str(), &self.theme.default) {
                        words.push(word)
                    }
                },
                Node::InlineCode(inline_code) => {
                    let string = inline_code.value;
                    let span = Span{text: string, style: self.theme.inline_code};
                    let word = Word(vec![span]);

                    words.push(word)
                }
                _ => { panic!("unsupported node: {:?}", child); }
            }
        }

        self.fragments.push(Fragment::Wrapping(words))
    }
}

pub fn parse(markdown: &str, theme: &Theme) -> Document {
    let ast = to_mdast(markdown, &ParseOptions::default()).unwrap();

    match ast {
        Node::Root(root) => {
            let mut converter = Converter::new(theme);
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
    fn paragraph_single_line() {
        let markdown = indoc! { r#"
        line of text
        "# };

        let theme = Theme::default();
        let document = parse(markdown, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping(text) = &document[0] {
            let expected = words(["line", "of", "text"].into_iter(), &theme.default).collect::<Vec<_>>();
            assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }

    #[test]
    fn paragraph_two_lines() {
        let markdown = indoc! { r#"
        line of text
        second line
        "# };

        let theme = Theme::default();
        let document = parse(markdown, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping(text) = &document[0] {
            let expected = words(["line", "of", "text", "second", "line"].into_iter(), &theme.default).collect::<Vec<_>>();
            assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }

    #[test]
    fn code_inside_paragraph() {
        let markdown = indoc! { r#"
        some `highlighted` word
        "# };

        let theme = Theme::default();
        let document = parse(markdown, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping(text) = &document[0] {
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

        let theme = Theme::default();
        let document = parse(markdown, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping(text) = &document[0] {
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

        let theme = Theme::default();
        let document = parse(markdown, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Wrapping(text) = &document[0] {
            let expected = words(["This", "is", "the", "title"].into_iter(), &theme.headings[0]).collect::<Vec<_>>();
            assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }
}