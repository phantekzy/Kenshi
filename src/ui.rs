use crate::app::App;
use humansize::{format_size, DECIMAL};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

const BAR_WIDTH: usize = 20;

pub fn draw(f: &mut Frame, app: &App, list_state: &mut ListState) {
    let area = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_list(f, app, chunks[1], list_state);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let node = app.current_node();
    let text = format!(
        " {}    {} item(s)    {} total ",
        app.breadcrumb(),
        node.children.len(),
        format_size(node.size, DECIMAL)
    );
    let title = format!(
        " wiztree-rs — disk usage ({}) ",
        match app.sort_mode {
            crate::app::SortMode::Size => "sorted by size",
            crate::app::SortMode::Name => "sorted by name",
        }
    );
    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(p, area);
}

fn draw_list(f: &mut Frame, app: &App, area: Rect, list_state: &mut ListState) {
    let node = app.current_node();
    let parent_size = node.size.max(1);

    if node.children.is_empty() {
        let msg = if node.readable {
            "(empty directory)"
        } else {
            "(permission denied)"
        };
        let p = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title("Contents"));
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = node
        .children
        .iter()
        .map(|c| {
            let pct = (c.size as f64 / parent_size as f64) * 100.0;
            let filled = ((pct / 100.0) * BAR_WIDTH as f64).round() as usize;
            let filled = filled.min(BAR_WIDTH);
            let bar = format!("{}{}", "#".repeat(filled), "-".repeat(BAR_WIDTH - filled));

            let (color, icon) = if c.is_dir {
                (Color::Cyan, "D")
            } else {
                (Color::White, "F")
            };

            let name_style = if c.is_dir {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>6.1}% ", pct),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(format!("[{}] ", bar), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:>10} ", format_size(c.size, DECIMAL)),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(format!("[{}] ", icon), Style::default().fg(color)),
                Span::styled(c.name.clone(), name_style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Contents (Enter: open dir, Backspace: up)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, list_state);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let base = "↑/↓ move   →/Enter open   ←/Backspace back   s sort   r rescan   q quit";
    let text = match &app.status {
        Some(s) => format!("{}   |   {}", base, s),
        None => base.to_string(),
    };
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(p, area);
}
