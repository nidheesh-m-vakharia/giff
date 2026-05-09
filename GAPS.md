# Gaps & Deferred Work

What's not built, and why I didn't build it. Grouped by surface. Each item lists what's
missing, the workaround that's currently in place (if any), and a rough priority.

Priority key:
- **P1** — operationally important; you'll hit this in real use
- **P2** — quality/UX issue; not blocking but worth doing before serious adoption
- **P3** — nice to have; defer until evidence the gap matters

---

## giff-runner

### Auth on the HTTP API
- **Gap:** All routes (`/repos`, `/stacks`, `/events`, `/retry-queue`, `/sync`) are unauthenticated.
- **Workaround:** Place behind a reverse proxy with auth (Caddy + basic-auth, Cloudflare Access, Tailscale Funnel ACLs). The webhook route IS authenticated via HMAC.
- **Priority:** P1 if you ever expose the runner publicly. Behind a private tunnel, P3.

### Approving-review enforcement for auto-merge
- **Gap:** `auto_merge = true` merges as soon as GitHub reports `mergeable: true`. It does *not* check for approving reviews — that's delegated to your repo's branch protection rules.
- **Workaround:** Configure required reviews on your trunk branch in GitHub settings before turning on auto-merge.
- **Priority:** P1. Without branch protection, this auto-merges unreviewed code.

### GitHub App support (vs PAT)
- **Gap:** PAT-only. A long-lived `repo`-scope token in the runner's env is a high blast radius if compromised.
- **Workaround:** Rotate the PAT regularly; restrict it to least-privilege scopes; run the runner on locked-down infra.
- **Priority:** P2 for self-hosted small teams; P1 if you go to Phase 2 (hosted SaaS).

### Per-job pre-check before retry execution
- **Gap:** When a retry job fires, we just run the action again. If state changed in the meantime (e.g. the PR was merged through a parallel path), the retry fails-til-abandonment instead of recognizing "the action is no longer needed."
- **Workaround:** Bounded by the abandonment threshold (~6.5 hours of retries, then dropped).
- **Priority:** P3. Wastes ~7 API calls per stale job. Add a `get_pr` pre-check before `update_pr`/`merge_pr` if it ever becomes a problem.

### Per-repo backoff schedule override
- **Gap:** Retry backoff (`30s → 1m → 5m → 15m → 30m → 1h → 4h`) is hardcoded globally.
- **Workaround:** None.
- **Priority:** P3 — the default is reasonable.

### Manual per-job retry trigger
- **Gap:** No `POST /retry-queue/:id/run-now` endpoint. `POST /sync` flushes *everything* ready.
- **Workaround:** `POST /sync` covers the common case. For one specific job, `UPDATE retry_jobs SET next_attempt_at = strftime('%s','now') WHERE id = ?` via sqlite then `POST /sync`.
- **Priority:** P3.

### Pagination on `list_open_pulls`
- **Gap:** Caps at 100 per page; no follow-up requests. Repos with >100 open PRs are partially tracked.
- **Workaround:** Stay under 100 open PRs. Realistic for most teams; less realistic for monorepos.
- **Priority:** P2 if you have a busy repo. The fix is straightforward — walk `Link` headers in `giff-github`.

### Rate-limit handling
- **Gap:** No exponential backoff on GitHub 403 rate-limited responses. The runner just logs the error and retries on the next polling cycle.
- **Workaround:** The 15-minute default poll interval makes hitting rate limits unlikely in practice.
- **Priority:** P2 for repos with many webhooks firing under load.

### Metrics / Prometheus endpoint
- **Gap:** No `/metrics` endpoint. Logs are the only operational surface.
- **Workaround:** Tail logs; query SQLite directly.
- **Priority:** P2 once you actually deploy this in production.

### Backup strategy for `state.db`
- **Gap:** No documented backup approach. Lose `/data` and you lose everything except what's reconstructible from GitHub.
- **Workaround:** SQLite WAL is enabled; back up `state.db` + `state.db-wal` together. Most of state is reconstructible from GitHub on a fresh start (PR snapshots get re-fetched on the next poll), but the `events` audit log is gone.
- **Priority:** P2 — document the backup pattern in the README.

### Multi-tenant / multi-team isolation
- **Gap:** One runner = one `GITHUB_TOKEN` = one set of tracked repos. Two teams sharing a runner can read each other's `/events`.
- **Workaround:** Run separate runner instances per team.
- **Priority:** P3 for self-hosted; P1 if you go to Phase 3 SaaS.

### Graceful shutdown for in-flight retries
- **Gap:** SIGTERM stops accepting HTTP and exits. In-flight retry jobs are dropped (and re-tried at next startup since they're persisted, but the current attempt's API call may complete server-side without the runner recording success).
- **Workaround:** The natural-key UNIQUE on `retry_jobs` plus the idempotent action design means re-trying after restart is safe — the worst case is one extra API call per interrupted job.
- **Priority:** P3.

---

## giff-cli

### `giff stack reorder` for tree-shaped stacks
- **Gap:** Refuses with "linear stacks only" — full tree reorder is non-trivial UX (siblings vs ancestors).
- **Workaround:** Restructure trees with `giff stack drop` / `giff stack squash` first, or by manually editing branches.
- **Priority:** P3.

