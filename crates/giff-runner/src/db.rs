//! SQLite storage. Single connection serialised behind a `Mutex` (rusqlite's `Connection`
//! isn't `Sync`); for our scale (a handful of writes per minute, occasional read bursts)
//! this is perfectly adequate. Switch to a pool if traffic ever justifies it.
//!
//! Schema is bootstrapped idempotently on every open, so existing deployments pick up new
//! columns / indexes by restart. No separate migrations machinery for v1.

use anyhow::{Context, Result};
use giff_github::PullRequest;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS repos (
    slug            TEXT PRIMARY KEY,
    last_synced_at  INTEGER,
    last_error      TEXT
);

CREATE TABLE IF NOT EXISTS pulls (
    repo        TEXT NOT NULL,
    number      INTEGER NOT NULL,
    title       TEXT NOT NULL,
    state       TEXT NOT NULL,
    merged      INTEGER NOT NULL,
    draft       INTEGER NOT NULL,
    head_ref    TEXT NOT NULL,
    base_ref    TEXT NOT NULL,
    body        TEXT,
    html_url    TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    seen_at     INTEGER NOT NULL,
    PRIMARY KEY (repo, number)
);
CREATE INDEX IF NOT EXISTS idx_pulls_repo_state ON pulls(repo, state);

CREATE TABLE IF NOT EXISTS events (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    repo      TEXT NOT NULL,
    pr_number INTEGER,
    kind      TEXT NOT NULL,
    detail    TEXT,
    at        INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_events_at ON events(at DESC);

-- Retry queue. Failed PR-update / merge operations land here and get re-tried on a
-- backoff schedule. Rows are deleted on success or after the abandonment threshold —
-- the audit trail of attempts lives in the `events` table.
--
-- The natural-key uniqueness on (kind, repo, pr_number, payload) means re-deciding the
-- same job (e.g. reconcile re-enqueues an already-pending retarget) is a no-op rather
-- than producing duplicate work.
CREATE TABLE IF NOT EXISTS retry_jobs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    kind            TEXT NOT NULL,
    repo            TEXT NOT NULL,
    pr_number       INTEGER NOT NULL,
    payload         TEXT NOT NULL,
    attempts        INTEGER NOT NULL DEFAULT 0,
    last_error      TEXT,
    next_attempt_at INTEGER NOT NULL,
    created_at      INTEGER NOT NULL,
    UNIQUE (kind, repo, pr_number, payload)
);
CREATE INDEX IF NOT EXISTS idx_retry_jobs_next ON retry_jobs(next_attempt_at);
"#;

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PullSnapshot {
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub state: String,
    pub merged: bool,
    pub draft: bool,
    pub head_ref: String,
    pub base_ref: String,
    pub body: Option<String>,
    pub html_url: String,
    pub updated_at: String,
    pub seen_at: i64,
}

