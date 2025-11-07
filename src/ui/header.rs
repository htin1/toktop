use crate::app::{App, Provider};
use crate::ui::banner;
use crate::ui::colors::ColorPalette;
use crate::ui::utils::format_tokens;
use chrono::Utc;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);
    let total = match provider {
        Provider::OpenAI => app.data.openai_total_cost(),
        Provider::Anthropic => app.data.anthropic_total_cost(),
    };
    let avg_per_day = total / 7.0;

    // Calculate date range metadata
    let data = match provider {
        Provider::OpenAI => &app.data.openai,
        Provider::Anthropic => &app.data.anthropic,
    };

    let date_range = if data.is_empty() {
        "No data".to_string()
    } else {
        let min_date = data.iter().map(|d| d.date).min().unwrap_or(Utc::now());
        let max_date = data.iter().map(|d| d.date).max().unwrap_or(Utc::now());
        format!(
            "{} - {}",
            min_date.format("%m/%d"),
            max_date.format("%m/%d")
        )
    };

    // Get usage statistics for both providers
    let (total_input_tokens, total_output_tokens) = match provider {
        Provider::Anthropic => (
            app.data.anthropic_total_input_tokens(),
            app.data.anthropic_total_output_tokens(),
        ),
        Provider::OpenAI => (
            app.data.openai_total_input_tokens(),
            app.data.openai_total_output_tokens(),
        ),
    };

    let mut text = vec![];

    // Check if we have data for the current provider
    let has_data = !data.is_empty();
    
    // Render animated ASCII art when loading OR when waiting for API key (no data)
    if app.loading || !has_data {
        text.extend(banner::render_animated_banner(app, &palette));
    } else {
        text.push(Line::from(vec![
            Span::styled(
                "Toktop",
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Monitor your LLM API spending"),
        ]));
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled("Total Cost (7d): ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("${:.2}", total),
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    text.push(Line::from(vec![
        Span::styled("Average per day: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("${:.2}", avg_per_day),
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Add usage statistics for both providers
    if total_input_tokens > 0 || total_output_tokens > 0 {
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::styled("Total Tokens (7d): ", Style::default().fg(Color::Gray)),
            Span::styled(
                format_tokens(total_input_tokens + total_output_tokens),
                Style::default()
                    .fg(palette.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        text.push(Line::from(vec![
            Span::styled("  Input: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_tokens(total_input_tokens)),
            Span::raw(" | "),
            Span::styled("Output: ", Style::default().fg(Color::Magenta)),
            Span::raw(format_tokens(total_output_tokens)),
        ]));
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled("Date Range: ", Style::default().fg(Color::Gray)),
        Span::raw(date_range),
    ]));

    f.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Summary")),
        area,
    );
}
