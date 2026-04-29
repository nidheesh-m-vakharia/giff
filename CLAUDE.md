# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Run tests for a single crate
cargo test -p giff-core
cargo test -p giff-cli

# Run a specific test
cargo test -p giff-core stack_frame_bottom_has_no_parent

# Build the CLI binary only
cargo build -p giff-cli

# Run the CLI in dev
cargo run -p giff-cli -- <subcommand>
```

## Architecture

This is a Cargo workspace of five crates with strict layering:

```
giff-core     ← pure Rust, zero I/O (serde/toml/thiserror only)
giff-git      ← GitBackend trait + ShellGitBackend impl (shells out to system git)
giff-github   ← ForgeBackend trait + GitHubForge impl (ureq HTTP, sync, no async)
giff-cli      ← clap binary, reads/writes files, calls all three above
giff-wasm     ← wasm-bindgen stub; exports giff-core types to JS
```

**The critical rule:** `giff-core` must stay I/O-free. No `std::fs`, `std::process`, or network calls ever belong there. This keeps it WASM-compilable for the future web UI.

## Core Data Model

Stack metadata lives in `.git/stacked.toml` (inside `.git/`, so git ignores it). `giff-core` owns the serde types; `giff-cli/src/config.rs` owns all file I/O.

- `Stack` contains an ordered `Vec<StackFrame>` (bottom → top). Bottom frame has `parent: None`; each subsequent frame points to the one below by `FrameId`.
- On `giff push`, each PR description gets a fenced `RemoteStackMeta` JSON block so the stack can be reconstructed from GitHub alone (no local file required).

## `giff sync` Conflict Resume

When `giff sync` hits a rebase conflict it **leaves git in the conflict state** (does not abort) and writes `.git/giff_sync_resume.json` containing `{ stack_id, resume_from_idx, original_branch }`. After the user resolves the conflict and runs `git rebase --continue`, they run `giff sync --continue` to restack the remaining frames and delete the resume file.

If a resume file already exists and the user runs `giff sync` (without `--continue`), the command errors out with instructions rather than silently restarting.

## Key File Locations

| Path | Purpose |
|------|---------|
| `crates/giff-cli/src/config.rs` | File I/O for `.git/stacked.toml` and `~/.config/giff/config.toml` |
| `crates/giff-cli/src/commands/` | One file per CLI command; `stack/` subdir for advanced ops |
| `crates/giff-core/src/types.rs` | All data model structs (`Stack`, `StackFrame`, `RemoteStackMeta`) |
| `crates/giff-core/src/algorithms.rs` | Pure traversal helpers (`frame_above`, `frame_below`, `ordered_frames`) |
| `crates/giff-git/src/shell.rs` | `ShellGitBackend` — every git operation; also `is_rebase_in_progress()` |
| `crates/giff-github/src/github.rs` | `GitHubForge` — PR create/update/get via `ureq` |

## Auth & Config

`GITHUB_TOKEN` env var takes precedence over `~/.config/giff/config.toml`. Token-only in v1 (no OAuth). GitHub Enterprise is supported via configurable `base_url` in the global config.

## Testing Conventions

- `giff-core`: pure unit tests, no mocking needed
- `giff-cli`: end-to-end tests use `assert_cmd` + `tempfile`; dev-deps include `assert_cmd`, `predicates`, `tempfile`
- GitHub API tests should use a mock HTTP server (`wiremock`) — not yet wired up
