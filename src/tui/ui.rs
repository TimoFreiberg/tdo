use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use super::{App, Mode};

/// Prefix shown before the input text in the first list entry.
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
                "^D:done  ^X:delete  ^A:all  ^Q:quit",
                chunks[1],
            );
        }
        Mode::ConfirmDelete { title, .. } => {
            draw_help(
                f,
                &format!("Delete '{title}'? y:confirm  n/Esc:cancel"),
                chunks[1],
            );
        }
    }
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let on_input = app.on_input();

    // Build the input field entry (always first)
    let input_item = if app.input.is_empty() && !on_input {
        ListItem::new(Line::from(Span::styled(
            "  Add a new todo...",
            Style::default().fg(Color::DarkGray),
        )))
    } else {
        ListItem::new(format!("{INPUT_PREFIX}{}", app.input))
    };

    // Build todo items
    let todo_items = app.todos.iter().map(|todo| {
        let label = if !todo.is_open() {
            format!("{}  [done] {}", todo.id, todo.title())
        } else {
            format!("{}  {}", todo.id, todo.title())
        };
        ListItem::new(label)
    });

    let items: Vec<ListItem> = std::iter::once(input_item).chain(todo_items).collect();

    let total = items.len();
    // Inner height = area minus top and bottom borders
    let visible = area.height.saturating_sub(2) as usize;
    let offset = app.list_state.offset();

    let has_items_above = offset > 0;
    let has_items_below = total > offset + visible;

    let mut block = Block::default().borders(Borders::ALL).title(" tdo ");
    if has_items_above {
        block = block.title_top(Line::from(" \u{25b2} ").alignment(Alignment::Right));
    }
    if has_items_below {
        block = block.title_bottom(Line::from(" \u{25bc} ").alignment(Alignment::Right));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, area, &mut app.list_state);

    // Show cursor in the input field when it's selected
    if on_input {
        // The input row is at the top of the list inner area (after the top border)
        // Subtract the scroll offset in case the list scrolled (unlikely for index 0)
        let row_in_view = 0_u16.saturating_sub(offset as u16);
        let cursor_y = area.y + 1 + row_in_view;
        let cursor_x = area.x + 1 + INPUT_PREFIX.len() as u16 + app.input.chars().count() as u16;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_help(f: &mut Frame, text: &str, area: Rect) {
    let help = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, area);
}
