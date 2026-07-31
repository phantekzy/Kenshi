# Kenshi

A fast disk usage analyzer written in Rust that scans once for instant browsing.

## Architecture & Roadmap

Kenshi is designed around a single-pass scan architecture. It reads the disk exactly once and maps the complete file tree into memory. This makes drilling down into directories an instant, zero-latency experience.

Here is how I plan to build the project and organize the codebase:

```text
├── Cargo.toml
└── src/
    ├── main.rs   — CLI args, terminal setup/teardown, event loop
    ├── tree.rs   — Recursive disk scanner (bounded-parallel via threads)
    ├── app.rs    — Navigation state (drill in/out, sort, selection)
    └── ui.rs     — Ratatui rendering (bars, colors, breadcrumbs)
```

### Module Breakdown

*   **`main.rs`**: Handles command-line arguments, manages the terminal setup and teardown, and runs the main application event loop.
*   **`tree.rs`**: The core scanning engine. It recursively scans the disk using bounded-parallel threads for speed and builds the in-memory tree.
*   **`app.rs`**: Manages the application state, keeping track of user navigation (drilling in and out of folders), sorting the biggest files to the top, and handling current selections.
*   **`ui.rs`**: Responsible for the visual terminal interface using `ratatui`. It draws the proportional size bars, colors, and the breadcrumb trail for navigatio
