// crates/giff-cli/src/commands/stack/reorder.rs
use crate::config::{find_stack_store_path, read_stack_store, write_stack_store};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use giff_git::{GitBackend, ShellGitBackend};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io;

/// Visible row in the reorder TUI — branch + PR snapshot.
struct Row {
    branch: String,
    pr_number: Option<u64>,
}

pub fn run() -> Result<()> {
    let store_path = find_stack_store_path()?;
    let mut store = read_stack_store(&store_path)?;
    let backend_git = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend_git.current_branch()?;

    let stack_id = store
        .find_stack_for_branch(&current)
        .map(|(s, _)| s.id.clone())
        .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;

    let stack_name: String;
    let trunk: String;
    {
        let stack = store.stacks.iter().find(|s| s.id == stack_id).unwrap();
        stack.validate()?;
        if !stack.is_linear() {
            anyhow::bail!(
                "stack `{}` is a tree — `giff stack reorder` only works on linear stacks. \
                 Restructure the tree with `giff stack drop` / `giff stack squash` first.",
                stack.name
            );
        }
        stack_name = stack.name.clone();
        trunk = stack.trunk.clone();
    }

    let mut rows: Vec<Row> = {
        let stack = store.stacks.iter().find(|s| s.id == stack_id).unwrap();
        stack
            .ordered_frames()
            .iter()
            .map(|f| Row {
                branch: f.branch.clone(),
                pr_number: f.pr_number,
            })
            .collect()
    };
    let mut cursor: usize = 0;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_tui(&mut terminal, &stack_name, &trunk, &current, &mut rows, &mut cursor);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    match result? {
        TuiOutcome::Confirm => {
            let s = store.stacks.iter_mut().find(|s| s.id == stack_id).unwrap();
            let frame_map: std::collections::HashMap<String, _> = s
                .frames
                .iter()
                .cloned()
                .map(|f| (f.branch.clone(), f))
                .collect();
            let mut reordered: Vec<_> = rows.iter().map(|r| frame_map[&r.branch].clone()).collect();
            for i in 0..reordered.len() {
                reordered[i].parent = if i == 0 {
                    None
                } else {
                    Some(reordered[i - 1].id.clone())
                };
            }
            s.frames = reordered;
            s.validate()?;
            write_stack_store(&store_path, &store)?;
            println!("Stack reordered. Run `giff push` to update PRs.");
        }
        TuiOutcome::Cancel => {
            println!("Reorder cancelled.");
        }
    }

    Ok(())
}

enum TuiOutcome {
    Confirm,
    Cancel,
}

fn run_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    stack_name: &str,
    trunk: &str,
    current_branch: &str,
    rows: &mut [Row],
    cursor: &mut usize,
) -> Result<TuiOutcome> {
    loop {
        terminal.draw(|f| {
            let area = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),                 // top status: stack header
                    Constraint::Min(3),                    // list
                    Constraint::Length(1),                 // bottom hints
                ])
                .split(area);

            // Top header — what stack are we reordering, what's the trunk, how many frames.
            let header = Paragraph::new(Line::from(vec![
                Span::styled(" stack ", Style::default().fg(Color::Black).bg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(stack_name.to_string(), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("trunk:", Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::raw(trunk.to_string()),
                Span::raw("  "),
                Span::styled("frames:", Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::raw(format!("{}", rows.len())),
            ]));
            f.render_widget(header, chunks[0]);

            // List with rich rows.
            let items: Vec<ListItem> = rows
                .iter()
                .enumerate()
                .map(|(i, row)| build_row(i, row, current_branch, *cursor))
                .collect();

            let block_title = Line::from(vec![
                Span::raw(" "),
                Span::styled("Reorder", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
            ]);
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(block_title)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
            let mut state = ListState::default();
            state.select(Some(*cursor));
            f.render_stateful_widget(list, chunks[1], &mut state);

            // Bottom hints bar.
            let hints = Paragraph::new(Line::from(vec![
                Span::styled(" ↑↓ ", Style::default().fg(Color::Cyan)),
                Span::raw("move row  "),
                Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
                Span::raw("apply  "),
                Span::styled(" Esc / q ", Style::default().fg(Color::Cyan)),
                Span::raw("cancel "),
            ]))
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
            f.render_widget(hints, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if *cursor > 0 {
                        rows.swap(*cursor, *cursor - 1);
                        *cursor -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *cursor + 1 < rows.len() {
                        rows.swap(*cursor, *cursor + 1);
                        *cursor += 1;
                    }
                }
                KeyCode::Enter => return Ok(TuiOutcome::Confirm),
                KeyCode::Char('q') | KeyCode::Esc => return Ok(TuiOutcome::Cancel),
                _ => {}
            }
        }
    }
}

fn build_row(idx: usize, row: &Row, current_branch: &str, cursor: usize) -> ListItem<'static> {
    let is_cursor = idx == cursor;
    let is_current = row.branch == current_branch;

    let cursor_marker = if is_cursor { "▸ " } else { "  " };
    let position = format!("{:>2}.", idx + 1);
    let pr = match row.pr_number {
        Some(n) => format!(" #{}", n),
        None => "  (no PR)".to_string(),
    };
    let here = if is_current { "  ← here" } else { "" };

    let row_style = if is_cursor {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let branch_style = if is_cursor {
        Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)
    } else if is_current {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    let pr_style = if is_cursor {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let line = Line::from(vec![
        Span::raw(cursor_marker),
        Span::styled(position, Style::default().fg(if is_cursor { Color::Black } else { Color::DarkGray })),
        Span::raw("  "),
        Span::styled(row.branch.clone(), branch_style),
        Span::styled(pr, pr_style),
        Span::styled(here, Style::default().fg(Color::Yellow)),
    ]);
    ListItem::new(line).style(row_style)
}
