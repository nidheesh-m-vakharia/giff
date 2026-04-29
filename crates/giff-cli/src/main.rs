mod cli;
mod commands;
mod config;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, StackCommands};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::New { branch } => commands::new::run(&branch),
        Commands::Checkout { target } => commands::checkout::run(&target),
        Commands::Next => commands::checkout::run_next(),
        Commands::Prev => commands::checkout::run_prev(),
        Commands::Push => commands::push::run(),
        Commands::Sync { r#continue } => commands::sync::run(r#continue),
        Commands::Log => commands::log::run(),
        Commands::Status => commands::status::run(),
        Commands::Stack { command } => match command {
            StackCommands::Reorder => commands::stack::reorder::run(),
            StackCommands::Squash { frame } => commands::stack::squash::run(&frame),
            StackCommands::Drop { frame } => commands::stack::drop::run(&frame),
            StackCommands::Land { method } => commands::stack::land::run(&method),
        },
    }
}
