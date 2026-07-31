# kenshi

A fast, terminal-based disk usage analyzer written in Rust. Point it at any folder (or your whole drive) and instantly see what's eating up your space — navigate directories, sort by size or name, and spot the big offenders without ever leaving the terminal.

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
- **Visual size bars** — every entry shows a percentage and a proportional bar, so you can see at a glance what's taking up the most space.
- **Sort on the fly** — toggle between sorting by size or by name with a single keypress.
- **Directory drill-down** — step into folders, back out, and keep exploring without re-scanning.
- **Live rescan** — refresh the scan at any time without restarting the app.
- **Graceful permission handling** — folders you can't read are shown, not skipped silently or crashed on.
- **Zero external runtime dependencies** — a single compiled binary, no interpreter, no config files.

---

## Preview

```
┌ kenshi — disk usage (sorted by size) ─────────────────────────────────────┐
│ / / home / phantekzy    12 item(s)    48.2 GB total                       │
└─────────────────────────────────────────────────────────────────────────┘
┌ Contents (Enter: open dir, Backspace: up) ────────────────────────────────┐
│  42.1% [########------------] 20.3 GB  [D] Downloads                     │
│> 28.7% [#####---------------] 13.8 GB  [D] .cache                        │
│  15.2% [###-----------------]  7.3 GB  [D] Projects                      │
│   9.4% [##-------------------]  4.5 GB  [D] Videos                        │
│   4.6% [#---------------------]  2.3 GB  [F] backup.tar.gz                │
└─────────────────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────────────────┐
│ ↑/↓ move   →/Enter open   ←/Backspace back   s sort   r rescan   q quit  │
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

The compiled binary will be at `target/release/kenshi-rs`.

### Install To PATH (Optional)

```bash
cargo install --path .
```

---

## Usage

### Scan The Current Directory

```bash
kenshi-rs
```

### Scan A Specific Directory

```bash
kenshi-rs /path/to/folder
```

### Scan The Entire Filesystem

Run with elevated permissions for a complete picture:

```bash
sudo kenshi-rs /
```

> **Note:** without `sudo`, directories you don't have read access to will show up as "permission denied" with 0 bytes counted, since the scanner can't look inside them.

---

## Controls

| Key                      | Action                          |
|---------------------------|----------------------------------|
| `↑` / `k`                 | Move selection up               |
| `↓` / `j`                 | Move selection down             |
| `Page Up`                 | Jump up 10 items                |
| `Page Down`               | Jump down 10 items              |
| `→` / `Enter` / `l`       | Open selected directory         |
| `←` / `Backspace` / `h`   | Go back to parent directory     |
| `s`                       | Toggle sort mode (size / name)  |
| `r`                       | Rescan the current path         |
| `q` / `Esc`               | Quit                             |

---

## How It Works

### Scanning

kenshi walks the directory tree starting from the given path. For each directory, it reads its entries and recursively scans them. To keep things fast on large trees, it spawns worker threads for the first few levels of depth (`MAX_PARALLEL_DEPTH`), then falls back to sequential scanning deeper in the tree — this keeps parallelism useful without spawning thousands of threads for huge, deeply nested directory structures. A thread cap (`MAX_LIVE_THREADS`) prevents runaway thread spawning on very wide directories.

Symlinks are recorded but never followed, which avoids infinite loops from circular links and matches how most disk usage tools report space (they count the link itself, not its target).

### Sizes

Each directory's size is the sum of its children's sizes, computed bottom-up as the scan completes. File counts are aggregated the same way.

### Rendering

The interface is built with [ratatui](https://github.com/ratatui-org/ratatui) and [crossterm](https://github.com/crossterm-rs/crossterm), rendering a header (current path + totals), a scrollable list of entries with size bars, and a footer with keybindings.

---

## Project Structure

```
src/
├── main.rs   — CLI argument parsing, terminal setup/teardown, event loop
├── tree.rs   — filesystem scanning logic (DirNode, multi-threaded walk)
├── app.rs    — application state, navigation, sorting logic
└── ui.rs     — rendering / layout of the terminal interface
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
- Sorting is applied per-directory-level on demand, not recursively across the whole tree — toggling sort re-sorts whatever directory you're currently viewing.
- No file-deletion feature yet — kenshi is read-only by design, for now.

---

## Roadmap

- [ ] Skip/flag virtual filesystems automatically
- [ ] File deletion / cleanup actions from within the TUI
- [ ] Export scan results (JSON/CSV)
- [ ] Search/filter within the current view
- [ ] Config file for default scan path and keybindings

---

## Contributing

Issues and pull requests are welcome. If you're adding a feature, please keep the scanning logic (`tree.rs`) decoupled from the UI (`ui.rs`) — that separation is what keeps the app easy to reason about.

### Steps

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Push and open a pull request

---

## License

MIT — do whatever you want with it, just don't hold me liable if it tells you your disk is full when it isn't.
