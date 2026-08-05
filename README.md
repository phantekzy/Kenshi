# Kenshi

A fast, animated disk usage analyzer for the terminal — written in Rust.

Kenshi scans a directory once, keeps the whole tree in memory, and lets you browse it instantly: drill into folders, sort by size or name or modified time, and see where your space is actually going through a live squarified treemap and a file-type breakdown, all with smooth, tasteful motion instead of a static wall of text.

---

## Features

- **Instant navigation.** The directory is scanned once up front; drilling in and out of folders afterward is pure in-memory traversal, so there's no re-scanning delay.
- **Bounded parallel scanning.** The scanner fans out across threads for the first few directory levels, then falls back to sequential scanning deeper in the tree — fast on wide directory structures without spawning an unbounded number of threads on deep ones.
- **Animated everything.**
  - Directory scanning runs on a background thread while an animated splash screen shows a live spinner, a running files/bytes counter, and a pulsing border — so a big scan never looks like a frozen terminal.
  - The file list reveals itself with a staggered, eased animation on every navigation, sort change, or panel switch.
  - The selected row breathes with a continuous sine-wave glow, so the UI feels alive even at rest.
  - Treemap cells pop in from their center when the view changes.
- **Two bottom panels**, toggled instantly:
  - **Map** — a real squarified treemap of the current directory's contents, with every cell assigned a distinct color (golden-angle spaced around the color wheel, so neighboring cells never look alike) and the selected cell clearly outlined without ever covering its own label.
  - **Types** — an animated stacked bar and legend breaking total usage down by file extension, sharing one consistent color palette with the rest of the UI.
- **Three sort modes** — size, name, and last-modified — each shown with a human-readable relative age ("3d ago") when the terminal is wide enough.
- **Zero unsafe surprises.** Unreadable directories are shown as such instead of crashing the scan; symlinked directories are never followed, avoiding cycles and double-counted sizes.

---

## Preview

```
┌ Kenshi — sorted by size — panel: map ──────────────────────────────────────┐
│ Projects / kenshi    12 item(s)    4.82 GB total                           │
└──────────────────────────────────────────────────────────────────────────┘
┌ Contents — Enter open · Backspace up ──────────────────────────────────────┐
│    41.2% [########----------]     1.99 GB [D] target        2h ago         │
│>   18.7% [####--------------]     902 MB  [D] .git           1d ago        │
│    12.1% [###---------------]     583 MB  [D] node_modules    5d ago       │
│     ...                                                                    │
└──────────────────────────────────────────────────────────────────────────┘
┌ Map — Tab: switch panel ───────────────────────────────────────────────────┐
│ target/                                          ┌──────────┐              │
│ 1.99 GB                                          │ .git/    │              │
│                                                   │ 902 MB   │              │
│                                                   └──────────┘              │
└──────────────────────────────────────────────────────────────────────────┘
│ ↑/↓ move · →/Enter open · ←/Backspace up · s sort · Tab/t panel · r rescan │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build from source

```bash
git clone https://github.com/<your-username>/kenshi.git
cd kenshi
cargo build --release
```

The compiled binary will be at `target/release/wiztree-rs`.

> **Tip:** always build with `--release`. Kenshi's animations and large-directory scans are noticeably smoother optimized than in debug mode.

---

## Usage

```bash
# Scan the current directory
./target/release/wiztree-rs

