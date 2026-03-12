use crate::document::{Color, Style};

pub struct Theme {
    pub default: Style,
    pub inline_code: Style,
    pub headings: [Style; 3]
}

impl Theme {
    pub fn default() -> Self {
        Theme{
            default: Style::default(),
            inline_code: Style::default().background(Color::gray(128)),
            headings: [
                Style::default().background(Color::blue(128)).bold(),
                Style::default().background(Color::blue(64)),
                Style::default().bold(),
            ]
        }
    }
}
