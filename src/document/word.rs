use rkyv::{Archive, Deserialize, Serialize};

use crate::document::Span;

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Word(pub Vec<Span>);

impl Word {
    pub fn len(&self) -> usize {
        self.0.iter().map(Span::len).sum()
    }

    pub fn spans(&self) -> impl Iterator<Item=&Span> {
        self.0.iter()
    }
}