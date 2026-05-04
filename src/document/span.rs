use rkyv::{Archive, Deserialize, Serialize};

use crate::document::Style;

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

impl Span {
    pub fn spaces(length: usize, style: Style) -> Self {
        let text = " ".repeat(length);
        
        Span { text, style }
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }
}