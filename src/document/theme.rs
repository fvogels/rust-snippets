use crate::document::{Color, Style};

pub struct Theme {
    pub default: Style,
    pub inline_code: Style,
    pub headings: [Style; 3],
    pub table_heading: Style,
    pub table_even_row: Style,
    pub table_odd_row: Style,
}

impl Theme {
    pub fn default() -> Self {
        Theme{
            default: Style::default(),
            inline_code: Style::default().background(Color::gray(128)),
            headings: [
                Style::default().background(Color { r: 255, g: 128, b: 128 }).bold(true),
                Style::default().background(Color::blue(128)).bold(true),
                Style::default().background(Color::blue(192)).bold(true),
            ],
            table_heading: Style::default().background(Color::blue(128)).underline(true),
            table_even_row: Style::default().background(Color::gray(128)),
            table_odd_row: Style::default().background(Color::gray(64)),
        }
    }
}
