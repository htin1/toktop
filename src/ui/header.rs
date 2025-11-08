use crate::app::App;
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

    let info = app.provider_info(provider);
    let cost_data = &info.cost_data;
    let usage_data = &info.usage_data;
    let has_data = !cost_data.is_empty() || !usage_data.is_empty();

    if app.loading || !has_data {
        let mut text = vec![];
        text.extend(banner::render_animated_banner(app, &palette));
        f.render_widget(
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Summary")),
            area,
        );
        return;
    }

    let total = info.total_cost();
    let avg_per_day = total / 7.0;

    let date_range = {
        let dates: Vec<_> = if !cost_data.is_empty() {
            cost_data.iter().map(|d| d.date).collect()
        } else {
            usage_data.iter().map(|d| d.date).collect()
        };
        let min_date = dates.iter().min().copied().unwrap_or(Utc::now());
        let max_date = dates.iter().max().copied().unwrap_or(Utc::now());
        format!(
            "{} - {}",
            min_date.format("%m/%d"),
            max_date.format("%m/%d")
        )
    };

    let total_input_tokens = info.total_input_tokens();
    let total_output_tokens = info.total_output_tokens();

    let mut text = vec![];

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
