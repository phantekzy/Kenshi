use crate::app::App;
use crate::treemap;
use humansize::{format_size, DECIMAL};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
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
    match app.view_mode {
        crate::app::ViewMode::List => draw_list(f, app, chunks[1], list_state),
        crate::app::ViewMode::Map => draw_map(f, app, chunks[1]),
    }
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
        " Kenshi by phantekzy — disk usage ({}, {}) ",
        match app.sort_mode {
            crate::app::SortMode::Size => "sorted by size",
            crate::app::SortMode::Name => "sorted by name",
        },
        match app.view_mode {
            crate::app::ViewMode::List => "list view",
            crate::app::ViewMode::Map => "map view",
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

fn draw_map(f: &mut Frame, app: &App, area: Rect) {
    let node = app.current_node();

    let outer = Block::default()
        .borders(Borders::ALL)
        .title("Map (Enter: open dir, Backspace: up)");
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    if node.children.is_empty() {
        let msg = if node.readable {
            "(empty directory)"
        } else {
            "(permission denied)"
        };
        let p = Paragraph::new(msg).style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        return;
    }

    let sizes: Vec<u64> = node.children.iter().map(|c| c.size).collect();
    let rects = treemap::layout(&sizes, inner);

    for (i, (child, rect)) in node.children.iter().zip(rects.iter()).enumerate() {
        if rect.width == 0 || rect.height == 0 {
            continue;
        }

        let selected = i == app.selected;
        let color = tile_color(i, child.is_dir);
        let can_frame = rect.width >= 3 && rect.height >= 2;

        if can_frame {
            let border_style = if selected {
                Style::default()
                    .fg(Color::White)
                    .bg(color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Black).bg(color)
            };

            let tile = Block::default()
                .borders(Borders::ALL)
                .border_type(if selected {
                    BorderType::Double
                } else {
                    BorderType::Plain
                })
                .border_style(border_style)
                .style(Style::default().bg(color));

            let tile_inner = tile.inner(*rect);
            f.render_widget(tile, *rect);

            if tile_inner.width > 0 && tile_inner.height > 0 {
                let icon = if child.is_dir { "D" } else { "F" };
                let label = format!(
                    "[{}] {} — {}",
                    icon,
                    child.name,
                    format_size(child.size, DECIMAL)
                );
                let text_style =
                    Style::default()
                        .fg(Color::White)
                        .bg(color)
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        });
                let p = Paragraph::new(label)
                    .style(text_style)
                    .wrap(Wrap { trim: true });
                f.render_widget(p, tile_inner);
            }
        } else {
            let fill_style = if selected {
                Style::default().bg(Color::White)
            } else {
                Style::default().bg(color)
            };
            let fill = Block::default().style(fill_style);
            f.render_widget(fill, *rect);
        }
    }
}

fn tile_color(index: usize, is_dir: bool) -> Color {
    const DIR_PALETTE: [Color; 6] = [
        Color::Rgb(40, 90, 160),
        Color::Rgb(35, 130, 130),
        Color::Rgb(70, 100, 185),
        Color::Rgb(30, 145, 145),
        Color::Rgb(60, 80, 165),
        Color::Rgb(45, 115, 155),
    ];
    const FILE_PALETTE: [Color; 6] = [
        Color::Rgb(150, 105, 35),
        Color::Rgb(140, 70, 70),
        Color::Rgb(165, 130, 45),
        Color::Rgb(130, 90, 90),
        Color::Rgb(170, 120, 55),
        Color::Rgb(120, 80, 100),
    ];
    if is_dir {
        DIR_PALETTE[index % DIR_PALETTE.len()]
    } else {
        FILE_PALETTE[index % FILE_PALETTE.len()]
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let base =
        "↑/↓ move   →/Enter open   ←/Backspace back   s sort   m map/list   r rescan   q quit";
    let text = match &app.status {
        Some(s) => format!("{}   |   {}", base, s),
        None => base.to_string(),
    };
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(p, area);
}
