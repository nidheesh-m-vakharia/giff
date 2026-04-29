# giff CLI — Design Spec

**Date:** 2026-04-27
**Scope:** Rust-based CLI tool (`giff`) for stacked diffs with GitHub PR management. Phase 1 of a larger system (web UI + Docker to follow).

---

## Overview

`giff` is a Rust CLI that implements stacked diffs using a branch-per-frame model (Graphite/ghstack style). Each "frame" in a stack is a git branch. PRs are opened against the frame below them, not against `main` directly. The tool manages local stack metadata, syncs it to GitHub PR descriptions, and handles restacking when the base branch updates.

---

## Architecture: Layered Crate Workspace

```
giff/
├── Cargo.toml              # workspace root
├── crates/
│   ├── giff-core/          # pure Rust, no I/O — stack models, metadata, algorithms
│   ├── giff-git/           # GitBackend trait + two impls (shell-out, gitoxide)
│   ├── giff-github/        # ForgeBackend trait + GitHub API impl
│   ├── giff-cli/           # binary: clap-based CLI, uses shell-out backend
│   └── giff-wasm/          # wasm-bindgen bindings, uses gitoxide backend
└── docs/
```

**Key rule:** `giff-core` has zero I/O — no `std::fs`, no `std::process`, no network. Enforced by keeping its `Cargo.toml` deps to `serde`, `thiserror`, and `toml` only. Anything touching the outside world lives in `giff-git`, `giff-github`, or the platform crates.

This separation means `giff-core` compiles to WASM cleanly for the future web UI.

---

## Core Data Model (`giff-core`)

```rust
// A single frame in the stack — maps 1:1 to a git branch
struct StackFrame {
    id: FrameId,           // stable UUID, survives branch renames
    branch: String,        // e.g. "feat/auth-step-2"
    parent: Option<FrameId>, // points to frame below; None for the bottom frame (which targets trunk)
    pr_number: Option<u64>,
    description: Option<String>,
}

// The full stack
struct Stack {
    id: StackId,
    name: String,          // human name, e.g. "auth-refactor"
    trunk: String,         // base branch, e.g. "main"
    frames: Vec<StackFrame>, // ordered bottom → top
}

// Stored in .git/stacked.toml — one file, multiple stacks
struct StackStore {
    stacks: Vec<Stack>,
}

// Synced into each PR description as a fenced JSON block
// ```giff
// { "stack_id": "...", "frame_id": "...", "position": 2, "total": 4 }
// ```
struct RemoteStackMeta {
    stack_id: StackId,
    frame_id: FrameId,
    position: usize,
    total: usize,
}
```

`StackStore` serializes to `.git/stacked.toml` locally. On push, `RemoteStackMeta` is embedded in each PR description as a fenced JSON block so the future web UI and teammates can reconstruct the stack from GitHub alone — no local file required.

---

## Git Backend Trait (`giff-git`)

```rust
trait GitBackend {
    // Read
    fn current_branch(&self) -> Result<String>;
    fn branch_exists(&self, name: &str) -> Result<bool>;
    fn commit_log(&self, branch: &str, base: &str) -> Result<Vec<Commit>>;
    fn merge_base(&self, a: &str, b: &str) -> Result<String>;

    // Write
    fn create_branch(&self, name: &str, from: &str) -> Result<()>;
    fn checkout(&self, branch: &str) -> Result<()>;
    fn rebase(&self, branch: &str, onto: &str) -> Result<RebaseOutcome>;
    fn push(&self, branch: &str, force: bool) -> Result<()>;
}

enum RebaseOutcome {
    Clean,
    Conflict { frame: String, hints: Vec<String> },
}
```

**Two implementations:**

- `ShellGitBackend` — shells out to the system `git` binary via `std::process::Command`. Used by `giff-cli`. Inherits the user's git config, credentials, SSH agents, and GPG signing with zero extra work.
- `GitoxideBackend` — pure Rust using `gix`. Used by `giff-wasm`. Covers reads fully; writes use gitoxide's native implementations.

`RebaseOutcome::Conflict` is how `giff sync` pauses per-frame and prompts the user rather than aborting the whole stack.

---

## GitHub API Integration (`giff-github`)

```rust
trait ForgeBackend {
    fn create_pr(&self, params: CreatePrParams) -> Result<PullRequest>;
    fn update_pr(&self, number: u64, params: UpdatePrParams) -> Result<PullRequest>;
    fn get_pr(&self, number: u64) -> Result<PullRequest>;
    fn pr_status(&self, number: u64) -> Result<PrStatus>; // checks, review state
}

