use crate::document::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
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

    pub fn background(&self, color: Color) -> Self {
        let mut result = self.clone();
        result.background = Some(color);
        result
    }

    pub fn no_background(&self) -> Self {
        let mut result = self.clone();
        result.background = None;
        result
    }
}