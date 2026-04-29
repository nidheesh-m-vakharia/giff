use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "giff", about = "Stacked diffs for GitHub", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize giff config
    Init,
    /// Create a new stack frame on top of the current branch
    New { branch: String },
    /// Navigate to a frame by name or position
    Checkout { target: String },
    /// Navigate to the frame above
    Next,
    /// Navigate to the frame below
    Prev,
    /// Open or update PRs for all frames in the stack
    Push,
    /// Rebase stack onto updated trunk (prompts on conflict)
    Sync {
        #[arg(long)]
        r#continue: bool,
    },
    /// Print the current stack with PR status
    Log,
    /// Show current frame, dirty state, and PR link
    Status,
    /// Advanced stack operations
    Stack {
        #[command(subcommand)]
        command: StackCommands,
    },
}

#[derive(Subcommand)]
pub enum StackCommands {
    /// Interactively reorder frames
    Reorder,
    /// Squash a frame into the one below
    Squash { frame: String },
    /// Remove a frame and restack above frames
    Drop { frame: String },
    /// Merge the bottom frame PR and promote the rest
    Land {
        /// Merge method: merge, squash, or rebase (default: merge)
        #[arg(long, default_value = "merge")]
        method: String,
    },
}
