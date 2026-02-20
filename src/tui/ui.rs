use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use super::{App, Mode};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    draw_list(f, app, chunks[0]);
    draw_help(f, &app.mode, chunks[1]);

    if let Mode::NewTodo { input } = &app.mode {
        draw_new_todo_popup(f, input);
    }
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .todos
        .iter()
        .enumerate()
        .map(|(i, todo)| {
            let style = if i == app.cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(format!("{}  {}", todo.id, todo.title())).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" tdo "));
    f.render_widget(list, area);
}

fn draw_help(f: &mut Frame, mode: &Mode, area: Rect) {
    let text = match mode {
        Mode::Normal => "j/k: navigate  Enter: edit  d: done  n: new  q: quit",
        Mode::NewTodo { .. } => "Enter: create  Esc: cancel",
    };
    let help = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, area);
}

fn draw_new_todo_popup(f: &mut Frame, input: &str) {
    let area = centered_rect(60, 3, f.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" New todo ");
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let input_widget = Paragraph::new(input);
    f.render_widget(input_widget, inner);
    f.set_cursor_position((inner.x + input.len() as u16, inner.y));
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 20) / 2),
            Constraint::Length(height),
            Constraint::Percentage((100 - 20) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}