struct GitHubForge {
    token: String,         // from GITHUB_TOKEN env or ~/.config/giff/config.toml
    repo: String,          // "owner/repo"
    base_url: String,      // supports GitHub Enterprise (custom base URL)
}
```

**`giff push` flow:**
1. For each frame bottom→top: create PR if no `pr_number`, update PR if one exists
2. Set each PR's base to the frame below's branch (bottom frame targets `trunk`)
3. Embed `RemoteStackMeta` JSON block into each PR description
4. Print stack summary with PR URLs on completion

**Auth:** `GITHUB_TOKEN` env var first, then `~/.config/giff/config.toml`. Token-based only in v1, no OAuth flow.

**GitHub Enterprise:** supported via configurable `base_url` — same API surface, different host.

HTTP client: `ureq` (sync, no async runtime, small binary footprint).

---

## CLI Command Surface (`giff-cli`)

Built with `clap` (derive API), `ratatui` for the `reorder` TUI.

```
# Stack creation & navigation
giff new <branch-name>         # create new frame on top of current, checkout it
giff checkout <branch-or-pos>  # navigate by name or position (e.g. giff checkout 2)
giff prev / giff next          # shorthand navigation

# Sync & push
giff push                      # open/update PRs for all frames, force-push branches
giff sync                      # pull trunk, restack entire stack (prompt on conflict per frame)
giff sync --continue           # resume after manually resolving a conflict

# Inspection
giff log                       # pretty-print stack with PR status
giff status                    # current frame position, dirty state, PR link

# Advanced (namespaced)
giff stack reorder             # interactive reorder frames (ratatui TUI)
giff stack squash <frame>      # squash a frame into the one below
giff stack drop <frame>        # remove a frame, restack above frames
giff stack land                # merge bottom frame PR, promote rest down one
```

**`giff log` output:**
```
● main
│
◉ feat/auth-base       PR #42 [open]   ← you are here
│
◉ feat/auth-tokens     PR #43 [open]
│
◉ feat/auth-middleware PR #44 [draft]
```

---

## Configuration & Metadata Files

**Global** — `~/.config/giff/config.toml`
```toml
[github]
token = "ghp_..."
base_url = "https://api.github.com"  # override for GitHub Enterprise

[defaults]
trunk = "main"
draft_prs = true        # open PRs as draft by default
pr_template = ""        # path to PR body template file
```

**Per-repo** — `.git/stacked.toml` (inside `.git/`, never committed)
```toml
[[stacks]]
id = "a1b2c3"
name = "auth-refactor"
trunk = "main"

[[stacks.frames]]
id = "f1"
branch = "feat/auth-base"
pr_number = 42

[[stacks.frames]]
id = "f2"
branch = "feat/auth-tokens"
pr_number = 43
parent = "f1"
```

- `.git/stacked.toml` lives inside `.git/` — git ignores it automatically, no `.gitignore` entry needed
- `giff init` writes the global config skeleton on first run
- Per-repo config is created lazily on first `giff new`
- `giff-core` owns the serde types and parsing logic — pure `serde` + `toml`, no I/O. `giff-cli` is responsible for reading/writing the actual files off disk and passing the parsed bytes to `giff-core`

---

## Error Handling

```rust
#[derive(thiserror::Error, Debug)]
enum GiffError {
    #[error("no stack found for branch `{0}`")]
    NoStack(String),
    #[error("rebase conflict in frame `{0}` — resolve and run `giff sync --continue`")]
    RebaseConflict(String),
    #[error("GitHub API error: {0}")]
    Forge(String),
    #[error("git error: {0}")]
    Git(String),
    #[error("config error: {0}")]
    Config(String),
}
```

Each crate defines errors via `thiserror`. `giff-cli` maps all errors to human-readable messages via `anyhow` at the binary boundary — no raw error types reach the terminal.

---

## Testing Strategy

| Crate | Approach |
|-------|----------|
| `giff-core` | Pure unit tests — no mocking needed (no I/O) |
| `giff-git` | Integration tests against real git repos in `tempdir` |
| `giff-github` | Tests against a mock HTTP server (`wiremock`) |
| `giff-cli` | End-to-end tests using `assert_cmd` + temp git repos |

---

## What This Spec Does Not Cover

- Web UI (separate spec)
- Docker container / self-hosted server (separate spec)
- WASM bindings detail (covered at web UI spec time)
- Multi-user / team workflows beyond PR description syncing
