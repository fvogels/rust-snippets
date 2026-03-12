use rkyv::{Archive, Deserialize, Serialize};
use crate::document::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
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

    pub fn foreground(&self, color: Color) -> Self {
        let mut result = self.clone();
        result.foreground = Some(color);
        result
    }

    pub fn no_foreground(&self) -> Self {
        let mut result = self.clone();
        result.foreground = None;
        result
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

    pub fn bold(&self) -> Self {
        let mut result = self.clone();
        result.bold = true;
        result
    }

    pub fn no_bold(&self) -> Self {
        let mut result = self.clone();
        result.bold = false;
        result
    }

    pub fn italic(&self) -> Self {
        let mut result = self.clone();
        result.italic = true;
        result
    }

    pub fn no_italic(&self) -> Self {
        let mut result = self.clone();
        result.italic = false;
        result
    }

    pub fn underline(&self) -> Self {
        let mut result = self.clone();
        result.underline = true;
        result
    }

    pub fn no_underline(&self) -> Self {
        let mut result = self.clone();
        result.underline = false;
        result
    }
}