use ratatui::style::Color;

const PALETTE: &[(&[&str], Color)] = &[
    (
        &["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv"],
        Color::Rgb(224, 108, 117),
    ), // video: red
    (
        &["mp3", "wav", "flac", "aac", "ogg", "m4a"],
        Color::Rgb(198, 120, 221),
    ), // audio: purple
    (
        &[
            "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico", "tiff",
        ],
        Color::Rgb(97, 175, 239), // images: blue
    ),
    (
        &["zip", "tar", "gz", "7z", "rar", "xz", "bz2", "zst"],
        Color::Rgb(209, 154, 102), // archives: orange
    ),
    (
        &[
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "txt", "md",
        ],
        Color::Rgb(152, 195, 121), // documents: green
    ),
    (
        &[
            "rs", "py", "js", "ts", "tsx", "jsx", "go", "c", "cpp", "h", "hpp", "java", "rb",
            "php", "sh",
        ],
        Color::Rgb(229, 192, 123), // source code: yellow
    ),
    (
        &["exe", "dll", "so", "dylib", "bin", "app", "msi"],
        Color::Rgb(86, 182, 194), // binaries: teal
    ),
    (
        &[
            "json", "toml", "yaml", "yml", "xml", "ini", "cfg", "conf", "lock",
        ],
        Color::Rgb(171, 178, 191), // config/data: grey-blue
    ),
];

const FALLBACK_COLORS: &[Color] = &[
    Color::Rgb(224, 108, 117),
    Color::Rgb(97, 175, 239),
    Color::Rgb(152, 195, 121),
    Color::Rgb(209, 154, 102),
    Color::Rgb(198, 120, 221),
    Color::Rgb(86, 182, 194),
];

pub const DIR_COLOR: Color = Color::Rgb(86, 182, 194);

pub fn extension_of(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((_, ext)) if !ext.is_empty() && ext.len() <= 8 => ext.to_lowercase(),
        _ => String::from("(none)"),
    }
}

/// Deterministic color for a file, based on its extension. Directories
/// should use [`DIR_COLOR`] directly instead of calling this.
pub fn color_for_extension(ext: &str) -> Color {
    for (exts, color) in PALETTE {
        if exts.contains(&ext) {
            return *color;
        }
    }
    if ext == "(none)" {
        return Color::Rgb(140, 140, 140);
    }
    let hash: u32 = ext
        .bytes()
        .fold(5381u32, |h, b| h.wrapping_mul(33).wrapping_add(b as u32));
    FALLBACK_COLORS[(hash as usize) % FALLBACK_COLORS.len()]
}

pub fn color_for_entry(name: &str, is_dir: bool) -> Color {
    if is_dir {
        DIR_COLOR
    } else {
        color_for_extension(&extension_of(name))
    }
}

fn to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (200, 200, 200),
    }
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (ar, ag, ab) = to_rgb(a);
    let (br, bg, bb) = to_rgb(b);
    let lerp = |x: u8, y: u8| -> u8 { (x as f32 + (y as f32 - x as f32) * t).round() as u8 };
    Color::Rgb(lerp(ar, br), lerp(ag, bg), lerp(ab, bb))
}
