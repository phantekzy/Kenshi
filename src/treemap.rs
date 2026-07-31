use ratatui::layout::Rect as CellRect;

#[derive(Clone, Copy, Debug)]
struct FRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn worst_ratio(row_areas: &[f64], length: f64) -> f64 {
    if length <= 0.0 {
        return f64::INFINITY;
    }
    let sum: f64 = row_areas.iter().sum();
    if sum <= 0.0 {
        return f64::INFINITY;
    }
    let thickness = sum / length;
    if thickness <= 0.0 {
        return f64::INFINITY;
    }
    row_areas
        .iter()
        .map(|&a| {
            let side = a / thickness;
            if side <= 0.0 {
                f64::INFINITY
            } else if thickness > side {
                thickness / side
            } else {
                side / thickness
            }
        })
        .fold(0.0_f64, f64::max)
}

fn layout_row(row: &[(usize, f64)], rect: FRect, out: &mut [FRect]) -> FRect {
    let row_sum: f64 = row.iter().map(|(_, a)| *a).sum();

    if rect.w <= rect.h {
        let thickness = if rect.w > 0.0 {
            (row_sum / rect.w).min(rect.h)
        } else {
            0.0
        };
        let mut x = rect.x;
        for &(idx, area) in row {
            let item_w = if thickness > 0.0 {
                area / thickness
            } else {
                0.0
            };
            out[idx] = FRect {
                x,
                y: rect.y,
                w: item_w,
                h: thickness,
            };
            x += item_w;
        }
        FRect {
            x: rect.x,
            y: rect.y + thickness,
            w: rect.w,
            h: (rect.h - thickness).max(0.0),
        }
    } else {
        let thickness = if rect.h > 0.0 {
            (row_sum / rect.h).min(rect.w)
        } else {
            0.0
        };
        let mut y = rect.y;
        for &(idx, area) in row {
            let item_h = if thickness > 0.0 {
                area / thickness
            } else {
                0.0
            };
            out[idx] = FRect {
                x: rect.x,
                y,
                w: thickness,
                h: item_h,
            };
            y += item_h;
        }
        FRect {
            x: rect.x + thickness,
            y: rect.y,
            w: (rect.w - thickness).max(0.0),
            h: rect.h,
        }
    }
}

fn squarify_rec(items: &[(usize, f64)], rect: FRect, out: &mut [FRect]) {
    if items.is_empty() {
        return;
    }
    if items.len() == 1 || rect.w <= 0.0 || rect.h <= 0.0 {
        for &(idx, _) in items {
            out[idx] = rect;
        }
        return;
    }

    let length = rect.w.min(rect.h);

    let mut i = 1;
    let mut best = worst_ratio(&[items[0].1], length);
    while i < items.len() {
        let row_areas: Vec<f64> = items[..=i].iter().map(|(_, a)| *a).collect();
        let candidate = worst_ratio(&row_areas, length);
        if candidate > best {
            break;
        }
        best = candidate;
        i += 1;
    }

    let row = &items[..i];
    let rest = &items[i..];
    let remainder = layout_row(row, rect, out);
    squarify_rec(rest, remainder, out);
}

pub fn layout(sizes: &[u64], area: CellRect) -> Vec<CellRect> {
    let n = sizes.len();
    if n == 0 || area.width == 0 || area.height == 0 {
        return vec![CellRect::new(0, 0, 0, 0); n];
    }

    let total_size: u64 = sizes.iter().sum();
    let floor = if total_size > 0 {
        (total_size as f64 * 0.001).max(1.0)
    } else {
        1.0
    };
    let weights: Vec<f64> = sizes.iter().map(|&s| (s as f64).max(floor)).collect();
    let total_weight: f64 = weights.iter().sum();

    let cell_area = area.width as f64 * area.height as f64;
    let scale = if total_weight > 0.0 {
        cell_area / total_weight
    } else {
        0.0
    };
    let scaled: Vec<f64> = weights.iter().map(|w| w * scale).collect();

    let items: Vec<(usize, f64)> = scaled.iter().copied().enumerate().collect();
    let mut out = vec![
        FRect {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0
        };
        n
    ];
    let root = FRect {
        x: area.x as f64,
        y: area.y as f64,
        w: area.width as f64,
        h: area.height as f64,
    };
    squarify_rec(&items, root, &mut out);

    let min_x = area.x as f64;
    let min_y = area.y as f64;
    let max_x = (area.x + area.width) as f64;
    let max_y = (area.y + area.height) as f64;

    out.iter()
        .map(|r| {
            let x0 = r.x.round().clamp(min_x, max_x);
            let y0 = r.y.round().clamp(min_y, max_y);
            let x1 = (r.x + r.w).round().clamp(min_x, max_x);
            let y1 = (r.y + r.h).round().clamp(min_y, max_y);
            CellRect {
                x: x0 as u16,
                y: y0 as u16,
                width: (x1 - x0).max(0.0) as u16,
                height: (y1 - y0).max(0.0) as u16,
            }
        })
        .collect()
}
