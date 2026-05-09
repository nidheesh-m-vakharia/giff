mod cli;
mod commands;
mod concurrency;
mod config;
mod hooks;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, StackCommands};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

/// Set `GIT_SSH_COMMAND` so every git invocation in this process uses an SSH
/// ControlMaster. The first git operation establishes the master connection (one
/// passphrase prompt); subsequent operations multiplex over it for free, so `giff push`
/// pushing 8 branches is one prompt, not eight.
///
/// Respects an existing `GIT_SSH_COMMAND` from the user's env (we only set it if unset).
/// On Windows the user's git typically uses native ssh.exe — same flags work; if they
/// don't, OpenSSH ignores the unknown options and behavior is unchanged.
fn setup_ssh_multiplexing() {
    if std::env::var_os("GIT_SSH_COMMAND").is_some() {
        return;
    }
    let dir = std::env::temp_dir();
    // %C is a hash over (host, port, user, IdentityFile) — stable per remote, distinct per repo
    // setup. ControlPersist=120 keeps the master alive 2 minutes after the last command, plenty
    // for a multi-step `giff push`.
    let cmd = format!(
        "ssh -o ControlMaster=auto -o ControlPath={}/giff-ssh-%C -o ControlPersist=120",
        dir.display()
    );
    std::env::set_var("GIT_SSH_COMMAND", cmd);
}

fn run() -> Result<()> {
    setup_ssh_multiplexing();
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::New { branch } => commands::new::run(&branch),
        Commands::Publish {
            message,
            branch,
            all,
        } => commands::publish::run(&message, branch.as_deref(), all),
        Commands::Checkout { target } => commands::checkout::run(&target),
        Commands::Next => commands::checkout::run_next(),
        Commands::Prev => commands::checkout::run_prev(),
        Commands::Commit {
            message,
            amend,
            all,
        } => commands::commit::run(message.as_deref(), amend, all),
        Commands::Push => commands::push::run(),
        Commands::Sync { r#continue } => commands::sync::run(r#continue),
        Commands::Log { all } => commands::log::run(all),
        Commands::Status => commands::status::run(),
        Commands::Stack { command } => match command {
            StackCommands::Reorder => commands::stack::reorder::run(),
            StackCommands::Squash { frame } => commands::stack::squash::run(&frame),
            StackCommands::Drop { frame } => commands::stack::drop::run(&frame),
            StackCommands::Land { method } => commands::stack::land::run(&method),
        },
        Commands::ParentBranch => commands::parent_branch::run(),
    }
}
