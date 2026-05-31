use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let left_width = app.effective_left_width(area.width);

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(left_width), Constraint::Min(0)])
        .split(root[0]);

    render_notes_panel(frame, columns[0], app);
    render_preview_panel(frame, columns[1], app);
    render_status_bar(frame, root[1]);
}

fn render_notes_panel(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title(" Notes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if app.notes.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(Span::styled("No notes in:", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(
                app.notes_dir.display().to_string(),
                Style::default().add_modifier(Modifier::DIM),
            )),
        ])
        .block(block)
        .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .notes
        .iter()
        .enumerate()
        .map(|(i, note)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(note.name.as_str()).style(style)
        })
        .collect();

    app.list_state.select(Some(app.selected));
    let list = List::new(items).block(block).highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);

    let footer_y = area.bottom().saturating_sub(2);
    if footer_y > area.top() {
        let footer = Paragraph::new(format!("{}/{}", app.selected + 1, app.notes.len())).style(
            Style::default().fg(Color::DarkGray),
        );
        frame.render_widget(
            footer,
            Rect {
                x: area.x + 1,
                y: footer_y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
        );
    }
}

fn render_preview_panel(frame: &mut Frame, area: Rect, app: &mut App) {
    let inner_height = area.height.saturating_sub(2) as u16;
    app.clamp_preview_scroll(inner_height);

    let title = app
        .selected_note()
        .map(|n| format!(" {} ", n.name))
        .unwrap_or_else(|| " Preview ".to_string());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);

    if app.notes.is_empty() {
        let empty = Paragraph::new("Select or create a note.").style(
            Style::default().fg(Color::DarkGray),
        );
        frame.render_widget(empty.block(block), area);
        return;
    }

    let Some(note) = app.selected_note() else {
        return;
    };

    if note.content.is_empty() {
        let empty = Paragraph::new("(empty file)").style(Style::default().add_modifier(Modifier::DIM));
        frame.render_widget(empty.block(block), area);
        return;
    }

    let paragraph = Paragraph::new(note.content.as_str())
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));

    frame.render_widget(paragraph, area);

    let total = note.content.lines().count() as u16;
    if total > inner_height && inner.height > 0 {
        let end = (app.preview_scroll + inner_height).min(total);
        let scroll_info = format!(
            "Line {}-{} of {}",
            app.preview_scroll + 1,
            end,
            total
        );
        frame.render_widget(
            Paragraph::new(scroll_info).style(Style::default().fg(Color::DarkGray)),
            Rect {
                x: inner.x,
                y: inner.bottom().saturating_sub(1),
                width: inner.width,
                height: 1,
            },
        );
    }
}

fn render_status_bar(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new(Span::styled(
        "↑↓/jk notes  [/] scroll  g/G top/bottom  q quit",
        Style::default().add_modifier(Modifier::DIM),
    ));
    frame.render_widget(help, area);
}
