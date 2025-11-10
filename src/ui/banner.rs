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
    let primary_color = palette.primary;
    let normal_style = Style::default().fg(primary_color);
    let bold_style = Style::default()
        .fg(primary_color)
        .add_modifier(Modifier::BOLD);

    let current_chunk_idx = (app.animation_frame / FRAMES_PER_CHUNK) as usize % TOTAL_CHUNKS;
    let chunk_start = current_chunk_idx * CHUNK_WIDTH;
    let chunk_end = chunk_start + CHUNK_WIDTH;

    let mut text = Vec::new();

    for (line_idx, original_line) in ASCII_LINES.iter().enumerate() {
        let mut spans = Vec::new();
        let mut current_span = String::new();
        let mut current_style = normal_style;

        for (char_idx, original_char) in original_line.chars().enumerate() {
            let is_in_jumping_chunk = char_idx >= chunk_start && char_idx < chunk_end;

            // Check if character from line below jumped up
            let (ch, style) = if line_idx + 1 < ASCII_LINES.len()
                && is_in_jumping_chunk
                && char_idx < ASCII_LINES[line_idx + 1].len()
            {
                let below_char = ASCII_LINES[line_idx + 1]
                    .chars()
                    .nth(char_idx)
                    .unwrap_or(' ');
                if !below_char.is_whitespace() {
                    (below_char, bold_style)
                } else if !original_char.is_whitespace() {
                    (' ', normal_style)
                } else {
                    (original_char, normal_style)
                }
            } else if is_in_jumping_chunk && !original_char.is_whitespace() {
                (' ', normal_style)
            } else {
                (original_char, normal_style)
            };

            // Update style and push span if style changed
            if current_style != style && !current_span.is_empty() {
                spans.push(Span::styled(current_span.clone(), current_style));
                current_span.clear();
            }
            current_style = style;
            current_span.push(ch);
        }

        if !current_span.is_empty() {
            spans.push(Span::styled(current_span, current_style));
        }

        text.push(if spans.is_empty() {
            Line::from("")
        } else {
            Line::from(spans)
        });
    }

    text
}