# Scan a specific path
./target/release/wiztree-rs /path/to/scan
```

### Keybindings

| Key                     | Action                          |
|--------------------------|----------------------------------|
| `↑` / `↓` or `j` / `k`   | Move selection                  |
| `→` / `Enter` / `l`      | Open selected directory         |
| `←` / `Backspace` / `h`  | Go up one level                 |
| `Page Up` / `Page Down`  | Jump 10 rows                    |
| `s`                      | Cycle sort: size → name → modified |
| `Tab` / `t`               | Toggle bottom panel: Map ↔ Types |
| `r`                       | Rescan the current root         |
| `q` / `Esc`               | Quit (or cancel an in-progress scan) |

---

## How it works

### Scanning

`tree::DirNode::scan` walks the filesystem recursively, building a tree where every directory's `size` is the recursive sum of its children. To keep large trees fast without spawning thousands of OS threads, parallelism is bounded two ways:

- **Depth cutoff** — only the first few levels of the tree fan out across threads (`MAX_PARALLEL_DEPTH`); everything deeper scans sequentially within its parent's thread.
- **Live thread cap** — a global atomic counter (`MAX_LIVE_THREADS`) prevents runaway thread spawning on unusually wide directories.

Symlinked directories are detected via `symlink_metadata` and never traversed, which avoids both infinite cycles and inflated size totals.

While the scan runs, two atomic counters (`SCAN_FILES_SEEN`, `SCAN_BYTES_SEEN`) are updated live and read by the splash screen, so the "N files · M scanned" counter on screen reflects real progress, not a fake animation.

### Animation

All motion in Kenshi is driven by a small, deliberately **stateless** clock (`anim::Animation`). Rather than mutating a progress value every frame, it stores only a start time and computes progress on demand from elapsed wall-clock time:

```rust
pub fn eased(&self) -> f32 {
    ease_out_cubic(self.linear())
}
```

This keeps the render loop pure — every frame is a fresh, deterministic function of "how long has it been," with no animation state to get out of sync. Restarting an animation (on navigation, sort, or panel switch) is just replacing the clock with a new one. A companion `Pulse` type drives continuous, never-ending effects (like the selection glow) off a sine wave for the same reason.

### The treemap

The **Map** panel implements the [squarified treemap](https://www.win.tue.nl/~vanwijk/stm.pdf) algorithm: it recursively lays out rectangles to keep aspect ratios close to square, which is what makes a treemap readable instead of a strip of slivers. Each cell is assigned a distinct color using **golden-angle hue spacing** around the color wheel — stepping by ~137.5° per item — rather than an even `360° / N` split. An even split puts adjacent items at nearly identical hues once you have more than a handful of entries; the golden angle guarantees strong contrast between neighbors regardless of how many cells are on screen.

Selection highlighting draws the border **before** placing the label inside its inset area, so the border glyphs and the text never occupy the same cell — a small but deliberate ordering fix to keep labels fully legible even when selected.

---

## Testing

Kenshi is validated at three levels:

1. **Unit checks** against the treemap math (single-item layouts fill their full area, empty/zero-size inputs don't panic, large item sets cover ~100% of the available cells) and the file-type aggregation (extension totals match known byte counts exactly).
2. **Buffer-level render checks** using `ratatui`'s `TestBackend` to render real frames and assert on the exact characters and colors produced — used to confirm, for example, that the selection border and its label no longer overlap, and that neighboring treemap cells get visibly different colors.
3. **End-to-end interactive testing** by driving the compiled binary inside a pseudo-terminal through full sessions — navigation, sorting, panel switching, rescanning, a mid-scan cancel, and a scan of a large real directory tree with permission-restricted subfolders — checking for clean exits and zero panics.

---

## Project structure

```
kenshi/
├── Cargo.toml
└── src/
    ├── main.rs      — CLI args, terminal setup/teardown, scan splash, event loop
    ├── tree.rs       — recursive filesystem scanner (bounded-parallel)
    ├── treemap.rs    — squarified treemap layout algorithm
    ├── app.rs        — navigation state (drill in/out, sort, panel, animation triggers)
    ├── ui.rs          — rendering: list, map, types panel, scan splash
    ├── anim.rs        — stateless animation clock and easing functions
    └── colors.rs      — extension-based and index-based color assignment
```

---

## Roadmap

- [ ] Delete files/folders directly from the UI (with confirmation)
- [ ] Mouse support for the treemap panel
- [ ] Export a scan as JSON/CSV
- [ ] Configurable color themes

---

## License

MIT — see [`LICENSE`](LICENSE) for details.

## Contributing

Issues and pull requests are welcome. If you're proposing a UI change, a quick before/after in the description goes a long way — terminal screenshots are hard to review in the abstract.
