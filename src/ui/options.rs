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

fn format_filter_display_name(app: &App, filter: &str) -> String {
    match app.group_by {
        GroupBy::Model => filter.to_string(),
        GroupBy::ApiKeys => {
            let api_key_names = &app.provider_info(app.current_provider()).api_key_names;
            api_key_names.get(filter).cloned().unwrap_or_else(|| {
                if filter.chars().count() <= 16 {
                    filter.to_string()
                } else {
                    let prefix: String = filter.chars().take(8).collect();
                    let suffix: String = filter
                        .chars()
                        .rev()
                        .take(4)
                        .collect::<Vec<char>>()
                        .into_iter()
                        .rev()
                        .collect();
                    format!("{}...{}", prefix, suffix)
                }
            })
        }
    }
}

fn render_group_by_options(
    lines: &mut Vec<Line>,
    app: &App,
    palette: &ColorPalette,
    is_active_column: bool,
    is_expanded: bool,
) {
    let group_by_options = [GroupBy::Model, GroupBy::ApiKeys];
    let is_usage_view = app.current_view == View::Usage;

    for group_by in group_by_options.iter() {
        let is_selected = app.group_by == *group_by;
        let is_disabled = !is_usage_view && *group_by == GroupBy::ApiKeys;

        let prefix = if is_expanded {
            if is_selected {
                "> "
            } else {
                "  "
            }
        } else {
            if is_active_column && is_selected && !is_disabled {
                "> "
            } else {
                "  "
            }
        };

        let label = match group_by {
            GroupBy::Model => "Model",
            GroupBy::ApiKeys => "API Keys",
        };

        let expansion_indicator = if is_expanded {
            if is_selected {
                " ▼"
            } else {
                ""
            }
        } else {
            if is_selected && is_active_column {
                " ▶"
            } else {
                ""
            }
        };

        let style = if is_disabled {
            Style::default().fg(Color::DarkGray)
        } else if is_selected && is_active_column && !is_expanded {
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

        lines.push(Line::from(Span::styled(
            format!("{prefix}{label}{expansion_indicator}"),
            style,
        )));
    }
}

fn render_filter_list(
    lines: &mut Vec<Line>,
    app: &App,
    palette: &ColorPalette,
    is_active_column: bool,
) {
    let filters = app.get_available_filters();
    if filters.is_empty() {
        lines.push(Line::from(Span::styled(
            "    (no data)",
            Style::default().fg(Color::DarkGray),
        )));
        return;
    }

    let is_all_selected = app.filter_cursor_index == 0;
    let all_prefix = if is_active_column && is_all_selected {
        "  > "
    } else {
        "    "
    };
    let all_style = if is_all_selected && is_active_column {
        Style::default()
            .fg(palette.selected_fg)
            .bg(palette.selected_bg)
            .add_modifier(Modifier::BOLD)
    } else if is_all_selected {
        Style::default()
            .fg(palette.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    lines.push(Line::from(Span::styled(
        format!("{all_prefix}All"),
        all_style,
    )));

    for (idx, filter) in filters.iter().enumerate() {
        let filter_idx = idx + 1;
        let is_selected = app.filter_cursor_index == filter_idx;
        let prefix = if is_active_column && is_selected {
            "  > "
        } else {
            "    "
        };

        let display_name = format_filter_display_name(app, filter);
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

        lines.push(Line::from(Span::styled(
            format!("{prefix}{display_name}"),
            style,
        )));
    }
}

fn render_group_by_column(f: &mut Frame, app: &App, area: Rect, palette: &ColorPalette) {
    let mut lines = Vec::new();
    let is_active_column = app.options_column == OptionsColumn::GroupBy;
    let is_expanded = app.group_by_expanded && is_active_column;

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

    render_group_by_options(&mut lines, app, palette, is_active_column, is_expanded);

    if is_expanded {
        render_filter_list(&mut lines, app, palette, is_active_column);
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}
