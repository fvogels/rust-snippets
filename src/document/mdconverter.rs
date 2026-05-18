use markdown::{ParseOptions, mdast::{AlignKind, Code, Heading, List, Node, Paragraph, Root, Table}, to_mdast};

use crate::document::{self, Document, Fragment, Line, Span, Style, SyntaxHighlighter, Theme, Word, fragment};


struct Converter<'a, 'b> {
    syntax_highlighter: &'a SyntaxHighlighter,
    fragments: Document,
    theme: &'b Theme,
}

impl<'a, 'b> Converter<'a, 'b> {
    pub fn new(syntax_highlighter: &'a SyntaxHighlighter, theme: &'b Theme) -> Self {
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
            Node::List(list) => self.convert_list(list),
            _ => { panic!("unsupported node: {:?}", node); }
        }
    }

    fn convert_list(&mut self, list: List) {
        let mut converted_list_items = Vec::new();

        for list_child in list.children {
            match list_child {
                Node::ListItem(list_item) => {
                    assert_eq!(list_item.children.len(), 1);

                    match list_item.children.first().unwrap() {
                        Node::Paragraph(paragraph) => {
                            let converted_children = convert_text_nodes(&paragraph.children, self.theme);
                            converted_list_items.push(converted_children);
                        },
                        _ => panic!("expected paragraph; found {:?}", list_item)
                    }
                },
                _ => {
                    panic!("expected list item; found {:?}", list_child);
                }
            }
        }

        let fragment = Fragment::List { items: converted_list_items };
        self.fragments.push(fragment);
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

        let lines = {
            let mut lines = Vec::new();

            rows.into_iter().enumerate().map(|(row_index, row)| {
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

                let indentation_span = Span { text: "  ".to_owned(), style: Style::default() };
                let mut spans = vec![indentation_span];

                row.into_iter().enumerate().map(|(column_index, cell_contents)| {
                    let column_width = column_widths[column_index];
                    let padded_text = {
                        match alignments[column_index] {
                            AlignKind::Left | AlignKind::None => format!("{content:<width$}", content=cell_contents, width=column_width),
                            AlignKind::Right => format!("{content:>width$}", content=cell_contents, width=column_width),
                            AlignKind::Center => format!("{content:^width$}", content=cell_contents, width=column_width),
                        }
                    };

                    Span { text: padded_text, style: style }
                }).for_each(|span| spans.push(span));

                Line(spans)
            }).for_each(|line| lines.push(line));

            lines
        };

        let fragment = Fragment::Verbatim { lines };

        self.fragments.push(fragment);
    }

    fn convert_code(&mut self, code: Code) {
        let language = code.lang;
        let metadata = code.meta;
        let lines = code.value.lines();
        let indentation = 0;
        let mut highlighted_lines = self.syntax_highlighter.highlight_lines(language.as_deref(), lines, indentation).collect::<Vec<_>>();

        // Indent and pad lines
        for highlighted_line in &mut highlighted_lines {
            highlighted_line.indent(1);
        }

        let fragment = Fragment::Code(fragment::Code { language, original: code.value, highlighted_lines, metadata });

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

        self.fragments.push(Fragment::Heading{ words, style: self.theme.headings[level], depth: level })
    }

    fn convert_paragraph(&mut self, paragraph: Paragraph) {
        let words = convert_text_nodes(&paragraph.children, self.theme);

        self.fragments.push(Fragment::Paragraph { words, style: self.theme.default })
    }
}

fn convert_text_nodes(nodes: &Vec<Node>, theme: &Theme) -> Vec<Word> {
    let mut words = Vec::new();
    let mut spans = Vec::new();

    for node in nodes {
        match node {
            Node::Text(text) => {
                let mut span: String = String::new();

                for char in text.value.chars() {
                    if char.is_whitespace() {
                        if !span.is_empty() {
                            spans.push(Span{text: span, style: theme.default});
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
                    spans.push(Span{text: span, style: theme.default})
                }
            },
            Node::InlineCode(inline_code) => {
                let string = inline_code.value.clone();
                let span = Span{text: string, style: theme.inline_code};
                spans.push(span);
            }
            _ => { panic!("unsupported node: {:?}", node); }
        }
    }

    if !spans.is_empty() {
        words.push(Word(spans))
    }

    words
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
