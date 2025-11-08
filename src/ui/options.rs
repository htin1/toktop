use crate::app::{App, GroupBy, OptionsColumn, Range, View};
use crate::provider::Provider;
use crate::ui::colors::ColorPalette;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let provider = app.current_provider();
    let palette = ColorPalette::for_provider(provider);

    let block = Block::default().borders(Borders::ALL).title("Options");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(inner);

    render_providers_column(f, app, chunks[0], &palette);
    render_metrics_column(f, app, chunks[1], &palette);
    render_group_by_column(f, app, chunks[2], &palette);
    render_range_column(f, app, chunks[3], &palette);
}

fn render_providers_column(f: &mut Frame, app: &App, area: Rect, palette: &ColorPalette) {
    let providers = [Provider::OpenAI, Provider::Anthropic];
    let mut lines = Vec::new();
    let is_active_column = app.options_column == OptionsColumn::Provider;

    lines.push(Line::from(Span::styled(
        "Providers",
        Style::default()
            .fg(if is_active_column {
                palette.primary
            } else {
                Color::Gray
            })
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for provider in providers.iter() {
        let is_selected = app.selected_provider == *provider;
        let has_client = app.has_client(*provider);

        let prefix = if is_active_column && is_selected {
            "> "
        } else {
            "  "
        };
        let mut label = provider.label().to_string();
        if !has_client {
            label.push_str(" (key needed)");
        }

        let style = if is_selected && is_active_column {
            Style::default()
                .fg(palette.selected_fg)
                .bg(palette.selected_bg)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD)
        } else if !has_client {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let padded = format!("{prefix}{label}");
        lines.push(Line::from(Span::styled(padded, style)));
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}

fn render_range_column(f: &mut Frame, app: &App, area: Rect, palette: &ColorPalette) {
    let ranges = [Range::SevenDays, Range::ThirtyDays];
    let mut lines = Vec::new();
    let is_active_column = app.options_column == OptionsColumn::Range;

    lines.push(Line::from(Span::styled(
        "Range",
        Style::default()
            .fg(if is_active_column {
                palette.primary
            } else {
                Color::Gray
            })
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for range in ranges.iter() {
        let is_selected = app.range == *range;
        let prefix = if is_active_column && is_selected {
            "> "
        } else {
            "  "
        };

        let style = if is_selected && is_active_column {
            Style::default()
                .fg(palette.selected_fg)
                .bg(palette.selected_bg)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let padded = format!("{prefix}{}", range.label());
        lines.push(Line::from(Span::styled(padded, style)));
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}

fn render_metrics_column(f: &mut Frame, app: &App, area: Rect, palette: &ColorPalette) {
    let metrics = [View::Usage, View::Cost];
    let mut lines = Vec::new();
    let is_active_column = app.options_column == OptionsColumn::Metric;

    lines.push(Line::from(Span::styled(
        "Metrics",
        Style::default()
            .fg(if is_active_column {
                palette.primary
            } else {
                Color::Gray
            })
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for metric in metrics.iter() {
        let is_selected = app.current_view == *metric;

        let prefix = if is_active_column && is_selected {
            "> "
        } else {
            "  "
        };
        let label = match metric {
            View::Cost => "Cost",
            View::Usage => "Usage",
        };

        let style = if is_selected && is_active_column {
            Style::default()
                .fg(palette.selected_fg)
                .bg(palette.selected_bg)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let padded = format!("{prefix}{label}");
        lines.push(Line::from(Span::styled(padded, style)));
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}

fn render_group_by_column(f: &mut Frame, app: &App, area: Rect, palette: &ColorPalette) {
    let group_by_options = [GroupBy::Model, GroupBy::ApiKeys];
    let mut lines = Vec::new();
    let is_active_column = app.options_column == OptionsColumn::GroupBy;

    lines.push(Line::from(Span::styled(
        "Group By",
        Style::default()
            .fg(if is_active_column {
                palette.primary
            } else {
                Color::Gray
            })
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let is_usage_view = app.current_view == View::Usage;

    for group_by in group_by_options.iter() {
        let is_selected = app.group_by == *group_by;
        let is_disabled = !is_usage_view && *group_by == GroupBy::ApiKeys;

        let prefix = if is_active_column && is_selected && !is_disabled {
            "> "
        } else {
            "  "
        };
        let label = match group_by {
            GroupBy::Model => "Model",
            GroupBy::ApiKeys => "API Keys",
        };

        let style = if is_disabled {
            Style::default().fg(Color::DarkGray)
        } else if is_selected && is_active_column {
            Style::default()
                .fg(palette.selected_fg)
                .bg(palette.selected_bg)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let padded = format!("{prefix}{label}");
        lines.push(Line::from(Span::styled(padded, style)));
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}
