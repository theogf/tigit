use super::app::App;
use super::widgets::{diff_view, help};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: header, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    render_header(frame, chunks[0], app);
    render_content(frame, chunks[1], app);
    render_footer(frame, chunks[2], app);

    // Render help overlay if needed
    if app.show_help {
        help::render_help(frame, size);
    }

    // Render confirmation dialog if needed
    if app.show_confirmation {
        help::render_confirmation(frame, size, app.diff_set.selected_hunks());
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        "tigit: Working Directory vs {} ({} files, {} hunks, {} selected)",
        app.diff_set.main_branch_name,
        app.diff_set.files.len(),
        app.diff_set.total_hunks(),
        app.diff_set.selected_hunks()
    );

    let header = Paragraph::new(Line::from(vec![Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(file) = app.get_current_file() {
        // File info bar
        let file_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let file_info = format!(
            "File: {} [{}] (Hunk {}/{})",
            file.path,
            file.status.as_str(),
            app.current_hunk + 1,
            file.hunks.len()
        );

        let file_bar = Paragraph::new(Line::from(vec![Span::styled(
            file_info,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(file_bar, file_chunks[0]);

        // Side-by-side diff view
        if let Some(hunk) = app.get_current_hunk() {
            let diff_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(file_chunks[1]);

            // Left side: Main branch
            diff_view::render_diff_hunk(
                frame,
                diff_chunks[0],
                hunk,
                hunk.selected,
                diff_view::DiffSide::Left,
                app.vertical_scroll,
                app.horizontal_scroll,
            );

            // Right side: HEAD
            diff_view::render_diff_hunk(
                frame,
                diff_chunks[1],
                hunk,
                hunk.selected,
                diff_view::DiffSide::Right,
                app.vertical_scroll,
                app.horizontal_scroll,
            );
        } else {
            let no_hunks = Paragraph::new("No hunks in this file")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(no_hunks, file_chunks[1]);
        }
    } else {
        let no_files =
            Paragraph::new("No files to display").block(Block::default().borders(Borders::ALL));
        frame.render_widget(no_files, area);
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        "↑/↓: Navigate  Space: Select (green=will revert)  Enter: Undo selected changes  q: Quit  ?: Help".to_string()
    };

    let footer = Paragraph::new(Line::from(vec![Span::styled(
        help_text,
        Style::default().fg(Color::Gray),
    )]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}
