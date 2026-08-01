use crate::app::App;
use crate::tree::DirNode;
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
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(8),    // list + map
            Constraint::Length(3), // footer
        ])
        .split(area);

    draw_header(f, app, outer[0]);

    // List on top, treemap docked underneath it — same arrangement WizTree uses.
    let panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(outer[1]);

    draw_list(f, app, panels[0], list_state);
    draw_map(f, app, panels[1]);

    draw_footer(f, app, outer[2]);
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
        " Kenshi by phantekzy — disk usage ({}) ",
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

/// How many folder levels the map expands into automatically. Deeper than this,
/// tiles just render as flat leaves — matches WizTree, which also stops
/// subdividing once boxes get too small to be readable.
const MAX_MAP_DEPTH: usize = 4;
const MIN_RECURSE_WIDTH: u16 = 8;
const MIN_RECURSE_HEIGHT: u16 = 4;

/// One rectangle in the fully-expanded map: a file or a folder, at whatever
/// nesting depth it landed at. `top_index` always points back to which
/// top-level (currently listed) entry this tile ultimately belongs to, so
/// the whole branch can be colored and selected consistently.
struct Tile<'a> {
    node: &'a DirNode,
    rect: Rect,
    top_index: usize,
    depth: usize,
    /// true if this folder's own children are drawn inside it (so its body
    /// shouldn't also print a size label that they'd just paint over).
    expanded: bool,
    /// This tile's share of its immediate containing folder's total size —
    /// not of the whole scan, so nested tiles read the way WizTree's do.
    pct_of_parent: f64,
}

/// Recursively squarifies `children` into `area`, then — for any directory
/// tile that's both big enough on screen and within `max_depth` — repeats the
/// process one level deeper inside that tile's own borders. This is what
/// makes the whole tree visible at once instead of just the current folder.
fn layout_tiles<'a>(
    children: &'a [DirNode],
    area: Rect,
    depth: usize,
    max_depth: usize,
    parent_top_index: Option<usize>,
    out: &mut Vec<Tile<'a>>,
) {
    if children.is_empty() || area.width == 0 || area.height == 0 {
        return;
    }

    let parent_total: u64 = children.iter().map(|c| c.size).sum();
    let sizes: Vec<u64> = children.iter().map(|c| c.size).collect();
    let rects = treemap::layout(&sizes, area);

    for (i, (child, rect)) in children.iter().zip(rects.iter()).enumerate() {
        if rect.width == 0 || rect.height == 0 {
            continue;
        }

        let top_index = parent_top_index.unwrap_or(i);
        let can_recurse = child.is_dir
            && !child.children.is_empty()
            && depth < max_depth
            && rect.width >= MIN_RECURSE_WIDTH
            && rect.height >= MIN_RECURSE_HEIGHT;

        let pct_of_parent = if parent_total > 0 {
            child.size as f64 / parent_total as f64 * 100.0
        } else {
            0.0
        };

        out.push(Tile {
            node: child,
            rect: *rect,
            top_index,
            depth,
            expanded: can_recurse,
            pct_of_parent,
        });

        if can_recurse {
            let inner = Rect {
                x: rect.x.saturating_add(1),
                y: rect.y.saturating_add(1),
                width: rect.width.saturating_sub(2),
                height: rect.height.saturating_sub(2),
            };
            if inner.width > 0 && inner.height > 0 {
                layout_tiles(
                    &child.children,
                    inner,
                    depth + 1,
                    max_depth,
                    Some(top_index),
                    out,
                );
            }
        }
    }
}

/// WizTree-style nested treemap, docked under the list: the current directory's
/// whole visible tree — files and subfolders, several levels deep — squarified
/// and drawn all at once, with every tile labeled with its name and size.
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

    let mut tiles: Vec<Tile> = Vec::new();
    layout_tiles(&node.children, inner, 1, MAX_MAP_DEPTH, None, &mut tiles);

    // Parents are always pushed before their own children in `tiles`, so
    // painting in this order naturally layers nested boxes on top of their
    // parent's fill instead of the other way around.
    for tile in &tiles {
        let rect = tile.rect;
        let selected = tile.depth == 1 && tile.top_index == app.selected;
        let base = tile_color(tile.top_index, tile.node.is_dir);
        let color = shade(base, tile.depth);
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

            let show_title = rect.width >= 6;
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_type(if selected {
                    BorderType::Double
                } else {
                    BorderType::Plain
                })
                .border_style(border_style)
                .style(Style::default().bg(color));

            if show_title {
                let icon = if tile.node.is_dir { "D" } else { "F" };
                block = block.title(format!(
                    " [{}] {} — {} ({:.0}%) ",
                    icon,
                    tile.node.name,
                    format_size(tile.node.size, DECIMAL),
                    tile.pct_of_parent
                ));
            }

            let tile_inner = block.inner(rect);
            f.render_widget(block, rect);

            if !tile.expanded && tile_inner.width > 0 && tile_inner.height > 0 {
                let mut lines: Vec<Line> = Vec::new();

                if show_title {
                    // Name, size and % already live in the title, so the body
                    // is free for the extra stats there wasn't room for up there.
                    if tile.node.is_dir {
                        lines.push(Line::from(format!(
                            "{} item(s), {} file(s)",
                            tile.node.children.len(),
                            format_count(tile.node.file_count)
                        )));
                    } else {
                        lines.push(Line::from(file_kind(&tile.node.name)));
                    }
                } else {
                    // No room for a title at all — the body has to carry
                    // everything, one fact per line as height allows.
                    lines.push(Line::from(format!(
                        "{} {}",
                        tile.node.name,
                        format_size(tile.node.size, DECIMAL)
                    )));
                    lines.push(Line::from(format!("{:.0}% of parent", tile.pct_of_parent)));
                    if tile.node.is_dir {
                        lines.push(Line::from(format!(
                            "{} item(s), {} file(s)",
                            tile.node.children.len(),
                            format_count(tile.node.file_count)
                        )));
                    } else {
                        lines.push(Line::from(file_kind(&tile.node.name)));
                    }
                }

                let text_style =
                    Style::default()
                        .fg(Color::White)
                        .bg(color)
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        });
                let p = Paragraph::new(lines)
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
            f.render_widget(fill, rect);
        }
    }
}

/// Cycles through a small palette so adjacent top-level branches are visually
/// distinct, using a cooler palette for directories and a warmer one for
/// files — same idea WizTree uses to separate folders from individual files
/// at a glance.
fn tile_color(top_index: usize, is_dir: bool) -> Color {
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
        DIR_PALETTE[top_index % DIR_PALETTE.len()]
    } else {
        FILE_PALETTE[top_index % FILE_PALETTE.len()]
    }
}

/// Darkens a color slightly per nesting depth, so deeper tiles visibly recede
/// while still reading as "part of" their parent's color family.
fn shade(color: Color, depth: usize) -> Color {
    if let Color::Rgb(r, g, b) = color {
        let factor = 1.0 - ((depth.saturating_sub(1)) as f32 * 0.12).min(0.55);
        Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        )
    } else {
        color
    }
}

/// Inserts thousands separators into a plain integer, e.g. 12345 -> "12,345".
/// A tiny hand-rolled formatter so we don't need an extra crate just for this.
fn format_count(n: u64) -> String {
    let digits = n.to_string();
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(bytes.len() + bytes.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i != 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

/// A short human label for a file tile's body — its extension if it has one,
/// otherwise a generic fallback.
fn file_kind(name: &str) -> String {
    match std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
    {
        Some(ext) if !ext.is_empty() => format!("{} file", ext.to_uppercase()),
        _ => "file".to_string(),
    }
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
