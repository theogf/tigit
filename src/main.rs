mod diff;
mod error;
mod git;
mod tui;

use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::Result;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "git-main-diff")]
#[command(about = "Interactive TUI for comparing git HEAD with main branch", long_about = None)]
struct Cli {
    /// Path to git repository (defaults to current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Main branch name (auto-detected if not specified)
    #[arg(short, long)]
    branch: Option<String>,

    /// Skip working directory clean check
    #[arg(long)]
    allow_dirty: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Open repository
    let repo = git::repository::open_repository(cli.path.as_deref())?;

    // Check if working directory is clean (unless --allow-dirty)
    if !cli.allow_dirty {
        if !git::repository::is_working_directory_clean(&repo)? {
            eprintln!("Warning: Working directory is dirty.");
            eprintln!("Some uncommitted changes exist. This tool compares commits, not working directory.");
            eprintln!("Use --allow-dirty to suppress this warning.");
            eprintln!();
        }
    }

    // Detect or use specified main branch
    let main_branch = if let Some(branch) = cli.branch {
        branch
    } else {
        git::repository::detect_main_branch(&repo)?
    };

    // Extract diff
    println!("Comparing HEAD with {}...", main_branch);
    let diff_set = git::diff::extract_diff_set(&repo, &main_branch)?;

    println!(
        "Found {} files with {} total hunks",
        diff_set.files.len(),
        diff_set.total_hunks()
    );

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let result = run_app(&mut terminal, diff_set, &repo);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    diff_set: diff::types::DiffSet,
    repo: &git2::Repository,
) -> Result<()> {
    let mut app = tui::app::App::new(diff_set);

    loop {
        terminal.draw(|f| tui::ui::render(f, &app))?;

        // Handle events
        if let Some(key_event) = tui::events::read_key_event(Duration::from_millis(100))? {
            match app.handle_key_event(key_event) {
                tui::app::AppAction::Quit => {
                    break;
                }
                tui::app::AppAction::ApplyReversions => {
                    let selected = app.diff_set.selected_hunks();
                    if selected == 0 {
                        app.status_message = Some("No hunks selected".to_string());
                    } else {
                        // Apply reversions
                        match git::revert::apply_and_stage_reversions(repo, &app.diff_set) {
                            Ok(count) => {
                                app.status_message =
                                    Some(format!("Applied {} reversions and staged", count));
                                // After successful application, we should probably exit or refresh
                                // For now, let's just show the message
                            }
                            Err(e) => {
                                app.status_message = Some(format!("Error: {}", e));
                            }
                        }
                    }
                }
                tui::app::AppAction::Continue => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
