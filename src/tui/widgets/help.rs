use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_help(frame: &mut Frame, area: Rect) {
    // Create a centered area for the help panel
    let popup_area = centered_rect(60, 70, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "git-main-diff - Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/k          - Previous hunk"),
        Line::from("  ↓/j          - Next hunk"),
        Line::from("  Tab          - Next file"),
        Line::from("  Shift+Tab    - Previous file"),
        Line::from("  PgUp/PgDn    - Scroll by page"),
        Line::from("  Home/End     - First/last hunk"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Selection:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Space        - Toggle current hunk selection"),
        Line::from("  a            - Select all hunks in current file"),
        Line::from("  n            - Deselect all hunks in current file"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Enter        - Apply selected reversions (stage with git)"),
        Line::from("  r            - Refresh diff"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "What is 'Revert'?",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Reverting UNDOES selected changes by restoring the"),
        Line::from("  main branch version. Selected hunks (green) will be"),
        Line::from("  reversed and staged, ready to commit."),
        Line::from(""),
        Line::from("  Example: If you changed 'v1.0' to 'v2.0' and revert"),
        Line::from("  that hunk, it goes back to 'v1.0'."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Other:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ?            - Toggle this help"),
        Line::from("  q/Esc        - Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to close this help",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    frame.render_widget(help_paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
