use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use super::{App, Mode};

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
                "j/k:move  e:edit  d:done/reopen  x:delete  n:new  a:all  q:quit",
                chunks[1],
            );
        }
        Mode::NewTodo { input } => {
            draw_input(f, "New todo: ", input, chunks[1]);
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
    let items: Vec<ListItem> = app
        .todos
        .iter()
        .map(|todo| {
            let label = if !todo.is_open() {
                format!("{}  [done] {}", todo.id, todo.title())
            } else {
                format!("{}  {}", todo.id, todo.title())
            };
            ListItem::new(label)
        })
        .collect();

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
}

fn draw_help(f: &mut Frame, text: &str, area: Rect) {
    let help = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, area);
}

fn draw_input(f: &mut Frame, prompt: &str, input: &str, area: Rect) {
    let text = format!("{prompt}{input}");
    let widget = Paragraph::new(text.as_str());
    f.render_widget(widget, area);
    f.set_cursor_position((area.x + prompt.len() as u16 + input.chars().count() as u16, area.y));
}
