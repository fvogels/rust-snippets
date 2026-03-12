use crate::document::{Color, Style};

pub struct Theme {
    pub default: Style,
    pub inline_code: Style,
}

impl Theme {
    pub fn default() -> Self {
        Theme{
            default: Style::default(),
            inline_code: Style::default().background(Color::gray(128)),
        }
    }
}
