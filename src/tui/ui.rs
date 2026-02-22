use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use super::{App, Mode};

/// Prefix shown before the input text in the search field.
const INPUT_PREFIX: &str = "> ";

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    draw_list(f, app, chunks[0]);

    match &app.mode {
        Mode::Normal => {
            draw_help(
                f,
                "Enter:select  ^D:done  ^S:assign  ^X:delete  ^A:all  ^Q:quit",
                chunks[1],
            );
        }
        Mode::ConfirmDelete { title, .. } => {
            draw_help(
                f,
                &format!("Delete '{title}'? y/Enter:confirm  n/Esc:cancel"),
                chunks[1],
            );
        }
    }
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    // Build selectable items for scroll indicator calculation
    let selectable = app.selectable_count();
    let list_offset = app.list_state.offset();
    // Inner height available for the items list (area minus 2 borders minus 1 input row)
    let list_visible = area.height.saturating_sub(3) as usize;

    let has_items_above = list_offset > 0;
    let has_items_below = selectable > list_offset + list_visible;

    let all_todos = app.store.list_all();
    let open_count = all_todos.iter().filter(|t| t.is_open()).count();
    let done_count = all_todos.len() - open_count;

    let mut title_spans = vec![
        Span::raw(" tdo  "),
        Span::styled(format!("{open_count} open"), Style::default().fg(Color::Green)),
        Span::styled(" · ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{done_count} done"), Style::default().fg(Color::DarkGray)),
    ];
    let skipped = app.store.skipped;
    if skipped > 0 {
        title_spans.push(Span::styled(" · ", Style::default().fg(Color::DarkGray)));
        title_spans.push(Span::styled(
            format!("{skipped} invalid"),
            Style::default().fg(Color::Red),
        ));
    }
    title_spans.push(Span::raw(" "));
    let title = Line::from(title_spans);

    let mut block = Block::default().borders(Borders::ALL).title(title);
    if has_items_above {
        block = block.title_top(Line::from(" \u{25b2} ").alignment(Alignment::Right));
    }
    if has_items_below {
        block = block.title_bottom(Line::from(" \u{25bc} ").alignment(Alignment::Right));
    }

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    // Split inner area: input row (fixed) + items list (scrollable)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    // Render the search/input field (always visible at top)
    let input_line = if app.input.is_empty() {
        Line::from(vec![
            Span::raw(INPUT_PREFIX),
            Span::styled(
                "Search or create...",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(format!("{INPUT_PREFIX}{}", app.input))
    };
    f.render_widget(Paragraph::new(input_line), chunks[0]);

    // Build selectable items
    let mut items: Vec<ListItem> = Vec::new();

    if app.has_create_line() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("+ ", Style::default().fg(Color::Green)),
            Span::raw(format!("Create \"{}\"", app.input)),
        ])));
    }

    for &idx in &app.filtered {
        let todo = &app.todos[idx];
        let mut spans = Vec::new();
        if !todo.is_open() {
            spans.push(Span::styled(
                format!("{}  [done] {}", todo.id, todo.title()),
                Style::default().fg(Color::DarkGray),
            ));
        } else {
            spans.push(Span::raw(format!("{}  {}", todo.id, todo.title())));
        }
        if todo.is_assigned() {
            let suffix = match &todo.frontmatter.assigned {
                Some(name) if !name.is_empty() => format!(" (assigned: {name})"),
                _ => " (assigned)".to_string(),
            };
            spans.push(Span::styled(suffix, Style::default().fg(Color::Magenta)));
        }
        items.push(ListItem::new(Line::from(spans)));
    }

    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Always show cursor in the input field
    let cursor_x = chunks[0].x + INPUT_PREFIX.len() as u16 + app.input.chars().count() as u16;
    let cursor_y = chunks[0].y;
    f.set_cursor_position((cursor_x, cursor_y));
}

fn draw_help(f: &mut Frame, text: &str, area: Rect) {
    let help = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, area);
}
