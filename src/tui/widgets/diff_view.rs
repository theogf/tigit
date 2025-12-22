use crate::diff::types::{DiffLine, Hunk, LineOrigin};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Represents an aligned pair of lines for side-by-side rendering
#[derive(Clone)]
struct AlignedLine {
    left: Option<DiffLine>,
    right: Option<DiffLine>,
}

/// Process hunk lines into aligned pairs for side-by-side display
fn create_aligned_lines(hunk: &Hunk) -> Vec<AlignedLine> {
    let mut aligned = Vec::new();
    let mut i = 0;

    while i < hunk.lines.len() {
        let line = &hunk.lines[i];

        match line.origin {
            LineOrigin::Context => {
                // Context lines appear on both sides
                aligned.push(AlignedLine {
                    left: Some(line.clone()),
                    right: Some(line.clone()),
                });
                i += 1;
            }
            LineOrigin::Deletion => {
                // Collect consecutive deletions
                let mut deletions = vec![line.clone()];
                i += 1;
                while i < hunk.lines.len() && hunk.lines[i].origin == LineOrigin::Deletion {
                    deletions.push(hunk.lines[i].clone());
                    i += 1;
                }

                // Collect consecutive additions that follow
                let mut additions = Vec::new();
                while i < hunk.lines.len() && hunk.lines[i].origin == LineOrigin::Addition {
                    additions.push(hunk.lines[i].clone());
                    i += 1;
                }

                // Pair up deletions and additions
                let max_len = deletions.len().max(additions.len());
                for j in 0..max_len {
                    aligned.push(AlignedLine {
                        left: deletions.get(j).cloned(),
                        right: additions.get(j).cloned(),
                    });
                }
            }
            LineOrigin::Addition => {
                // Addition without preceding deletion
                // Collect consecutive additions
                let mut additions = vec![line.clone()];
                i += 1;
                while i < hunk.lines.len() && hunk.lines[i].origin == LineOrigin::Addition {
                    additions.push(hunk.lines[i].clone());
                    i += 1;
                }

                // Show additions on right side only
                for addition in additions {
                    aligned.push(AlignedLine {
                        left: None,
                        right: Some(addition),
                    });
                }
            }
        }
    }

    aligned
}

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

    // Create aligned line pairs
    let aligned_lines = create_aligned_lines(hunk);

    // Render based on which side we're showing
    for aligned in &aligned_lines {
        match side {
            DiffSide::Left => {
                if let Some(line) = &aligned.left {
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
                } else {
                    // Blank line (right side has content)
                    lines.push(Line::from(vec![Span::raw("")]));
                }
            }
            DiffSide::Right => {
                if let Some(line) = &aligned.right {
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
                } else {
                    // Blank line (left side has content)
                    lines.push(Line::from(vec![Span::raw("")]));
                }
            }
        }
    }

    let title = match side {
        DiffSide::Left => " Merge Base (before your changes) ",
        DiffSide::Right => " HEAD (your changes) ",
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
