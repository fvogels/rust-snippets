mod style;
mod color;
mod theme;
mod syntax;
mod mdconverter;

mod span;
mod word;
mod line;
mod fragment;

pub use span::Span;
pub use word::Word;
pub use line::Line;
pub use fragment::Fragment;
pub use style::Style;
pub use color::Color;
pub use theme::Theme;
pub use syntax::SyntaxHighlighter;
pub use mdconverter::{parse};

pub type Document = Vec<Fragment>;


// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct TableRow(Vec<Span>);

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
            assert!(false, "fragment should be a Fragment::Wrapping");
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
        if let Fragment::Heading{ words: text, style: _, depth } = &document[0] {
            let expected = vec![word("Title", &theme.headings[0])];
            assert_eq!(&expected, text);
            assert_eq!(0, *depth);
        }
        else {
            assert!(false, "fragment should be a Fragment::Wrapping, was a {:?} instead", document[0]);
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
        if let Fragment::Heading{ words: text, style: _, depth } = &document[0] {
            let expected = words(["This", "is", "the", "title"].into_iter(), &theme.headings[0]).collect::<Vec<_>>();
            assert_eq!(&expected, text);
            assert_eq!(0, *depth);
        }
        else {
            assert!(false, "fragment should be a Fragment::Wrapping, was a {:?} instead", document[0]);
        }
    }

    #[test]
    fn heading_depth() {
        let markdown = indoc! { r#"
        ## Title
        "# };

        let syntax_highlighter = SyntaxHighlighter::new();
        let theme = Theme::default();
        let document = parse(markdown, &syntax_highlighter, &theme);

        assert_eq!(1, document.len());
        if let Fragment::Heading{ words: text, style: _, depth } = &document[0] {
            let expected = words(["Title"].into_iter(), &theme.headings[1]).collect::<Vec<_>>();
            assert_eq!(&expected, text);
            assert_eq!(1, *depth);
        }
        else {
            assert!(false, "fragment should be a Fragment::Wrapping, was a {:?} instead", document[0]);
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