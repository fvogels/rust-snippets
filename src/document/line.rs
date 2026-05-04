use rkyv::{Archive, Deserialize, Serialize};

use crate::document::{Span, Style};

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Line(pub Vec<Span>);

impl Line {
    pub fn spans(&self) -> &Vec<Span> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(|span| span.len()).sum()
    }

    pub fn indent(&mut self, indentation: usize) {
        let indentation_span = Span::spaces(indentation, Style::default());
        self.0.insert(0, indentation_span);
    }

    pub fn pad_with_spaces(&mut self, target_length: usize) {
        if self.len() < target_length {
            let padding_size = target_length - self.len();
            let padding_span = Span::spaces(padding_size, Style::default());
            self.0.push(padding_span);
        }
    }
}