# giff Web Dashboard — Design Spec

**Date:** 2026-04-29
**Scope:** SvelteKit SPA dashboard for browsing stacked diffs and full PR content. GitHub is the source of truth; the app is a read-heavy helper with no write operations except the token/repo config in `localStorage`.

---

## Architecture

**Location:** `apps/web/` in the monorepo root (not inside `crates/`).

**Stack:**
- SvelteKit with `@sveltejs/adapter-static` (pure SPA, no server)
- shadcn-svelte for UI primitives
- All data from GitHub REST API v3, called directly from the browser
- Auth token and `owner/repo` stored in `localStorage`

**No backend. No build-time data fetching.** Every page fetches on mount using the stored token.

---

## Pages

### `/` — Stack Dashboard

Lists all open PRs for the configured repo, grouped into stacks by parsing the `giff` JSON block embedded in each PR description:

```
```giff
{ "stack_id": "...", "frame_id": "...", "position": 2, "total": 4 }
```
```

PRs without a `giff` block are shown in an "Ungrouped PRs" section beneath the stacks.

Each stack is rendered as a vertical chain:

```
stack: auth-refactor
● main
│
◉ feat/auth-base       PR #42  open  ✓ CI  2 reviews
│
◉ feat/auth-tokens     PR #43  open  ✗ CI  0 reviews
│
◉ feat/auth-middleware PR #44  draft
```

Clicking any frame row navigates to `/pr/[number]`.

**Data sources:**
- `GET /repos/:owner/:repo/pulls?state=open&per_page=100` — all open PRs
- Parse `RemoteStackMeta` blocks from PR bodies to group into stacks
- `GET /repos/:owner/:repo/commits/:ref/check-runs` — CI status per frame (best-effort)

---

### `/pr/[number]` — Full PR View

The main content page. Three-column layout on desktop, stacked on mobile:

**Left sidebar:** stack context — where this PR sits in the stack, links to adjacent frames.

**Main area (tabs):**

1. **Conversation** — timeline of all activity in order:
   - General comments (`GET /repos/:owner/:repo/issues/:number/comments`)
   - Review submissions with body (`GET /repos/:owner/:repo/pulls/:number/reviews`)
   - Rendered as a chat-style thread with avatars, timestamps, markdown bodies

2. **Files changed** — full diff view:
   - File list with +/- stats on the left; click to jump
   - Each file rendered as a unified diff with syntax highlighting (shiki)
   - Inline review comments (`GET /repos/:owner/:repo/pulls/:number/comments`) shown at the relevant line
   - Collapsed by default for files with >200 lines changed; expand button

3. **Commits** — list of commits in this PR (`GET /repos/:owner/:repo/pulls/:number/commits`), each showing SHA, message, author

**Right sidebar:** PR metadata — state, draft badge, base→head branch, reviewers + their decision (approved / changes requested / pending), labels, milestone.

---

### `/settings` — Settings

Single form, no navigation sidebar:

- **GitHub token** — `<input type="password">`, saved to `localStorage` on submit. Shows a masked preview of the stored value if one exists. Link to GitHub's token creation page.
- **Repository** — `owner/repo` text input, saved to `localStorage`. On save, validates by calling `GET /repos/:owner/:repo` and showing success/error inline.
- **Clear data** — button to wipe `localStorage` and reset the app.

No OAuth, no server-side storage. Token never leaves the browser.

---

## Data Flow

```
localStorage { token, repo }
        ↓
GitHub REST API (Authorization: Bearer <token>)
        ↓
SvelteKit load functions (client-side only)
        ↓
Svelte stores (stacks, prs, settings)
        ↓
Page components → shadcn-svelte UI
```

A `$settings` store is the single source of truth for `{ token, repo }`. All API calls read from it. If token or repo is missing, every page redirects to `/settings`.

---

## Components

| Component | Purpose |
|-----------|---------|
| `StackChain` | Renders the `● main / │ / ◉ frame` visual for a stack |
| `FrameRow` | Single row in a stack chain — branch, PR number, state badge, CI badge |
| `PRStatusBadge` | Colored pill: open / draft / merged / closed |
| `CIBadge` | Green ✓ / red ✗ / grey pending based on check-runs |
| `DiffView` | File-by-file diff with shiki syntax highlighting |
| `InlineComment` | A review comment anchored to a diff line |
| `ConversationThread` | Timeline of comments + reviews in chronological order |
| `ReviewDecision` | Approved / changes requested / pending badge per reviewer |
| `TokenForm` | Settings form with masked token input + repo validation |

---

## Error States

- **No token/repo set** → redirect to `/settings` with a banner explaining why
- **401 from GitHub** → banner: "Token invalid or expired — update it in Settings"
- **403 rate limit** → banner with reset time from `X-RateLimit-Reset` header
- **404 repo** → settings page inline error: "Repository not found or token lacks access"
- **PR with no giff block** → shown in "Ungrouped PRs" on dashboard, full PR view still works

---

## Project Structure

```
apps/web/
├── src/
│   ├── lib/
│   │   ├── api/
│   │   │   ├── github.ts       # typed wrappers for all GitHub API calls
│   │   │   └── stack.ts        # RemoteStackMeta parsing, stack grouping logic
│   │   ├── stores/
│   │   │   └── settings.ts     # { token, repo } store backed by localStorage
│   │   └── components/
│   │       ├── StackChain.svelte
│   │       ├── FrameRow.svelte
│   │       ├── PRStatusBadge.svelte
│   │       ├── CIBadge.svelte
│   │       ├── DiffView.svelte
│   │       ├── InlineComment.svelte
│   │       ├── ConversationThread.svelte
│   │       ├── ReviewDecision.svelte
│   │       └── TokenForm.svelte
│   └── routes/
│       ├── +layout.svelte       # nav, settings redirect guard
│       ├── +page.svelte         # / dashboard
│       ├── pr/[number]/
│       │   └── +page.svelte
│       └── settings/
│           └── +page.svelte
├── package.json
├── svelte.config.js
├── vite.config.ts
└── tailwind.config.ts
```

---

## Out of Scope (this spec)

- Writing back to GitHub (commenting, merging, approving) — read-only for now
- giff-wasm integration — future enhancement
- Authentication via OAuth — token-only in v1
- Notifications or real-time updates — manual refresh only
