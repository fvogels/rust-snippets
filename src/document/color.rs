#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn black() -> Self {
        Color { r: 0, g: 0, b: 0 }
    }

    pub fn white() -> Self {
        Color { r: 255, g: 255, b: 255 }
    }

    pub fn gray(level: u8) -> Self {
        Color { r: level, g: level, b: level }
    }

    pub fn red(level: u8) -> Self {
        Color { r: level, g: 0, b: 0 }
    }

    pub fn green(level: u8) -> Self {
        Color { r: 0, g: level, b: 0 }
    }

    pub fn blue(level: u8) -> Self {
        Color { r: 0, g: 0, b: level }
    }
}
