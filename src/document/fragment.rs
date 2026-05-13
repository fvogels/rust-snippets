use rkyv::{Archive, Deserialize, Serialize};

use crate::document::{Line, Style, Word};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum Fragment {
    Heading { words: Vec<Word>, depth: usize, style: Style },
    Paragraph { words: Vec<Word>, style: Style },
    Code { language: Option<String>, original: String, highlighted_lines: Vec<Line> },
    Verbatim { lines: Vec<Line> },
    List { items: Vec<Vec<Word>> },
}
