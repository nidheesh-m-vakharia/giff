use crate::config::{find_stack_store_path, read_stack_store};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use giff_core::StackFrame;
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

pub fn run(target: &str) -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let current = backend.current_branch()?;

    let branch = if let Ok(pos) = target.parse::<usize>() {
        let (stack, _) = store
            .find_stack_for_branch(&current)
            .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;
        if !stack.is_linear() {
            anyhow::bail!(
                "stack `{}` is a tree — `giff checkout <position>` only works for linear stacks. \
                 Use `giff checkout <branch-name>` instead.",
                stack.name
            );
        }
        let frames = stack.ordered_frames();
        let idx = pos
            .checked_sub(1)
            .ok_or_else(|| anyhow::anyhow!("position must be >= 1"))?;
        frames
            .get(idx)
            .ok_or_else(|| anyhow::anyhow!("position {} out of range", pos))?
            .branch
            .clone()
    } else {
        target.to_string()
    };

    backend.checkout(&branch)?;
    println!("Checked out: {}", branch);
    Ok(())
}

pub fn run_next() -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let (stack, frame) = store
        .find_stack_for_branch(&current)
        .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;
    let children = stack.children(&frame.id);
    let above = match children.len() {
        0 => anyhow::bail!("already at top of stack"),
        1 => children[0].clone(),
        _ => {
            // Tree fork — let the user pick interactively. Falls back to a plain error
            // listing the children if no TTY is available (CI, piped invocations).
            match pick_child_interactive(&frame.branch, &children)? {
                Some(picked) => picked,
                None => return Ok(()), // user cancelled
            }
        }
    };
    backend.checkout(&above.branch)?;
    println!("Checked out: {}", above.branch);
    Ok(())
}

/// Open a tiny ratatui list for the user to pick which child of the current frame to descend
/// into. Returns the chosen frame, or `None` if the user cancelled. Errors only on terminal
/// setup failure.
fn pick_child_interactive(
    parent_branch: &str,
    children: &[&StackFrame],
) -> Result<Option<StackFrame>> {
    // Bail with a friendly error if stdin/stdout aren't a TTY (e.g. piped). The TUI would
    // either hang or render garbage in those cases.
    use std::io::IsTerminal;
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        let names: Vec<&str> = children.iter().map(|c| c.branch.as_str()).collect();
        anyhow::bail!(
            "frame `{}` has {} children — `giff next` is ambiguous (no TTY for picker). Pick one with `giff checkout <branch>`: {}",
            parent_branch,
            children.len(),
            names.join(", ")
        );
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let owned: Vec<StackFrame> = children.iter().map(|c| (*c).clone()).collect();
    let mut cursor = 0usize;
    let result = pick_loop(&mut terminal, parent_branch, &owned, &mut cursor);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(result?)
}

fn pick_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    parent_branch: &str,
    children: &[StackFrame],
    cursor: &mut usize,
) -> Result<Option<StackFrame>> {
    loop {
        terminal.draw(|f| {
            let area = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(1)])
                .split(area);

            let items: Vec<ListItem> = children
                .iter()
                .enumerate()
                .map(|(i, frame)| {
                    let pr = match frame.pr_number {
                        Some(n) => format!(" #{}", n),
                        None => String::new(),
                    };
                    let style = if i == *cursor {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    let line = Line::from(vec![
                        Span::raw(if i == *cursor { "▸ " } else { "  " }),
                        Span::styled(frame.branch.clone(), Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(pr, Style::default().fg(Color::DarkGray)),
                    ]);
                    ListItem::new(line).style(style)
                })
                .collect();

            let title = format!(" frame `{}` has {} children — pick one ", parent_branch, children.len());
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
            let mut state = ListState::default();
            state.select(Some(*cursor));
            f.render_stateful_widget(list, chunks[0], &mut state);

            let hints = Paragraph::new(Line::from(vec![
                Span::styled(" ↑↓ ", Style::default().fg(Color::Cyan)),
                Span::raw("move  "),
                Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
                Span::raw("checkout  "),
                Span::styled(" Esc ", Style::default().fg(Color::Cyan)),
                Span::raw("cancel "),
            ]))
            .style(Style::default().bg(Color::DarkGray));
            f.render_widget(hints, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if *cursor > 0 {
                        *cursor -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *cursor + 1 < children.len() {
                        *cursor += 1;
                    }
                }
                KeyCode::Enter => return Ok(Some(children[*cursor].clone())),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                _ => {}
            }
        }
    }
}

pub fn run_prev() -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let (stack, frame) = store
        .find_stack_for_branch(&current)
        .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;
    let below = stack
        .frame_below(&frame.id)
        .ok_or_else(|| anyhow::anyhow!("already at bottom of stack"))?;
    backend.checkout(&below.branch)?;
    println!("Checked out: {}", below.branch);
    Ok(())
}
