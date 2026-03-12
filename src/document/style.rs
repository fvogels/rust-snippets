use crate::document::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    foreground: Option<Color>,
    background: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl Style {
    pub fn default() -> Self {
        Style{
            foreground: Some(Color::white()),
            background: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}