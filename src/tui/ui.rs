use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use super::{App, Mode};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    match &app.mode {
        Mode::Normal => {
            draw_list(f, app, chunks[0]);
            draw_help(
                f,
                "j/k: navigate  e/Enter: edit  d: done  n: new  q/Esc: quit",
                chunks[1],
            );
        }
        Mode::NewTodo { input } => {
            draw_list(f, app, chunks[0]);
            draw_input(f, "New todo: ", input, chunks[1]);
        }
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

fn draw_help(f: &mut Frame, text: &str, area: Rect) {
    let help = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, area);
}

fn draw_input(f: &mut Frame, prompt: &str, input: &str, area: Rect) {
    let text = format!("{prompt}{input}");
    let widget = Paragraph::new(text.as_str());
    f.render_widget(widget, area);
    f.set_cursor_position((area.x + prompt.len() as u16 + input.len() as u16, area.y));
}
