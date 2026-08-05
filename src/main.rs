mod app;
mod cache;
mod tree;
mod treemap;
mod ui;
use app::App;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tree::DirNode;

#[derive(Parser, Debug)]
#[command(name = "wiztree-rs", version, about)]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let path = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());

    if !path.is_dir() {
        eprintln!("Not a directory: {}", path.display());
        std::process::exit(1);
    }

    cache::prune_stale();

    let (initial_root, showing_cache) = match cache::load(&path) {
        Some(cached) => (cached, true),
        None => {
            eprintln!("Scanning {} ...", path.display());
            let start = Instant::now();
            let root = DirNode::scan(&path);
            eprintln!(
                "Done in {:.2}s — {} bytes across {} files.",
                start.elapsed().as_secs_f64(),
                root.size,
                root.file_count
            );
            (root, false)
        }
    };

    let mut terminal = setup_terminal()?;
    let mut app = App::new(initial_root);

    // If we loaded from cache, a fresh scan is still needed to catch up
    // with reality — kick it off in the background and swap it in once done.
    let rx = if showing_cache {
        app.status = Some("Loaded cached view — rescanning...".to_string());
        let (tx, rx) = mpsc::channel();
        let scan_path = path.clone();
        thread::spawn(move || {
            let fresh = DirNode::scan(&scan_path);
            let _ = tx.send(fresh);
        });
        Some(rx)
    } else {
        // Still cache this fresh scan for next launch.
        let _ = cache::save(&path, &app.root);
        None
    };

    let result = run(&mut terminal, &mut app, &path, rx);
    restore_terminal(&mut terminal)?;

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    scan_path: &PathBuf,
    mut rx: Option<mpsc::Receiver<DirNode>>,
) -> io::Result<()> {
    let mut list_state = ListState::default();
    loop {
        // Pick up a completed background scan, if one is pending.
        if let Some(receiver) = &rx {
            if let Ok(fresh) = receiver.try_recv() {
                let _ = cache::save(scan_path, &fresh);
                let loaded_from_cache = app.status.is_some();
                *app = App::new(fresh);
                if loaded_from_cache {
                    app.status = Some("Loaded cached view - rescan complete".to_string());
                }
                rx = None;
            }
        }

        list_state.select(Some(app.selected));
        terminal.draw(|f| ui::draw(f, app, &mut list_state))?;

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::PageDown => app.page_down(10),
                    KeyCode::PageUp => app.page_up(10),
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.enter(),
                    KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => app.back(),
                    KeyCode::Char('s') => app.toggle_sort(),
                    KeyCode::Char('t') | KeyCode::Tab => app.toggle_bottom_panel(),
                    KeyCode::Char('r') => {
                        app.status = Some("rescanning...".to_string());
                        terminal.draw(|f| ui::draw(f, app, &mut list_state))?;
                        let root = DirNode::scan(scan_path);
                        let _ = cache::save(scan_path, &root);
                        *app = App::new(root);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}