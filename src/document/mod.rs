mod style;
mod color;

pub use style::Style;
pub use color::Color;

use markdown::{ParseOptions, mdast::{Node, Paragraph, Root}, to_mdast};

pub type Document = Vec<Fragment>;

pub enum Fragment {
    Paragraph(Vec<Word>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word(Vec<Span>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    text: String,
    style: Style,
}


struct Converter {
    fragments: Document,
}

impl Converter {
    fn new() -> Self {
        Converter{
            fragments: Vec::new(),
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
            _ => { panic!("unsupported node: {:?}", node); }
        }
    }

    fn convert_paragraph(&mut self, paragraph: Paragraph) {
        for child in paragraph.children {
            match child {
                Node::Text(text) => {
                    let string = text.value;
                    let words = split_string_into_words(string.as_str());

                    self.fragments.push(Fragment::Paragraph(words));
                },
                _ => { panic!("unsupported node: {:?}", child); }
            }
        }
    }
}

pub fn parse(markdown: &str) -> Document {
    let ast = to_mdast(markdown, &ParseOptions::default()).unwrap();

    match ast {
        Node::Root(root) => {
            let mut converter = Converter::new();
            converter.convert_root(root);
            converter.fragments
        },
        _ => {
            panic!("expected root node");
        }
    }
}

fn split_string_into_words(string: &str) -> Vec<Word> {
    string.split_ascii_whitespace().map(|part| {
        let span = Span{text: part.into(), style: Style::default()};
        Word(vec![span])
    }).collect()
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use super::*;

    fn word(s: &str) -> Word {
        Word(vec![Span{text: s.into(), style: Style::default()}])
    }

    #[test]
    fn paragraph_single_line() {
        let markdown = indoc! { r#"
        line of text
        "# };

        let document = parse(markdown);

        assert_eq!(1, document.len());
        if let Fragment::Paragraph(text) = &document[0] {
            let expected = vec![word("line"), word("of"), word("text")];
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

        let document = parse(markdown);

        assert_eq!(1, document.len());
        if let Fragment::Paragraph(text) = &document[0] {
            let expected = vec![word("line"), word("of"), word("text"), word("second"), word("line")];
            assert_eq!(&expected, text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }

    // #[test]
    // fn code_inside_paragraph() {
    //     let markdown = indoc! { r#"
    //     some `highlighted` word
    //     "# };

    //     let document = parse(markdown);

    //     assert_eq!(1, document.len());
    //     if let Fragment::Paragraph(text) = &document[0] {
    //         assert_eq!("line of text\nsecond line", text);
    //     }
    //     else {
    //         assert!(false, "fragment should be a paragraph");
    //     }
    // }
}