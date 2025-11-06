use crate::app::Provider;
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
            ],
            // Cyan for selected background
            selected_bg: Color::Cyan,
            selected_fg: Color::Black,
        }
    }
}

