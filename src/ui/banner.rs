use crate::app::App;
use crate::ui::colors::ColorPalette;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

const ASCII_LINES: &[&str] = &[
    "   __                __          __",
    "  |  \\              |  \\        |  \\",
    " _| $$_     ______  | $$   __  _| $$_     ______    ______",
    "|   $$ \\   /      \\ | $$  /  \\|   $$ \\   /      \\  /      \\",
    " \\$$$$$$  |  $$$$$$\\| $$_/  $$ \\$$$$$$  |  $$$$$$\\|  $$$$$$\\",
    "  | $$ __ | $$  | $$| $$   $$   | $$ __ | $$  | $$| $$  | $$",
    "  | $$|  \\| $$__/ $$| $$$$$$\\   | $$|  \\| $$__/ $$| $$__/ $$",
    "   \\$$  $$ \\$$    $$| $$  \\$$\\   \\$$  $$ \\$$    $$| $$    $$",
    "    \\$$$$   \\$$$$$$  \\$$   \\$$    \\$$$$   \\$$$$$$ | $$$$$$$",
    "                                                  | $$",
    "                                                  | $$",
    "                                                   \\$$",
];

const CHUNK_WIDTH: usize = 10;
const TOTAL_CHUNKS: usize = 6; // T, O, K, T, O, P
const FRAMES_PER_CHUNK: u32 = 3;

pub fn render_animated_banner(app: &App, palette: &ColorPalette) -> Vec<Line<'static>> {
    let mut text = Vec::new();
    let primary_color = palette.primary;

    let current_chunk_idx = (app.animation_frame / FRAMES_PER_CHUNK) as usize % TOTAL_CHUNKS;

    // Calculate the column range for the currently jumping chunk
    let chunk_start = current_chunk_idx * CHUNK_WIDTH;
    let chunk_end = chunk_start + CHUNK_WIDTH;

    // Render lines
    for (line_idx, original_line) in ASCII_LINES.iter().enumerate() {
        let mut spans = Vec::new();
        let mut current_span = String::new();
        let mut current_style = Style::default().fg(primary_color);

        for (char_idx, original_char) in original_line.chars().enumerate() {
            // Check if this column is in the jumping chunk
            let is_in_jumping_chunk = char_idx >= chunk_start && char_idx < chunk_end;

            // Check if a character from line below (in the jumping chunk) jumped up to this position
            let jumped_char = if line_idx + 1 < ASCII_LINES.len()
                && is_in_jumping_chunk
                && char_idx < ASCII_LINES[line_idx + 1].len()
            {
                let below_char = ASCII_LINES[line_idx + 1]
                    .chars()
                    .nth(char_idx)
                    .unwrap_or(' ');
                if !below_char.is_whitespace() {
                    Some(below_char)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(ch) = jumped_char {
                // Render jumped character (bold)
                let style = Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD);
                if current_style != style && !current_span.is_empty() {
                    spans.push(Span::styled(current_span.clone(), current_style));
                    current_span.clear();
                }
                current_style = style;
                current_span.push(ch);
            } else if is_in_jumping_chunk && !original_char.is_whitespace() {
                // Character is jumping, leave space
                current_span.push(' ');
            } else {
                // Render character normally - reset style if it was bold
                let normal_style = Style::default().fg(primary_color);
                if current_style != normal_style && !current_span.is_empty() {
                    spans.push(Span::styled(current_span.clone(), current_style));
                    current_span.clear();
                }
                current_style = normal_style;
                current_span.push(original_char);
            }
        }

        if !current_span.is_empty() {
            spans.push(Span::styled(current_span, current_style));
        }

        if spans.is_empty() {
            text.push(Line::from(""));
        } else {
            text.push(Line::from(spans));
        }
    }

    text
}
