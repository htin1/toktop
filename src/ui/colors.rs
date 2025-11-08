use crate::provider::Provider;
use ratatui::style::Color;

pub struct ColorPalette {
    pub primary: Color,
    pub accent: Color,
    pub error: Color,
    pub chart_colors: Vec<Color>,
    pub selected_bg: Color,
    pub selected_fg: Color,
}

impl ColorPalette {
    pub fn for_provider(provider: Provider) -> Self {
        match provider {
            Provider::Anthropic => Self::anthropic(),
            Provider::OpenAI => Self::openai(),
        }
    }

    fn anthropic() -> Self {
        Self {
            // Book Cloth - warm reddish-orange
            primary: Color::Rgb(0xCC, 0x78, 0x5C),
            // Focus blue (for accents)
            accent: Color::Rgb(0x61, 0xAA, 0xF2),
            // Error red
            error: Color::Rgb(0xBF, 0x4D, 0x43),
            // Warm color palette for charts
            chart_colors: vec![
                Color::Rgb(0xCC, 0x78, 0x5C), // Book Cloth - reddish-orange
                Color::Rgb(0xD4, 0xA2, 0x7F), // Kraft - warm brown
                Color::Rgb(0xEB, 0xDB, 0xBC), // Manilla - creamy beige
                Color::Rgb(0xBF, 0x4D, 0x43), // Error - muted red
                Color::Rgb(0xE5, 0xE4, 0xDF), // Ivory Dark - light beige
                Color::Rgb(0xF0, 0xF0, 0xEB), // Ivory Medium - off-white
            ],
            // Book Cloth for selected background
            selected_bg: Color::Rgb(0xCC, 0x78, 0x5C),
            selected_fg: Color::Rgb(0xFF, 0xFF, 0xFF), // White text
        }
    }

    fn openai() -> Self {
        Self {
            // Cool blue
            primary: Color::Cyan,
            // Green accent
            accent: Color::Green,
            // Red for errors
            error: Color::Red,
            // Cool color palette for charts
            chart_colors: vec![
                Color::Blue,
                Color::Cyan,
                Color::Green,
                Color::Magenta,
                Color::Yellow,
                Color::LightBlue,
                Color::Rgb(0x4A, 0x90, 0xE2), // Bright blue
                Color::Rgb(0x50, 0xC8, 0x78), // Mint green
                Color::Rgb(0x9B, 0x59, 0xB6), // Purple
                Color::Rgb(0xE7, 0x4C, 0x3C), // Coral red
                Color::Rgb(0xF3, 0x9C, 0x12), // Orange
                Color::Rgb(0x1A, 0xBC, 0x9C), // Turquoise
                Color::Rgb(0x34, 0x98, 0xDB), // Sky blue
                Color::Rgb(0x95, 0xA5, 0xA6), // Gray-blue
                Color::Rgb(0xE6, 0x7E, 0x22), // Dark orange
                Color::Rgb(0x8E, 0x44, 0xAD), // Deep purple
                Color::Rgb(0x16, 0xA0, 0x85), // Teal
                Color::Rgb(0x27, 0xAE, 0x60), // Emerald
                Color::Rgb(0x29, 0x80, 0xB9), // Steel blue
            ],
            // Cyan for selected background
            selected_bg: Color::Cyan,
            selected_fg: Color::Black,
        }
    }
}
