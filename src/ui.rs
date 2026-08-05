use crate::app::{App, BottomPanel, SortMode};
use crate::colors::{color_for_entry, color_for_extension, lerp_color, DIR_COLOR};
use crate::treemap;
use humansize::{format_size, DECIMAL};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::time::SystemTime;

const BAR_WIDTH: usize = 18;
const SELECT_GLOW_LOW: Color = Color::Rgb(30, 55, 95);
const SELECT_GLOW_HIGH: Color = Color::Rgb(55, 110, 195);

pub fn draw(f: &mut Frame, app: &App, list_state: &mut ListState) {
    let area = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(f, app, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(chunks[1]);

    draw_list(f, app, body[0], list_state);
    match app.bottom_panel {
        BottomPanel::Map => draw_map(f, app, body[1]),
        BottomPanel::Types => draw_types(f, app, body[1]),
    }

    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let node = app.current_node();
    let reveal = app.anim.eased();
    let animated_size = (node.size as f64 * reveal as f64).round() as u64;
    let animated_items = ((node.children.len() as f32) * reveal).round() as usize;

    let text = format!(
        " {}    {} item(s)    {} total ",
        app.breadcrumb(),
        animated_items,
        format_size(animated_size, DECIMAL)
    );

    let sort_label = match app.sort_mode {
        SortMode::Size => "size",
        SortMode::Name => "name",
        SortMode::Modified => "modified",
    };
    let panel_label = match app.bottom_panel {
        BottomPanel::Map => "map",
        BottomPanel::Types => "types",
    };
    let title = format!(" Kenshi — sorted by {sort_label} — panel: {panel_label} ");

    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(p, area);
}

fn row_progress(app: &App, index: usize) -> f32 {
    let stagger = (index as u64 * 14).min(180);
    app.anim.eased_staggered(stagger, 260)
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(1);
    let mut out: String = s.chars().take(keep).collect();
    out.push('…');
    out
}

fn format_age(modified: SystemTime) -> String {
    let secs = SystemTime::now()
        .duration_since(modified)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else if secs < 86400 * 30 {
        format!("{}d ago", secs / 86400)
    } else if secs < 86400 * 365 {
        format!("{}mo ago", secs / (86400 * 30))
    } else {
        format!("{}y ago", secs / (86400 * 365))
    }
}

fn draw_list(f: &mut Frame, app: &App, area: Rect, list_state: &mut ListState) {
    let node = app.current_node();
    let parent_size = node.size.max(1);
    let show_age = area.width >= 100;

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
        .enumerate()
        .map(|(i, c)| {
            let progress = row_progress(app, i);
            let target_pct = (c.size as f64 / parent_size as f64) * 100.0;
            let pct = target_pct * progress as f64;

            let filled = ((pct / 100.0) * BAR_WIDTH as f64).round() as usize;
            let filled = filled.min(BAR_WIDTH);
            let bar = format!("{}{}", "#".repeat(filled), "-".repeat(BAR_WIDTH - filled));

            let color = color_for_entry(&c.name, c.is_dir);
            let icon = if c.is_dir { "D" } else { "F" };

            let name_style = if c.is_dir {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            let mut spans = vec![
                Span::styled(
                    format!("{:>6.1}% ", pct),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(format!("[{}] ", bar), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{:>10} ",
                        format_size((c.size as f64 * progress as f64) as u64, DECIMAL)
                    ),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(format!("[{}] ", icon), Style::default().fg(color)),
                Span::styled(truncate(&c.name, 40), name_style),
            ];

            if show_age {
                spans.push(Span::styled(
                    format!("   {}", format_age(c.modified)),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let glow = lerp_color(SELECT_GLOW_LOW, SELECT_GLOW_HIGH, app.pulse.wave(1400));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Contents — Enter open · Backspace up"),
        )
        .highlight_style(
            Style::default()
                .bg(glow)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, list_state);
}

fn animate_rect(target: Rect, progress: f32) -> Rect {
    if progress >= 1.0 || target.width == 0 || target.height == 0 {
        return target;
    }
    let progress = progress.max(0.0);
    let cx = target.x as f32 + target.width as f32 / 2.0;
    let cy = target.y as f32 + target.height as f32 / 2.0;
    let w = target.width as f32 * progress;
    let h = target.height as f32 * progress;
    let x = (cx - w / 2.0).round().max(target.x as f32) as u16;
    let y = (cy - h / 2.0).round().max(target.y as f32) as u16;
    Rect {
        x,
        y,
        width: w.round() as u16,
        height: h.round() as u16,
    }
}

fn draw_map(f: &mut Frame, app: &App, area: Rect) {
    let node = app.current_node();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Map — Tab: switch panel");
    let inner = block.inner(area);
    f.render_widget(block, area);

    if node.children.is_empty() || inner.width == 0 || inner.height == 0 {
        return;
    }

    let sizes: Vec<u64> = node.children.iter().map(|c| c.size).collect();
    let rects = treemap::layout(&sizes, inner);
    let progress = app.anim.eased();
    let glow = lerp_color(Color::White, Color::Yellow, app.pulse.wave(900));

    for (i, (c, target)) in node.children.iter().zip(rects.iter()).enumerate() {
        let rect = animate_rect(*target, progress);
        if rect.width == 0 || rect.height == 0 {
            continue;
        }

        let bg = color_for_entry(&c.name, c.is_dir);
        let text_fg = if is_light(bg) {
            Color::Black
        } else {
            Color::White
        };

        let label = if rect.height >= 2 && rect.width >= 4 {
            format!(
                "{}\n{}",
                truncate(&c.name, rect.width as usize),
                truncate(&format_size(c.size, DECIMAL), rect.width as usize)
            )
        } else if rect.height >= 1 && rect.width >= 3 {
            truncate(&c.name, rect.width as usize)
        } else {
            String::new()
        };

        let p = Paragraph::new(label).style(Style::default().fg(text_fg).bg(bg));
        f.render_widget(p, rect);

        if i == app.selected && rect.width >= 3 && rect.height >= 3 {
            let border = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(glow).add_modifier(Modifier::BOLD));
            f.render_widget(border, rect);
        }
    }
}

fn is_light(c: Color) -> bool {
    if let Color::Rgb(r, g, b) = c {
        (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000 > 150
    } else {
        false
    }
}

fn draw_types(f: &mut Frame, app: &App, area: Rect) {
    let node = app.current_node();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Types — Tab: switch panel");
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut breakdown = node.extension_breakdown();
    if breakdown.is_empty() {
        let p = Paragraph::new("(no files)").style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        return;
    }

    const TOP_N: usize = 7;
    let total: u64 = breakdown
        .iter()
        .map(|(_, size, _)| *size)
        .sum::<u64>()
        .max(1);
    let mut rows: Vec<(String, u64, u64)> = breakdown.drain(..breakdown.len().min(TOP_N)).collect();
    if !breakdown.is_empty() {
        let other_size: u64 = breakdown.iter().map(|(_, s, _)| *s).sum();
        let other_count: u64 = breakdown.iter().map(|(_, _, c)| *c).sum();
        rows.push(("other".to_string(), other_size, other_count));
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    draw_stacked_bar(f, app, layout[0], &rows, total);

    let legend_area = layout[2];
    for (row_i, (ext, size, count)) in rows.iter().enumerate() {
        if row_i as u16 >= legend_area.height {
            break;
        }
        let progress = app.anim.eased_staggered((row_i as u64) * 40, 300);
        let slide = ((1.0 - progress) * 8.0).round() as usize;
        let pct = (*size as f64 / total as f64) * 100.0;

        let color = if ext == "other" {
            Color::DarkGray
        } else {
            color_for_extension(ext)
        };

        let line = Line::from(vec![
            Span::raw(" ".repeat(slide)),
            Span::styled("■ ", Style::default().fg(color)),
            Span::styled(format!("{:<10}", ext), Style::default().fg(Color::White)),
            Span::styled(
                format!("{:>10}  ", format_size(*size, DECIMAL)),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("{:>5.1}%  ", pct),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("{} file(s)", count),
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let row_area = Rect {
            x: legend_area.x,
            y: legend_area.y + row_i as u16,
            width: legend_area.width,
            height: 1,
        };
        f.render_widget(Paragraph::new(line), row_area);
    }
}

fn draw_stacked_bar(f: &mut Frame, app: &App, area: Rect, rows: &[(String, u64, u64)], total: u64) {
    if area.width == 0 {
        return;
    }
    let mut x = area.x;
    let full_width = area.width as f64;

    for (i, (ext, size, _)) in rows.iter().enumerate() {
        let progress = app.anim.eased_staggered((i as u64) * 30, 320);
        let share = (*size as f64 / total as f64) * full_width;
        let seg_width = (share * progress as f64).round() as u16;
        if seg_width == 0 {
            continue;
        }
        let color = if ext == "other" {
            Color::DarkGray
        } else {
            color_for_extension(ext)
        };
        let seg_area = Rect {
            x,
            y: area.y,
            width: seg_width.min(area.x + area.width - x),
            height: 1,
        };
        f.render_widget(Block::default().style(Style::default().bg(color)), seg_area);
        x += seg_width;
        if x >= area.x + area.width {
            break;
        }
    }
    let _ = DIR_COLOR; // keep import alive if unused on some cfgs
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn draw_scanning(
    f: &mut Frame,
    path: &str,
    elapsed_secs: f32,
    files_seen: u64,
    bytes_seen: u64,
) {
    let area = f.size();
    let frame_idx = ((elapsed_secs * 12.5) as usize) % SPINNER_FRAMES.len();
    let spinner = SPINNER_FRAMES[frame_idx];

    let glow_t = ((elapsed_secs * 1.4).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let border_color = lerp_color(Color::Rgb(60, 90, 140), Color::Rgb(100, 160, 230), glow_t);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" Kenshi — scanning ");

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {spinner}  Scanning {path}"),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  {} files  ·  {} scanned  ·  {:.1}s elapsed",
                files_seen,
                format_size(bytes_seen, DECIMAL),
                elapsed_secs
            ),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Esc/q to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let p = Paragraph::new(text).block(block);
    f.render_widget(p, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let base =
        "↑/↓ move · →/Enter open · ←/Backspace up · s sort · Tab/t panel · r rescan · q quit";
    let text = match &app.status {
        Some(s) => format!("{}   |   {}", base, s),
        None => base.to_string(),
    };
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(p, area);
}
