mod anim;
mod app;
mod colors;
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

const FRAME_INTERVAL: Duration = Duration::from_millis(33);

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

    let mut terminal = setup_terminal()?;

    let root = match scan_with_splash(&mut terminal, &path)? {
        Some(root) => root,
        None => {
            // User cancelled the scan.
            restore_terminal(&mut terminal)?;
            println!("Cancelled.");
            return Ok(());
        }
    };

    let mut app = App::new(root);
    let result = run(&mut terminal, &mut app, &path);
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

fn scan_with_splash(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    path: &PathBuf,
) -> io::Result<Option<DirNode>> {
    let (tx, rx) = mpsc::channel();
    let scan_path = path.clone();
    thread::spawn(move || {
        let root = DirNode::scan(&scan_path);
        let _ = tx.send(root);
    });

    let display_path = path.display().to_string();
    let start = Instant::now();

    loop {
        let elapsed = start.elapsed().as_secs_f32();
        terminal.draw(|f| {
            ui::draw_scanning(
                f,
                &display_path,
                elapsed,
                tree::scanned_files(),
                tree::scanned_bytes(),
            )
        })?;

        if let Ok(root) = rx.try_recv() {
            return Ok(Some(root));
        }

        if event::poll(FRAME_INTERVAL)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                {
                    return Ok(None);
                }
            }
        }

        if let Ok(root) = rx.try_recv() {
            return Ok(Some(root));
        }
    }
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    scan_path: &PathBuf,
) -> io::Result<()> {
    let mut list_state = ListState::default();

    loop {
        list_state.select(Some(app.selected));
        terminal.draw(|f| ui::draw(f, app, &mut list_state))?;

        if event::poll(FRAME_INTERVAL)? {
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
                        *app = App::new(root);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
