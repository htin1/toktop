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
    render_simple_column(
        f,
        app,
        area,
        palette,
        OptionsColumn::Provider,
        "Providers",
        &[Provider::OpenAI, Provider::Anthropic],
        |_app, item| item.label().to_string(),
        |app, item| app.selected_provider == *item,
        |app, item| {
            if !app.has_client(*item) {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            }
        },
    );
}

fn render_metrics_column(f: &mut Frame, app: &App, area: Rect, palette: &ColorPalette) {
    render_simple_column(
        f,
        app,
        area,
        palette,
        OptionsColumn::Metric,
        "Metrics",
        &[View::Usage, View::Cost],
        |_app, item| {
            match item {
                View::Cost => "Cost",
                View::Usage => "Usage",
            }
            .to_string()
        },
        |app, item| app.current_view == *item,
        |_app, _item| Style::default().fg(Color::Gray),
    );
}

fn render_range_column(f: &mut Frame, app: &App, area: Rect, palette: &ColorPalette) {
    render_simple_column(
        f,
        app,
        area,
        palette,
        OptionsColumn::Range,
        "Range",
        &[Range::SevenDays, Range::ThirtyDays],
        |_app, item| item.label().to_string(),
        |app, item| app.range == *item,
        |_app, _item| Style::default().fg(Color::Gray),
    );
}

fn render_simple_column<T: Copy>(
    f: &mut Frame,
    app: &App,
    area: Rect,
    palette: &ColorPalette,
    column: OptionsColumn,
    title: &str,
    items: &[T],
    get_label: impl Fn(&App, &T) -> String,
    is_selected: impl Fn(&App, &T) -> bool,
    get_default_style: impl Fn(&App, &T) -> Style,
) {
    let mut lines = Vec::new();
    let is_active = app.options_column == column;

    lines.push(Line::from(Span::styled(
        title,
        Style::default()
            .fg(if is_active {
                palette.primary
            } else {
                Color::Gray
            })
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for item in items {
        let selected = is_selected(app, item);
        let prefix = if is_active && selected { "> " } else { "  " };
        let style = item_style(palette, selected, is_active, get_default_style(app, item));
        lines.push(Line::from(Span::styled(
            format!("{prefix}{}", get_label(app, item)),
            style,
        )));
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}

fn item_style(palette: &ColorPalette, selected: bool, active: bool, default: Style) -> Style {
    if selected && active {
        Style::default()
            .fg(palette.selected_fg)
            .bg(palette.selected_bg)
            .add_modifier(Modifier::BOLD)
    } else if selected {
        Style::default()
            .fg(palette.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        default
    }
}

fn format_filter_display_name(app: &App, filter: &str) -> String {
    match app.group_by {
        GroupBy::Model => filter.to_string(),
        GroupBy::ApiKeys => {
            let api_key_names = &app.provider_info(app.current_provider()).api_key_names;
            api_key_names.get(filter).cloned().unwrap_or_else(|| {
                if filter.len() <= 16 {
                    filter.to_string()
                } else {
                    format!("{}...{}", &filter[..8], &filter[filter.len() - 4..])
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
        } else if is_active_column && is_selected && !is_disabled {
            "> "
        } else {
            "  "
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
        } else if is_selected && is_active_column {
            " ▶"
        } else {
            ""
        };

        let style = if is_disabled {
            Style::default().fg(Color::DarkGray)
        } else {
            item_style(
                palette,
                is_selected,
                is_active_column && !is_expanded,
                Style::default().fg(Color::Gray),
            )
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
    lines.push(Line::from(Span::styled(
        format!("{all_prefix}All"),
        item_style(
            palette,
            is_all_selected,
            is_active_column,
            Style::default().fg(Color::Gray),
        ),
    )));

    for (idx, filter) in filters.iter().enumerate() {
        let is_selected = app.filter_cursor_index == idx + 1;
        let prefix = if is_active_column && is_selected {
            "  > "
        } else {
            "    "
        };
        lines.push(Line::from(Span::styled(
            format!("{prefix}{}", format_filter_display_name(app, filter)),
            item_style(
                palette,
                is_selected,
                is_active_column,
                Style::default().fg(Color::Gray),
            ),
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
