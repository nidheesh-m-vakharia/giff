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
    /// Create a new frame AND commit your staged changes in one step.
    /// The message is used both as the commit message and (slugified) as the branch name —
    /// override with `-b`. Conventional-commit prefixes like `feat:` become path segments
    /// in the branch name.
    Publish {
        /// Description of the change. Becomes the commit message and the branch name.
        message: String,
        /// Override the auto-derived branch name.
        #[arg(short = 'b', long)]
        branch: Option<String>,
        /// Stage all modified tracked files before committing (like `git commit -a`).
        #[arg(short = 'a', long = "all")]
        all: bool,
    },
    /// Navigate to a frame by name or position
    Checkout { target: String },
    /// Navigate to the frame above
    Next,
    /// Navigate to the frame below
    Prev,
    /// Create the single commit for the current frame.
    /// One commit per frame is enforced — to add more changes, run `giff new` to start
    /// a new frame on top, or `giff commit --amend` to revise the existing commit.
    Commit {
        /// Commit message (required for new commits; optional with --amend to keep old message)
        #[arg(short, long)]
        message: Option<String>,
        /// Amend the existing commit instead of creating a new one
        #[arg(long)]
        amend: bool,
        /// Stage all modified tracked files before committing (like `git commit -a`)
        #[arg(short = 'a', long = "all")]
        all: bool,
    },
    /// Open or update PRs for all frames in the stack
    Push,
    /// Rebase stack onto updated trunk (prompts on conflict)
    Sync {
        #[arg(long)]
        r#continue: bool,
    },
    /// Print the current stack with PR status. By default hides frames whose PR is closed
    /// or merged — pass `--all` to show every frame.
    Log {
        /// Include frames whose PR is closed or merged.
        #[arg(short = 'a', long)]
        all: bool,
    },
    /// Show current frame, dirty state, and PR link
    Status,
    /// Open the native desktop dashboard.
    /// Requires the companion `giffstack-app` crate (install: `cargo install giffstack-app`).
    Dashboard,
    /// Advanced stack operations
    Stack {
        #[command(subcommand)]
        command: StackCommands,
    },
    /// (internal) Print the parent branch of the current frame for the pre-commit hook.
    /// Exits 0 with empty output when not in a stack so the hook stays best-effort.
    #[command(hide = true)]
    ParentBranch,
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
