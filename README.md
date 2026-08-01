# kenshi

A fast, terminal-based disk usage analyzer written in Rust. Point it at any folder (or your whole drive) and instantly see what's eating up your space — browse directories, sort by size, name, or date, and see a live WizTree-style treemap of where the space actually is, all without ever leaving the terminal.

Inspired by tools like WizTree and `ncdu`, built from scratch with a multi-threaded scanner and a clean [ratatui](https://github.com/ratatui-org/ratatui) interface.

---

## Table of Contents

- [Features](#features)
- [Preview](#preview)
- [Installation](#installation)
- [Usage](#usage)
- [Controls](#controls)
- [How It Works](#how-it-works)
- [Project Structure](#project-structure)
- [Built With](#built-with)
- [Known Limitations](#known-limitations)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **Multi-threaded scanning** — walks your filesystem in parallel across the first few directory levels, so large drives scan fast instead of crawling through one file at a time.
- **Interactive TUI** — browse your filesystem like a file explorer, right in the terminal.
- **Nested treemap, WizTree-style** — a squarified treemap sits docked directly under the file list, showing the current folder's whole visible tree — files and subfolders, several levels deep — all at once, colored by branch and shaded by depth.
- **Detailed treemap tiles** — every tile is labeled with its name, size, and percentage of its parent folder, plus item counts for directories or the file type for files, not just a colored box.
- **File type breakdown** — a dedicated view (toggle with `t`) that recursively tallies every file under the current folder by extension, ranked by total size, so you can instantly see whether videos, logs, caches, or something else is eating your space.
- **Date-aware** — every file and folder tracks when it was last touched, shown as a relative time ("3d ago", "2mo ago") right in the list, with a dedicated sort mode for surfacing what's old and forgotten.
- **Three-way sort** — cycle between size, name, and date modified with a single keypress.
- **Directory drill-down** — step into folders, back out, and keep exploring without re-scanning.
- **Live rescan** — refresh the scan at any time without restarting the app.
- **Graceful permission handling** — folders you can't read are shown, not skipped silently or crashed on.
- **Zero external runtime dependencies** — a single compiled binary, no interpreter, no config files.

---

## Preview

```
┌ Kenshi by phantekzy — disk usage (sorted by size) ────────────────────────┐
│ / / home / phantekzy    12 item(s)    48.2 GB total                       │
└─────────────────────────────────────────────────────────────────────────┘
┌ Contents (Enter: open dir, Backspace: up) ────────────────────────────────┐
│  42.1% [########------------] 20.3 GB   3d ago  [D] Downloads            │
│> 28.7% [#####---------------] 13.8 GB   2h ago  [D] .cache               │
│  15.2% [###-----------------]  7.3 GB  12d ago  [D] Projects             │
│   9.4% [##-------------------]  4.5 GB   1mo ago [D] Videos              │
│   4.6% [#---------------------]  2.3 GB   6mo ago [F] backup.tar.gz       │
└─────────────────────────────────────────────────────────────────────────┘
┌ Map (Enter: open dir, Backspace: up) ──────────────────────────────────────┐
│ ╔═[D] Downloads — 20.3 GB (42%)══╗ [D] .cache — 13.8 GB (29%)             │
│ ║ [D] movies — 12.1 GB (60%)     ║ [D] Projects — 7.3 GB (15%)            │
│ ║  [D] setup.exe — 3.2 GB (16%)  ║ [D] Videos — 4.5 GB (9%)               │
│ ╚═════════════════════════════════╝ [F] backup.tar.gz — 2.3 GB (5%)      │
└─────────────────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────────────────┐
│ ↑/↓ move   →/Enter open   ←/Backspace back   s sort   t types/map   ... │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Installation

### Prerequisites

You'll need the Rust toolchain installed. If you don't have it yet:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.org | sh
```

### Build From Source

```bash
git clone https://github.com/yourusername/kenshi.git
cd kenshi
cargo build --release
```

The compiled binary will be at `target/release/kenshi`.

### Install To PATH

Installing puts a plain `kenshi` command on your system, so you can run it from anywhere the same way you'd run `btop` or `ncdu`:

```bash
cargo install --path .
```

Make sure `~/.cargo/bin` is on your `PATH` (rustup usually sets this up automatically). After that:

```bash
kenshi ~
```

Whenever you pull new changes or edit the source yourself, reinstall to pick them up:

```bash
cargo install --path . --force
```

`cargo run` always builds fresh from source without touching the installed copy, which is a quick way to sanity-check changes before reinstalling:

```bash
cargo run --release -- ~
```

---

## Usage

### Scan The Current Directory

```bash
kenshi
```

### Scan A Specific Directory

```bash
kenshi /path/to/folder
```

### Scan The Entire Filesystem

Run with elevated permissions for a complete picture:

```bash
sudo kenshi /
```

> **Note:** without `sudo`, directories you don't have read access to will show up as "permission denied" with 0 bytes counted, since the scanner can't look inside them.

---

## Controls

| Key                      | Action                                    |
|---------------------------|--------------------------------------------|
| `↑` / `k`                 | Move selection up                         |
| `↓` / `j`                 | Move selection down                       |
| `Page Up`                 | Jump up 10 items                          |
| `Page Down`               | Jump down 10 items                        |
| `→` / `Enter` / `l`       | Open selected directory                   |
| `←` / `Backspace` / `h`   | Go back to parent directory               |
| `s`                       | Cycle sort mode (size → name → date)      |
| `t` / `Tab`               | Toggle bottom panel (treemap ↔ file types) |
| `r`                       | Rescan the current path                   |
| `q` / `Esc`               | Quit                                      |

---

## How It Works

### Scanning

kenshi walks the directory tree starting from the given path. For each directory, it reads its entries and recursively scans them. To keep things fast on large trees, it spawns worker threads for the first few levels of depth (`MAX_PARALLEL_DEPTH`), then falls back to sequential scanning deeper in the tree — this keeps parallelism useful without spawning thousands of threads for huge, deeply nested directory structures. A thread cap (`MAX_LIVE_THREADS`) prevents runaway thread spawning on very wide directories.

Symlinks are recorded but never followed, which avoids infinite loops from circular links and matches how most disk usage tools report space (they count the link itself, not its target).

Each entry's last-modified timestamp is also captured during the scan: files keep their own timestamp, and directories inherit the most recent timestamp found anywhere inside them — so a folder's "modified" time reflects the last time anything actually changed within it.

### Sizes

Each directory's size is the sum of its children's sizes, computed bottom-up as the scan completes. File counts are aggregated the same way.

### The Treemap

The bottom panel renders a squarified treemap (the same layout family WizTree and `ncdu --map` use) via a custom algorithm in `treemap.rs`, based on the Bruls/Huizing/van Wijk squarify method — it keeps tiles close to a 1:1 aspect ratio instead of producing thin, unreadable slivers.

Unlike a flat single-level treemap, kenshi's map is recursive: any folder tile that's both large enough on screen and within a depth limit gets its own children laid out *inside* its borders, so the whole visible tree — several levels deep — renders at once instead of requiring you to drill in level by level.

### File Type Breakdown

Pressing `t` switches the bottom panel to a recursive walk of every file under the current folder, tallying total size and count per extension, then ranking them by size — the same category-of-space-usage view TreeSize and WizTree both dedicate a tab to.

### Rendering

The interface is built with [ratatui](https://github.com/ratatui-org/ratatui) and [crossterm](https://github.com/crossterm-rs/crossterm), rendering a header (current path + totals), the file list, the treemap or file-types panel underneath it, and a footer with keybindings.

---

## Project Structure

```
src/
├── main.rs      — CLI argument parsing, terminal setup/teardown, event loop
├── tree.rs      — filesystem scanning logic (DirNode, multi-threaded walk, modified-time tracking)
├── app.rs       — application state, navigation, sorting, panel toggling
├── treemap.rs   — squarified treemap layout algorithm (pure geometry, no rendering)
└── ui.rs        — rendering: list view, nested treemap, file-type breakdown, layout
```

---

## Built With

- [Rust](https://www.rust-lang.org/)
- [ratatui](https://github.com/ratatui-org/ratatui) — terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — cross-platform terminal manipulation
- [clap](https://github.com/clap-rs/clap) — command-line argument parsing
- [humansize](https://github.com/LeopoldArkham/humansize) — human-readable byte formatting

---

## Known Limitations

- Virtual filesystems (`/proc`, `/sys`, `/dev`) are scanned like regular directories, so they'll appear in results even though their "sizes" aren't meaningful disk usage.
- The file type breakdown recomputes on every redraw while that panel is open — fine for most folders, but on a folder with millions of files it can add noticeable CPU while the panel stays open. Caching the result per-folder would fix this.
- No file-deletion feature yet — kenshi is read-only by design, for now.

---

## Roadmap

- [ ] Cache file-type breakdown results per folder instead of recomputing every frame
- [ ] Skip/flag virtual filesystems automatically
- [ ] File deletion / cleanup actions from within the TUI
- [ ] Export scan results (JSON/CSV)
- [ ] Search/filter within the current view
- [ ] Mouse support — click a treemap tile to select it
- [ ] Config file for default scan path and keybindings

---

## Contributing

Issues and pull requests are welcome. If you're adding a feature, please keep the scanning logic (`tree.rs`), the layout math (`treemap.rs`), and the rendering (`ui.rs`) decoupled from each other — that separation is what keeps the app easy to reason about.

### Steps

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Push and open a pull request

---

## License

MIT — do whatever you want with it, just don't hold me liable if it tells you your disk is full when it isn't.
