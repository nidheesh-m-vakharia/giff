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
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use std::io;

pub fn run() -> Result<()> {
    let store_path = find_stack_store_path()?;
    let mut store = read_stack_store(&store_path)?;
    let backend_git = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend_git.current_branch()?;

    let stack_id = store
        .find_stack_for_branch(&current)
        .map(|(s, _)| s.id.clone())
        .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;

    let mut frames: Vec<String> = {
        let stack = store.stacks.iter().find(|s| s.id == stack_id).unwrap();
        stack
            .ordered_frames()
            .iter()
            .map(|f| f.branch.clone())
            .collect()
    };
    let mut cursor: usize = 0;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_tui(&mut terminal, &mut frames, &mut cursor);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    match result? {
        TuiOutcome::Confirm => {
            // Apply new order to store
            let s = store.stacks.iter_mut().find(|s| s.id == stack_id).unwrap();
            let frame_map: std::collections::HashMap<String, _> = s
                .frames
                .iter()
                .cloned()
                .map(|f| (f.branch.clone(), f))
                .collect();
            let mut reordered: Vec<_> = frames.iter().map(|b| frame_map[b].clone()).collect();
            // Fix parent pointers: bottom has None, each subsequent has previous frame's id
            for i in 0..reordered.len() {
                reordered[i].parent = if i == 0 {
                    None
                } else {
                    Some(reordered[i - 1].id.clone())
                };
            }
            s.frames = reordered;
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
    frames: &mut [String],
    cursor: &mut usize,
) -> Result<TuiOutcome> {
    loop {
        terminal.draw(|f| {
            let items: Vec<ListItem> = frames
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let style = if i == *cursor {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(name.as_str())).style(style)
                })
                .collect();
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Reorder Stack (↑↓ move, Enter confirm, q quit)"),
            );
            let mut state = ListState::default();
            state.select(Some(*cursor));
            f.render_stateful_widget(list, f.size(), &mut state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if *cursor > 0 {
                        frames.swap(*cursor, *cursor - 1);
                        *cursor -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *cursor + 1 < frames.len() {
                        frames.swap(*cursor, *cursor + 1);
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
