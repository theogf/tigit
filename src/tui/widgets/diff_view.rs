use crate::diff::types::{Hunk, LineOrigin};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_diff_hunk(
    frame: &mut Frame,
    area: Rect,
    hunk: &Hunk,
    is_selected: bool,
    side: DiffSide,
) {
    let mut lines = Vec::new();

    // Add hunk header
    let header_style = if is_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };

    lines.push(Line::from(vec![Span::styled(&hunk.header, header_style)]));

    // Render lines based on which side we're showing
    for line in &hunk.lines {
        match side {
            DiffSide::Left => {
                // Left side: show context and deletions
                match line.origin {
                    LineOrigin::Context | LineOrigin::Deletion => {
                        let style = match line.origin {
                            LineOrigin::Deletion => Style::default().fg(Color::Red),
                            LineOrigin::Context => Style::default().fg(Color::Gray),
                            _ => Style::default(),
                        };

                        let lineno = line
                            .old_lineno
                            .map(|n| format!("{:4} ", n))
                            .unwrap_or_else(|| "     ".to_string());

                        let prefix = line.origin.to_char();
                        let content = line.content.trim_end_matches('\n');

                        lines.push(Line::from(vec![
                            Span::styled(lineno, Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("{} {}", prefix, content), style),
                        ]));
                    }
                    LineOrigin::Addition => {
                        // Blank line for additions (only on right side)
                        lines.push(Line::from(vec![Span::raw("")]));
                    }
                }
            }
            DiffSide::Right => {
                // Right side: show context and additions
                match line.origin {
                    LineOrigin::Context | LineOrigin::Addition => {
                        let style = match line.origin {
                            LineOrigin::Addition => Style::default().fg(Color::Green),
                            LineOrigin::Context => Style::default().fg(Color::Gray),
                            _ => Style::default(),
                        };

                        let lineno = line
                            .new_lineno
                            .map(|n| format!("{:4} ", n))
                            .unwrap_or_else(|| "     ".to_string());

                        let prefix = line.origin.to_char();
                        let content = line.content.trim_end_matches('\n');

                        lines.push(Line::from(vec![
                            Span::styled(lineno, Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("{} {}", prefix, content), style),
                        ]));
                    }
                    LineOrigin::Deletion => {
                        // Blank line for deletions (only on left side)
                        lines.push(Line::from(vec![Span::raw("")]));
                    }
                }
            }
        }
    }

    let title = match side {
        DiffSide::Left => " Main Branch ",
        DiffSide::Right => " HEAD ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(if is_selected {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });

    let paragraph = Paragraph::new(lines).block(block);

    frame.render_widget(paragraph, area);
}

pub enum DiffSide {
    Left,
    Right,
}
