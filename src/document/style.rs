use rkyv::{Archive, Deserialize, Serialize};
use crate::document::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
}

impl Style {
    pub fn default() -> Self {
        Style{
            foreground: Some(Color::white()),
            background: None,
            bold: Some(false),
            italic: Some(false),
            underline: Some(false),
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

    pub fn bold(&self, value: bool) -> Self {
        let mut result = self.clone();
        result.bold = Some(value);
        result
    }

    pub fn no_bold(&self) -> Self {
        let mut result = self.clone();
        result.bold = None;
        result
    }

    pub fn italic(&self, value: bool) -> Self {
        let mut result = self.clone();
        result.italic = Some(value);
        result
    }

    pub fn no_italic(&self) -> Self {
        let mut result = self.clone();
        result.italic = None;
        result
    }

    pub fn underline(&self, value: bool) -> Self {
        let mut result = self.clone();
        result.underline = Some(value);
        result
    }

    pub fn no_underline(&self) -> Self {
        let mut result = self.clone();
        result.underline = None;
        result
    }
}