### Pre-commit hook for non-`giff` repos cloned by team members
- **Gap:** The pre-commit hook auto-installs the first time a `giff` command runs in a repo with `.git/stacked.toml`. A teammate cloning a giff-managed repo will not have the hook until they run any giff command.
- **Workaround:** Run any giff command (`giff status` is cheapest) once after cloning.
- **Priority:** P2. Documenting it in a "first-time setup" section of the README would help.

### E2E tests for `giff push` against real GitHub
- **Gap:** `giff-cli` integration tests cover everything except the actual `push` flow against a mock GitHub. There's no test that asserts the parallel push + reconcile + retarget logic works end-to-end.
- **Workaround:** Underlying pieces (`giff-github` wiremock tests, `giff-core` algorithm tests) cover the components. Integration is verified manually.
- **Priority:** P2. Wiremock-based test in `giff-cli/tests/` would catch regressions.

### Concurrency in `giff push` git operations
- **Gap:** Branches are pushed sequentially (parallel API calls only). For stacks with many frames this leaves wall-clock time on the table.
- **Workaround:** Sequential git push is safer (avoids object-DB lock contention) and rarely the bottleneck — GitHub API roundtrips dominate.
- **Priority:** P3.

---

## giff-wasm / giff-web

### `GitoxideBackend` (the original spec gap)
- **Gap:** The spec calls for a gitoxide-backed `GitBackend` impl in `giff-wasm` so the future web UI can perform git ops in the browser. None of it is built.
- **Workaround:** The web app reads from the GitHub REST API only — it never executes git locally. For the current dashboard's purposes this is fine.
- **Priority:** P3 unless/until you want the web UI to manipulate stacks (e.g. drag-and-drop reorder that persists).

### Markdown rendering of PR comments
- **Gap:** Comment bodies render as plain text with `whitespace-pre-wrap`. No GFM, no syntax highlighting in code blocks within comments, no link auto-detection.
- **Workaround:** Acceptable for read-only viewing; users can click through to GitHub for properly-rendered comments.
- **Priority:** P2 for usability. `marked` is ~10KB and a one-call swap.

### Inline review comment anchoring fallback
- **Gap:** Anchors by `comment.line` (right-side post-image). GitHub's older `position` field isn't a fallback — comments on diffs that lack `line` won't anchor.
- **Workaround:** Modern PRs have `line` populated; affects only old PRs.
- **Priority:** P3.

### Web app tests
- **Gap:** Zero tests on the web side. No Vitest, no Playwright.
- **Workaround:** `npm run check` (svelte-check) catches type errors. Build verifies it compiles.
- **Priority:** P2 once the codebase grows. Vitest for `lib/api/` would be the highest-value first investment.

### Dark mode toggle
- **Gap:** CSS variables include a `.dark` palette in `app.css`, but nothing toggles it. The site is light-mode-only in practice.
- **Workaround:** Manually inspect with `<html class="dark">` for testing.
- **Priority:** P3.

### Mobile responsive
- **Gap:** Desktop-first; the sidebar at `w-72` and the PR view's two-column grid don't collapse cleanly under ~640px.
- **Workaround:** Use on a desktop or tablet.
- **Priority:** P3.

---

## Architecture / cross-cutting

### Crate not yet published to crates.io
- **Gap:** The CLI package is named `giffstack` (with `version`, `description`, `license = "MIT"`, and `repository` set), but no `cargo publish` has happened yet. `cargo install giffstack` therefore won't resolve until publication.
- **Workaround:** Install from a local checkout: `cargo install --path crates/giff-cli`. CI is wired up to auto-publish on push to `main` once the `CARGO_REGISTRY_TOKEN` repo secret is set (see `.github/workflows/release.yml`); the four crates publish in dependency order automatically.
- **Priority:** P1 before announcing the project.

### No prebuilt release artifacts
- **Gap:** No standalone CLI binaries (CLI installs via `cargo install`); no published Docker image for the runner; no Tauri bundle published for the dashboard.
- **Workaround:** Build locally — `cargo install`, `docker compose build`, `npm run tauri:build`.
- **Priority:** P2 once there are users beyond the author. CLI binary releases via GitHub Releases would be a small extension to `release.yml`; Docker image push to GHCR is similar; Tauri cross-platform bundles need a separate matrix workflow.

### Observability beyond logs
- **Gap:** No structured tracing export (OpenTelemetry), no error reporting (Sentry-style). When auto-merge fires at 3am and breaks something, you have logs and the SQLite events table; that's it.
- **Workaround:** Logs + events table + SQLite queries.
- **Priority:** P2 after Phase 2.

---

## Phase 2/3 (SaaS path) — explicitly out of scope for now

These were called out during the SaaS-vs-self-hosted discussion. Listed here as a forward-looking checklist; **none of this is built**.

- Multi-tenancy (every table grows a `tenant_id`; SQLite → Postgres)
- GitHub OAuth login flow + session management
- Stripe billing + webhook handling for subscription state
- Org / team model with roles
- Onboarding flow (sign up → connect GitHub → pick repo → land first stack)
- Marketing / pricing / docs site
- Email support funnel
- Status page
- Compliance (SOC 2, GDPR, sales-tax handling)
- Enterprise tier: SSO/SAML, audit logs, SLA, on-prem with paid support

The Phase 1 runner you currently have is the substrate, not the product. Validate it has demand (downloads, issues, PRs from outside) before investing in this stack.