#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub slug: String,
    pub last_synced_at: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: i64,
    pub repo: String,
    pub pr_number: Option<u64>,
    pub kind: String,
    pub detail: Option<String>,
    pub at: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RetryRow {
    pub id: i64,
    pub kind: String,
    pub repo: String,
    pub pr_number: u64,
    pub payload: String,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub next_attempt_at: i64,
    pub created_at: i64,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("opening sqlite at {}", path.display()))?;
        // WAL is the right journaling mode for a server with concurrent reads + writes.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(SCHEMA).context("running schema bootstrap")?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Run a closure with the locked connection on a blocking thread pool. This is the only
    /// way the rest of the code touches the DB — keeps the async/blocking boundary obvious.
    async fn with_conn<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Connection) -> Result<R> + Send + 'static,
        R: Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let guard = conn.lock().expect("db mutex poisoned");
            f(&guard)
        })
        .await
        .context("blocking task panicked")?
    }

    pub async fn upsert_repo(&self, slug: String) -> Result<()> {
        self.with_conn(move |c| {
            c.execute(
                "INSERT OR IGNORE INTO repos (slug) VALUES (?1)",
                params![slug],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn mark_repo_synced(&self, slug: String, at: i64) -> Result<()> {
        self.with_conn(move |c| {
            c.execute(
                "UPDATE repos SET last_synced_at = ?2, last_error = NULL WHERE slug = ?1",
                params![slug, at],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn mark_repo_error(&self, slug: String, err: String) -> Result<()> {
        self.with_conn(move |c| {
            c.execute(
                "UPDATE repos SET last_error = ?2 WHERE slug = ?1",
                params![slug, err],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn list_repo_status(&self) -> Result<Vec<RepoStatus>> {
        self.with_conn(move |c| {
            let mut stmt =
                c.prepare("SELECT slug, last_synced_at, last_error FROM repos ORDER BY slug")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(RepoStatus {
                        slug: row.get(0)?,
                        last_synced_at: row.get(1)?,
                        last_error: row.get(2)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
        .await
    }

    pub async fn upsert_pull(&self, repo: String, pr: PullRequest, seen_at: i64) -> Result<()> {
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO pulls (repo, number, title, state, merged, draft, head_ref,
                                    base_ref, body, html_url, updated_at, seen_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(repo, number) DO UPDATE SET
                    title=excluded.title,
                    state=excluded.state,
                    merged=excluded.merged,
                    draft=excluded.draft,
                    head_ref=excluded.head_ref,
                    base_ref=excluded.base_ref,
                    body=excluded.body,
                    html_url=excluded.html_url,
                    updated_at=excluded.updated_at,
                    seen_at=excluded.seen_at",
                params![
                    repo,
                    pr.number as i64,
                    pr.title,
                    pr.state,
                    pr.merged as i64,
                    pr.draft as i64,
                    pr.head.r#ref,
                    pr.base.r#ref,
                    pr.body,
                    pr.html_url,
                    pr.updated_at,
                    seen_at,
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn get_pull(&self, repo: String, number: u64) -> Result<Option<PullSnapshot>> {
        self.with_conn(move |c| {
            let row = c
                .query_row(
                    "SELECT repo, number, title, state, merged, draft, head_ref, base_ref,
                            body, html_url, updated_at, seen_at
                     FROM pulls WHERE repo = ?1 AND number = ?2",
                    params![repo, number as i64],
                    snapshot_from_row,
                )
                .optional()?;
            Ok(row)
        })
        .await
    }

    pub async fn list_pulls(&self, repo: String) -> Result<Vec<PullSnapshot>> {
        self.with_conn(move |c| {
            let mut stmt = c.prepare(
                "SELECT repo, number, title, state, merged, draft, head_ref, base_ref,
                        body, html_url, updated_at, seen_at
                 FROM pulls WHERE repo = ?1 ORDER BY number",
            )?;
            let rows = stmt
                .query_map(params![repo], snapshot_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
        .await
    }

    pub async fn list_all_pulls(&self) -> Result<Vec<PullSnapshot>> {
        self.with_conn(move |c| {
            let mut stmt = c.prepare(
                "SELECT repo, number, title, state, merged, draft, head_ref, base_ref,
                        body, html_url, updated_at, seen_at
                 FROM pulls ORDER BY repo, number",
            )?;
            let rows = stmt
                .query_map([], snapshot_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
        .await
    }

    pub async fn record_event(
        &self,
        repo: String,
        pr_number: Option<u64>,
        kind: String,
        detail: Option<String>,
        at: i64,
    ) -> Result<()> {
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO events (repo, pr_number, kind, detail, at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![repo, pr_number.map(|n| n as i64), kind, detail, at],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn list_events(&self, since: Option<i64>, limit: usize) -> Result<Vec<Event>> {
        self.with_conn(move |c| {
            let (sql, args): (&str, Vec<Box<dyn rusqlite::ToSql>>) = match since {
                Some(s) => (
                    "SELECT id, repo, pr_number, kind, detail, at
                     FROM events WHERE at > ?1 ORDER BY at DESC LIMIT ?2",
                    vec![Box::new(s), Box::new(limit as i64)],
                ),
                None => (
                    "SELECT id, repo, pr_number, kind, detail, at
                     FROM events ORDER BY at DESC LIMIT ?1",
                    vec![Box::new(limit as i64)],
                ),
            };
            let mut stmt = c.prepare(sql)?;
            let rows = stmt
                .query_map(rusqlite::params_from_iter(args.iter().map(|a| a.as_ref())), |row| {
                    Ok(Event {
                        id: row.get(0)?,
                        repo: row.get(1)?,
                        pr_number: row.get::<_, Option<i64>>(2)?.map(|n| n as u64),
                        kind: row.get(3)?,
                        detail: row.get(4)?,
                        at: row.get(5)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
        .await
    }

    /// Insert a retry job. If a row with the same `(kind, repo, pr_number, payload)` already
    /// exists, leave it alone — preserves the running `attempts` count so we don't reset
    /// progress every time a polling cycle re-discovers the same failure.
    pub async fn enqueue_retry(
        &self,
        kind: String,
        repo: String,
        pr_number: u64,
        payload: String,
        next_attempt_at: i64,
        created_at: i64,
    ) -> Result<()> {
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO retry_jobs (kind, repo, pr_number, payload, next_attempt_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (kind, repo, pr_number, payload) DO NOTHING",
                params![kind, repo, pr_number as i64, payload, next_attempt_at, created_at],
            )?;
            Ok(())
        })
        .await
    }

    /// Pop up to `limit` jobs whose `next_attempt_at <= now`, oldest first. Single-worker
    /// model — no lock-and-claim semantics needed.
    pub async fn claim_ready_retries(&self, now: i64, limit: usize) -> Result<Vec<RetryRow>> {
        self.with_conn(move |c| {
            let mut stmt = c.prepare(
                "SELECT id, kind, repo, pr_number, payload, attempts, last_error,
                        next_attempt_at, created_at
                 FROM retry_jobs
                 WHERE next_attempt_at <= ?1
                 ORDER BY next_attempt_at ASC
                 LIMIT ?2",
            )?;
            let rows = stmt
                .query_map(params![now, limit as i64], retry_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
        .await
    }

    pub async fn list_retry_jobs(&self, limit: usize) -> Result<Vec<RetryRow>> {
        self.with_conn(move |c| {
            let mut stmt = c.prepare(
                "SELECT id, kind, repo, pr_number, payload, attempts, last_error,
                        next_attempt_at, created_at
                 FROM retry_jobs
                 ORDER BY next_attempt_at ASC
                 LIMIT ?1",
            )?;
            let rows = stmt
                .query_map(params![limit as i64], retry_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
        .await
    }

    pub async fn complete_retry(&self, id: i64) -> Result<()> {
        self.with_conn(move |c| {
            c.execute("DELETE FROM retry_jobs WHERE id = ?1", params![id])?;
            Ok(())
        })
        .await
    }

    /// Bump attempts + reschedule. Used after a failed attempt that hasn't yet exceeded
    /// the abandonment threshold.
    pub async fn bump_retry(
        &self,
        id: i64,
        last_error: String,
        next_attempt_at: i64,
    ) -> Result<()> {
        self.with_conn(move |c| {
            c.execute(
                "UPDATE retry_jobs
                 SET attempts = attempts + 1,
                     last_error = ?2,
                     next_attempt_at = ?3
                 WHERE id = ?1",
                params![id, last_error, next_attempt_at],
            )?;
            Ok(())
        })
        .await
    }

    #[cfg(test)]
    pub async fn count_retry_jobs(&self) -> Result<i64> {
        self.with_conn(move |c| {
            Ok(c.query_row("SELECT COUNT(*) FROM retry_jobs", [], |r| r.get::<_, i64>(0))?)
        })
        .await
    }
}

fn retry_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RetryRow> {
    Ok(RetryRow {
        id: row.get(0)?,
        kind: row.get(1)?,
        repo: row.get(2)?,
        pr_number: row.get::<_, i64>(3)? as u64,
        payload: row.get(4)?,
        attempts: row.get(5)?,
        last_error: row.get(6)?,
        next_attempt_at: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn snapshot_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PullSnapshot> {
    Ok(PullSnapshot {
        repo: row.get(0)?,
        number: row.get::<_, i64>(1)? as u64,
        title: row.get(2)?,
        state: row.get(3)?,
        merged: row.get::<_, i64>(4)? != 0,
        draft: row.get::<_, i64>(5)? != 0,
        head_ref: row.get(6)?,
        base_ref: row.get(7)?,
        body: row.get(8)?,
        html_url: row.get(9)?,
        updated_at: row.get(10)?,
        seen_at: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use giff_github::BranchRef;
    use tempfile::TempDir;

    fn sample_pr(num: u64, body: &str) -> PullRequest {
        PullRequest {
            number: num,
            html_url: format!("https://github.com/o/r/pull/{}", num),
            state: "open".into(),
            merged: false,
            title: format!("PR #{}", num),
            body: Some(body.into()),
            head: BranchRef {
                r#ref: format!("feat/{}", num),
            },
            base: BranchRef { r#ref: "main".into() },
            draft: false,
            updated_at: "2026-04-29T00:00:00Z".into(),
        }
    }

    #[tokio::test]
    async fn round_trips_a_pull() {
        let td = TempDir::new().unwrap();
        let db = Db::open(&td.path().join("test.db")).unwrap();
        db.upsert_repo("o/r".into()).await.unwrap();
        db.upsert_pull("o/r".into(), sample_pr(1, "body"), 100)
            .await
            .unwrap();

        let got = db.get_pull("o/r".into(), 1).await.unwrap().unwrap();
        assert_eq!(got.title, "PR #1");
        assert_eq!(got.head_ref, "feat/1");
        assert_eq!(got.base_ref, "main");
    }

    #[tokio::test]
    async fn upsert_replaces_existing() {
        let td = TempDir::new().unwrap();
        let db = Db::open(&td.path().join("test.db")).unwrap();
        db.upsert_repo("o/r".into()).await.unwrap();
        db.upsert_pull("o/r".into(), sample_pr(1, "first"), 100)
            .await
            .unwrap();
        let mut updated = sample_pr(1, "second");
        updated.merged = true;
        updated.state = "closed".into();
        db.upsert_pull("o/r".into(), updated, 200).await.unwrap();

        let got = db.get_pull("o/r".into(), 1).await.unwrap().unwrap();
        assert!(got.merged);
        assert_eq!(got.state, "closed");
        assert_eq!(got.body.as_deref(), Some("second"));
        assert_eq!(got.seen_at, 200);
    }

    #[tokio::test]
    async fn retry_jobs_dedupe_on_natural_key() {
        let td = TempDir::new().unwrap();
        let db = Db::open(&td.path().join("test.db")).unwrap();
        db.enqueue_retry("retarget".into(), "o/r".into(), 1, r#"{"base":"main"}"#.into(), 100, 100)
            .await
            .unwrap();
        db.enqueue_retry("retarget".into(), "o/r".into(), 1, r#"{"base":"main"}"#.into(), 200, 200)
            .await
            .unwrap();
        assert_eq!(db.count_retry_jobs().await.unwrap(), 1);

        // Different payload → new row.
        db.enqueue_retry("retarget".into(), "o/r".into(), 1, r#"{"base":"feat/a"}"#.into(), 300, 300)
            .await
            .unwrap();
        assert_eq!(db.count_retry_jobs().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn claim_ready_filters_by_next_attempt_at() {
        let td = TempDir::new().unwrap();
        let db = Db::open(&td.path().join("test.db")).unwrap();
        db.enqueue_retry("retarget".into(), "o/r".into(), 1, "{}".into(), 50, 50)
            .await
            .unwrap();
        db.enqueue_retry("retarget".into(), "o/r".into(), 2, "{}".into(), 200, 50)
            .await
            .unwrap();
        let ready = db.claim_ready_retries(100, 10).await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].pr_number, 1);
    }

    #[tokio::test]
    async fn bump_increments_attempts_and_reschedules() {
        let td = TempDir::new().unwrap();
        let db = Db::open(&td.path().join("test.db")).unwrap();
        db.enqueue_retry("retarget".into(), "o/r".into(), 1, "{}".into(), 50, 50)
            .await
            .unwrap();
        let job = db.list_retry_jobs(10).await.unwrap().into_iter().next().unwrap();
        db.bump_retry(job.id, "boom".into(), 500).await.unwrap();
        let after = db.list_retry_jobs(10).await.unwrap().into_iter().next().unwrap();
        assert_eq!(after.attempts, 1);
        assert_eq!(after.last_error.as_deref(), Some("boom"));
        assert_eq!(after.next_attempt_at, 500);
    }

    #[tokio::test]
    async fn events_are_ordered_newest_first() {
        let td = TempDir::new().unwrap();
        let db = Db::open(&td.path().join("test.db")).unwrap();
        db.record_event("o/r".into(), Some(1), "merged".into(), None, 100)
            .await
            .unwrap();
        db.record_event("o/r".into(), Some(2), "merged".into(), None, 200)
            .await
            .unwrap();
        db.record_event("o/r".into(), Some(3), "merged".into(), None, 150)
            .await
            .unwrap();

        let events = db.list_events(None, 10).await.unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].pr_number, Some(2));
        assert_eq!(events[1].pr_number, Some(3));
        assert_eq!(events[2].pr_number, Some(1));
    }
}
