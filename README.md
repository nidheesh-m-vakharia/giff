# giff

Stacked diffs for GitHub. Work on multiple dependent pull requests as a linear stack of branches, with automatic PR management and conflict-aware rebasing.

```
● main
│
◉ feat/auth-base       PR #42 [open]   ← you are here
│
◉ feat/auth-tokens     PR #43 [open]
│
◉ feat/auth-middleware PR #44 [draft]
```

## Install

**From crates.io (requires Rust ≥ 1.75):**

```sh
cargo install giffstack
```

The crate is published as `giffstack`; the binary it ships is `giff`.

**From a local checkout** (for hacking on it):

```sh
git clone https://github.com/nidheesh-m-vakharia/giff
cd giff
cargo install --path crates/giff-cli
```

**Auth:**

Set a GitHub personal access token with `repo` scope:

```sh
export GITHUB_TOKEN=ghp_...
# or add it to ~/.config/giff/config.toml (run `giff init` to create the skeleton)
```

## Usage

```sh
# Start a stack from main
giff new feat/auth-base       # creates branch, starts tracking it
giff new feat/auth-tokens     # stacks on top of current frame
giff new feat/auth-middleware # stacks on top again

# Navigate
giff prev / giff next              # move up/down the stack
giff checkout <branch-or-position> # jump to frame by name or number

# Open / update PRs for all frames
giff push

# See the stack with live PR status
giff log

# See where you are
giff status

# Rebase the whole stack onto updated main
giff sync

# If a conflict occurs, resolve it, then:
git rebase --continue
giff sync --continue

# Open the web dashboard in your default browser (PRs, diffs, comments, all yours)
giff dashboard
# → http://local.giffstack.com:51743 (or http://localhost:51743 if your DNS blocks it)

# Advanced
giff stack reorder          # interactive TUI reorder (↑↓ to move, Enter to confirm)
giff stack squash <branch>  # squash a frame into the one below
giff stack drop <branch>    # remove a frame and relink the one above
giff stack land [--method merge|squash|rebase]  # merge bottom PR, promote rest
```

`giff dashboard` runs an embedded HTTP server on a localhost port and opens your
default browser to it. The same SvelteKit app that runs at giffstack.com — your
GitHub token stays in `localStorage`, no data leaves your machine. Ctrl-C to stop.

## Configuration

`~/.config/giff/config.toml` (created by `giff init`):

```toml
[github]
token = "ghp_..."
base_url = "https://api.github.com"  # override for GitHub Enterprise

[defaults]
trunk = "main"    # base branch for new stacks
draft_prs = true  # open PRs as drafts by default
```

Stack metadata is stored in `.git/stacked.toml` — inside `.git/` so it's never committed.

## How it works

Each stack frame is a regular git branch. PRs target the frame below them (not `main` directly), so reviewers see only the diff for that layer. On `giff push`, each PR description gets a small JSON block embedded in a fenced code block — this lets the stack be reconstructed from GitHub alone, with no local file required.

## Contributing

```sh
git clone https://github.com/nidheesh-m-vakharia/giff
cd giff
cargo build
cargo test
```

Pull requests welcome. Please open an issue first for anything beyond small bug fixes.

## License

MIT
