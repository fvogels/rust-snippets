use markdown::{ParseOptions, mdast::{Node, Paragraph, Root}, to_mdast};

pub type Document = Vec<Fragment>;

pub enum Fragment {
    Paragraph(String)
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

                    self.fragments.push(Fragment::Paragraph(string));
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

#[cfg(test)]
mod test {
    use indoc::indoc;

    use super::*;

    #[test]
    fn paragraph_single_line() {
        let markdown = indoc! { r#"
        line of text
        "# };

        let document = parse(markdown);

        assert_eq!(1, document.len());
        if let Fragment::Paragraph(text) = &document[0] {
            assert_eq!("line of text", text);
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
            assert_eq!("line of text\nsecond line", text);
        }
        else {
            assert!(false, "fragment should be a paragraph");
        }
    }
}