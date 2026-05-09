//! `giff-runner` — single-tenant self-hostable service that maintains stack metadata for one
//! or more GitHub repos, reacts to events, and (when configured) auto-merges the bottom frame
//! of stacks once they're approved.
//!
//! Architecture:
//! ```text
//!   GitHub webhook ─────▶  POST /webhook/github  ─┐
//!                                                  ▼
//!   poll every 15min  ─▶  worker task         ──▶ reconcile.rs
//!                                                  │  (upsert + retarget children
//!                                                  │   of merged frames + auto-merge)
//!                                                  ▼
//!                                              SQLite DB  ◀── HTTP API (read-only)
//! ```
//!
//! Webhooks are the primary signal; polling is a long-interval safety net for missed events.
//! Both code paths funnel through `reconcile.rs` so behaviour is identical regardless of source.

pub mod api;
pub mod config;
pub mod db;
pub mod grouping;
pub mod reconcile;
pub mod retry;
pub mod webhook;
pub mod worker;
