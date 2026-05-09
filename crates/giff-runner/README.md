# giff-runner

Single-tenant self-hostable service for `giff` stacks. Reacts to GitHub webhooks (with
polling as a safety net), persists stack metadata in SQLite, and optionally auto-merges
the bottom frame of stacks once they're approved.

## Quick start (docker compose, from workspace root)

```sh
mkdir -p config
cp crates/giff-runner/example-config.toml config/runner.toml
# edit config/runner.toml — at minimum set the [[repos]] slug
echo 'GITHUB_TOKEN=ghp_...' > .env
docker compose up -d
curl http://localhost:8080/healthz   # → ok
curl http://localhost:8080/repos
```

## Local dev

```sh
GITHUB_TOKEN=ghp_... cargo run -p giff-runner -- --config crates/giff-runner/example-config.toml
```

## Webhooks (recommended)

For instant reactions instead of 15-minute polling:

1. Make the runner reachable from the public internet. Easiest path: [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/) or [Tailscale Funnel](https://tailscale.com/kb/1223/funnel).
2. Generate a webhook secret: `openssl rand -hex 32`. Put it in your repo's config block as `webhook_secret = "..."`.
3. In your repo's GitHub settings → Webhooks → Add webhook:
   - **Payload URL:** `https://your-runner.example/webhook/github`
   - **Content type:** `application/json`
   - **Secret:** the value from step 2
   - **Events:** select *Pull requests* and *Pull request reviews*

Without a `webhook_secret`, deliveries for that repo are rejected (they'd be unauthenticated) and the runner falls back to polling-only mode for it.

## API

Read-only HTTP, no auth (put it behind a reverse proxy, Cloudflare Access, or Tailscale ACL).

| Method | Path                  | Description                                          |
|--------|-----------------------|------------------------------------------------------|
| GET    | `/healthz`            | Liveness check                                       |
| GET    | `/repos`              | Configured repos + sync status                       |
| GET    | `/stacks`             | All reconstructed stacks across tracked repos        |
| GET    | `/stacks/:id`         | One stack                                            |
| GET    | `/events?since=&limit=` | Audit log (merged / retargeted / auto_merged / retry_succeeded / abandoned / error) |
| GET    | `/retry-queue?limit=` | Outstanding retry jobs (kind, attempts, last_error, next_attempt_at) |
| POST   | `/sync`               | Force an immediate poll cycle + flush ready retries  |
| POST   | `/webhook/github`     | GitHub webhook receiver (HMAC-verified)              |

## Auto-merge gate

When `auto_merge = true` for a repo, the runner will merge the bottom (root) PR of any
single-root stack as soon as GitHub reports the PR is `mergeable`. Approving-review
enforcement is delegated to your repo's branch protection rules — if those aren't set up
to require reviews, this will merge unreviewed code. Turn it off if that's not what you
want.

## Retry queue

Failed `update_pr` and `merge_pr` calls don't disappear into the void — they land in a
`retry_jobs` SQLite table and get re-tried by a dedicated worker on a backoff schedule:
`30s → 1m → 5m → 15m → 30m → 1h → 4h`, then abandoned (~6.5 hours total). Re-deciding the
same job (e.g. polling re-discovering the same failure) is de-duped by the natural-key
UNIQUE constraint, so attempt counts don't reset.

Inspect the queue at any time:

```sh
curl http://localhost:8080/retry-queue
sqlite3 /data/state.db 'SELECT * FROM retry_jobs'
```

Successful retries log `retry_succeeded` events; abandoned ones log `abandoned`. Both
visible at `GET /events`.

## What lands where

- `state.db` (SQLite) — persisted at `/data/state.db`. Inspect with `sqlite3 state.db`.
- Tables: `repos` (per-repo sync status), `pulls` (PR snapshots), `events` (audit log),
  `retry_jobs` (outstanding retries).
- All reconstruction (stacks, trees, topological order) happens in code from `pulls.body`
  on each request — there is no derived state to keep in sync.
