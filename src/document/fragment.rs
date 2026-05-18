use rkyv::{Archive, Deserialize, Serialize};

use crate::document::{Line, Style, Word};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum Fragment {
    Heading { words: Vec<Word>, depth: usize, style: Style },
    Paragraph { words: Vec<Word>, style: Style },
    Code(Code),
    Verbatim { lines: Vec<Line> },
    List { items: Vec<Vec<Word>> },
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct Code {
    pub language: Option<String>,
    pub original: String,
    pub highlighted_lines: Vec<Line>,
    pub metadata: Option<String>,
}