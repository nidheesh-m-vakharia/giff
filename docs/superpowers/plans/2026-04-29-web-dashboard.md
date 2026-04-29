# giff Web Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a static SvelteKit SPA at `apps/web/` that lets developers browse their giff stacked diffs, read full PR content (diff + conversation + commits), and manage their GitHub token — with GitHub as the sole source of truth.

**Architecture:** Pure client-side SvelteKit app with `adapter-static`. All data fetched from the GitHub REST API v3 using a token stored in `localStorage`. A `$settings` Svelte store is the single source of truth for `{ token, repo }`; every page redirects to `/settings` if either is missing.

**Tech Stack:** SvelteKit 2, shadcn-svelte, Tailwind CSS 3, shiki (syntax highlighting), TypeScript, `@sveltejs/adapter-static`

---

## File Map

| File | Responsibility |
|------|---------------|
| `apps/web/package.json` | Dependencies and scripts |
| `apps/web/svelte.config.js` | SvelteKit config with adapter-static |
| `apps/web/vite.config.ts` | Vite config |
| `apps/web/tailwind.config.ts` | Tailwind config |
| `apps/web/src/app.html` | HTML shell |
| `apps/web/src/lib/api/github.ts` | Typed fetch wrappers for every GitHub endpoint used |
| `apps/web/src/lib/api/stack.ts` | `RemoteStackMeta` parsing + stack grouping logic |
| `apps/web/src/lib/stores/settings.ts` | `{ token, repo }` store backed by localStorage |
| `apps/web/src/lib/components/PRStatusBadge.svelte` | Colored pill: open / draft / merged / closed |
| `apps/web/src/lib/components/CIBadge.svelte` | ✓ / ✗ / pending based on check-runs result |
| `apps/web/src/lib/components/FrameRow.svelte` | One row in a stack chain |
| `apps/web/src/lib/components/StackChain.svelte` | Full vertical stack visual |
| `apps/web/src/lib/components/ReviewDecision.svelte` | Per-reviewer approval badge |
| `apps/web/src/lib/components/ConversationThread.svelte` | Chronological comment + review timeline |
| `apps/web/src/lib/components/InlineComment.svelte` | Single review comment anchored to a diff line |
| `apps/web/src/lib/components/DiffView.svelte` | File-by-file diff with shiki highlighting + inline comments |
| `apps/web/src/lib/components/TokenForm.svelte` | Settings form: token input + repo validation |
| `apps/web/src/routes/+layout.svelte` | Nav bar + settings redirect guard |
| `apps/web/src/routes/+page.svelte` | `/` dashboard |
| `apps/web/src/routes/pr/[number]/+page.svelte` | Full PR view |
| `apps/web/src/routes/settings/+page.svelte` | Settings page |

---

## Task 1: Scaffold the SvelteKit app

**Files:**
- Create: `apps/web/package.json`
- Create: `apps/web/svelte.config.js`
- Create: `apps/web/vite.config.ts`
- Create: `apps/web/tailwind.config.ts`
- Create: `apps/web/src/app.html`
- Create: `apps/web/src/app.css`
- Create: `apps/web/postcss.config.js`

- [ ] **Step 1: Create the app directory and package.json**

```bash
mkdir -p apps/web/src/lib/api apps/web/src/lib/stores apps/web/src/lib/components apps/web/src/routes/pr/\[number\] apps/web/src/routes/settings
```

Create `apps/web/package.json`:
```json
{
  "name": "giff-web",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "lint": "prettier --check . && eslint ."
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "prettier": "^3.0.0",
    "prettier-plugin-svelte": "^3.0.0",
    "svelte": "^4.0.0",
    "svelte-check": "^3.0.0",
    "tailwindcss": "^3.4.0",
    "tslib": "^2.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0"
  },
  "dependencies": {
    "shiki": "^1.0.0"
  },
  "type": "module"
}
```

- [ ] **Step 2: Create svelte.config.js**

```js
// apps/web/svelte.config.js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: 'index.html'
    })
  }
};

export default config;
```

- [ ] **Step 3: Create vite.config.ts**

```ts
// apps/web/vite.config.ts
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()]
});
```

- [ ] **Step 4: Create tsconfig.json**

```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true
  }
}
```

- [ ] **Step 5: Create Tailwind config**

```ts
// apps/web/tailwind.config.ts
import type { Config } from 'tailwindcss';

export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {}
  },
  plugins: []
} satisfies Config;
```

- [ ] **Step 6: Create postcss.config.js**

```js
// apps/web/postcss.config.js
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {}
  }
};
```

- [ ] **Step 7: Create app.html**

```html
<!-- apps/web/src/app.html -->
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <link rel="icon" href="%sveltekit.assets%/favicon.png" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    %sveltekit.head%
  </head>
  <body data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
```

- [ ] **Step 8: Create app.css**

```css
/* apps/web/src/app.css */
@tailwind base;
@tailwind components;
@tailwind utilities;
```

- [ ] **Step 9: Install dependencies and verify**

```bash
cd apps/web && npm install
npm run check
```

Expected: no errors (will warn about missing routes — that's fine at this stage).

- [ ] **Step 10: Commit**

```bash
git add apps/web/
git commit -m "feat(web): scaffold SvelteKit app with Tailwind"
```

---

## Task 2: Install shadcn-svelte and add base UI primitives

**Files:**
- Modify: `apps/web/package.json`
- Create: `apps/web/components.json`
- Create: `apps/web/src/lib/utils.ts`

- [ ] **Step 1: Install shadcn-svelte CLI and initialize**

```bash
cd apps/web
npx shadcn-svelte@latest init
```

When prompted:
- Style: **Default**
- Base color: **Slate**
- CSS variables: **Yes**

This creates `components.json` and updates `app.css` with CSS variables.

- [ ] **Step 2: Add the components we need**

```bash
npx shadcn-svelte@latest add badge button card separator tabs tooltip
```

These install to `apps/web/src/lib/components/ui/`.

- [ ] **Step 3: Verify utils.ts was created by shadcn**

`apps/web/src/lib/utils.ts` should exist with a `cn()` helper. If not, create it:

```ts
// apps/web/src/lib/utils.ts
import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

And install the missing deps:
```bash
npm install clsx tailwind-merge
```

- [ ] **Step 4: Commit**

```bash
git add apps/web/
git commit -m "feat(web): add shadcn-svelte with badge, button, card, separator, tabs"
```

---

## Task 3: Settings store and GitHub API client

**Files:**
- Create: `apps/web/src/lib/stores/settings.ts`
- Create: `apps/web/src/lib/api/github.ts`

- [ ] **Step 1: Create the settings store**

```ts
// apps/web/src/lib/stores/settings.ts
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface Settings {
  token: string;
  repo: string; // "owner/repo"
}

function createSettingsStore() {
  const initial: Settings = browser
    ? {
        token: localStorage.getItem('giff_token') ?? '',
        repo: localStorage.getItem('giff_repo') ?? ''
      }
    : { token: '', repo: '' };

  const { subscribe, set, update } = writable<Settings>(initial);

  return {
    subscribe,
    save(values: Settings) {
      if (browser) {
        localStorage.setItem('giff_token', values.token);
        localStorage.setItem('giff_repo', values.repo);
      }
      set(values);
    },
    clear() {
      if (browser) {
        localStorage.removeItem('giff_token');
        localStorage.removeItem('giff_repo');
      }
      set({ token: '', repo: '' });
    }
  };
}

export const settings = createSettingsStore();
```

- [ ] **Step 2: Create the GitHub API client**

```ts
// apps/web/src/lib/api/github.ts

export interface GithubPR {
  number: number;
  title: string;
  state: 'open' | 'closed';
  draft: boolean;
  body: string | null;
  html_url: string;
  head: { ref: string; sha: string };
  base: { ref: string };
  user: { login: string; avatar_url: string };
  requested_reviewers: Array<{ login: string; avatar_url: string }>;
  labels: Array<{ name: string; color: string }>;
  milestone: { title: string } | null;
  created_at: string;
  updated_at: string;
  merged_at: string | null;
}

export interface GithubReview {
  id: number;
  user: { login: string; avatar_url: string };
  state: 'APPROVED' | 'CHANGES_REQUESTED' | 'COMMENTED' | 'DISMISSED' | 'PENDING';
  body: string;
  submitted_at: string;
  html_url: string;
}

export interface GithubComment {
  id: number;
  user: { login: string; avatar_url: string };
  body: string;
  created_at: string;
  updated_at: string;
  html_url: string;
}

export interface GithubReviewComment extends GithubComment {
  path: string;
  line: number | null;
  original_line: number | null;
  diff_hunk: string;
  in_reply_to_id?: number;
}

export interface GithubFile {
  filename: string;
  status: 'added' | 'removed' | 'modified' | 'renamed';
  additions: number;
  deletions: number;
  changes: number;
  patch?: string; // unified diff — absent for binary files
}

export interface GithubCommit {
  sha: string;
  commit: {
    message: string;
    author: { name: string; date: string };
  };
  author: { login: string; avatar_url: string } | null;
}

export interface GithubCheckRun {
  name: string;
  status: 'queued' | 'in_progress' | 'completed';
  conclusion: 'success' | 'failure' | 'neutral' | 'cancelled' | 'skipped' | 'timed_out' | null;
}

export type ApiError =
  | { kind: 'unauthorized' }
  | { kind: 'rate_limited'; resetAt: Date }
  | { kind: 'not_found' }
  | { kind: 'unknown'; message: string };

export type ApiResult<T> = { ok: true; data: T } | { ok: false; error: ApiError };

function parseError(status: number, headers: Headers): ApiError {
  if (status === 401) return { kind: 'unauthorized' };
  if (status === 404) return { kind: 'not_found' };
  if (status === 403 || status === 429) {
    const reset = headers.get('X-RateLimit-Reset');
    return { kind: 'rate_limited', resetAt: new Date(Number(reset) * 1000) };
  }
  return { kind: 'unknown', message: `HTTP ${status}` };
}

async function ghFetch<T>(
  token: string,
  path: string
): Promise<ApiResult<T>> {
  const res = await fetch(`https://api.github.com${path}`, {
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28'
    }
  });
  if (!res.ok) return { ok: false, error: parseError(res.status, res.headers) };
  const data = await res.json();
  return { ok: true, data };
}

export const github = {
  listPRs(token: string, repo: string) {
    return ghFetch<GithubPR[]>(token, `/repos/${repo}/pulls?state=open&per_page=100`);
  },
  getPR(token: string, repo: string, number: number) {
    return ghFetch<GithubPR>(token, `/repos/${repo}/pulls/${number}`);
  },
  getRepo(token: string, repo: string) {
    return ghFetch<{ full_name: string }>(token, `/repos/${repo}`);
  },
  listReviews(token: string, repo: string, number: number) {
    return ghFetch<GithubReview[]>(token, `/repos/${repo}/pulls/${number}/reviews`);
  },
  listIssueComments(token: string, repo: string, number: number) {
    return ghFetch<GithubComment[]>(token, `/repos/${repo}/issues/${number}/comments`);
  },
  listReviewComments(token: string, repo: string, number: number) {
    return ghFetch<GithubReviewComment[]>(token, `/repos/${repo}/pulls/${number}/comments`);
  },
  listFiles(token: string, repo: string, number: number) {
    return ghFetch<GithubFile[]>(token, `/repos/${repo}/pulls/${number}/files`);
  },
  listCommits(token: string, repo: string, number: number) {
    return ghFetch<GithubCommit[]>(token, `/repos/${repo}/pulls/${number}/commits`);
  },
  listCheckRuns(token: string, repo: string, ref: string) {
    return ghFetch<{ check_runs: GithubCheckRun[] }>(
      token,
      `/repos/${repo}/commits/${ref}/check-runs`
    );
  }
};
```

- [ ] **Step 3: Commit**

```bash
git add apps/web/src/lib/stores/settings.ts apps/web/src/lib/api/github.ts
git commit -m "feat(web): settings store and typed GitHub API client"
```

---

## Task 4: Stack parsing logic

**Files:**
- Create: `apps/web/src/lib/api/stack.ts`

- [ ] **Step 1: Create stack.ts**

```ts
// apps/web/src/lib/api/stack.ts
import type { GithubPR } from './github';

export interface RemoteStackMeta {
  stack_id: string;
  frame_id: string;
  position: number;
  total: number;
}

export interface StackFrame {
  pr: GithubPR;
  meta: RemoteStackMeta;
}

export interface Stack {
  stackId: string;
  frames: StackFrame[]; // ordered bottom → top (position ascending)
}

/** Parse the ```giff ... ``` block from a PR body. Returns null if not present. */
export function parseStackMeta(body: string | null): RemoteStackMeta | null {
  if (!body) return null;
  const match = body.match(/```giff\n([\s\S]*?)\n```/);
  if (!match) return null;
  try {
    return JSON.parse(match[1]) as RemoteStackMeta;
  } catch {
    return null;
  }
}

/** Group a flat list of PRs into stacks. PRs without a giff block go into ungrouped. */
export function groupIntoStacks(prs: GithubPR[]): {
  stacks: Stack[];
  ungrouped: GithubPR[];
} {
  const stackMap = new Map<string, StackFrame[]>();
  const ungrouped: GithubPR[] = [];

  for (const pr of prs) {
    const meta = parseStackMeta(pr.body);
    if (!meta) {
      ungrouped.push(pr);
      continue;
    }
    const frames = stackMap.get(meta.stack_id) ?? [];
    frames.push({ pr, meta });
    stackMap.set(meta.stack_id, frames);
  }

  const stacks: Stack[] = [];
  for (const [stackId, frames] of stackMap.entries()) {
    stacks.push({
      stackId,
      frames: frames.sort((a, b) => a.meta.position - b.meta.position)
    });
  }

  // Sort stacks by the lowest PR number in each (stable ordering)
  stacks.sort((a, b) => a.frames[0].pr.number - b.frames[0].pr.number);

  return { stacks, ungrouped };
}

/** Given all PRs and a target PR number, find which stack it belongs to and its position. */
export function findFrameInStacks(
  stacks: Stack[],
  prNumber: number
): { stack: Stack; frame: StackFrame } | null {
  for (const stack of stacks) {
    const frame = stack.frames.find((f) => f.pr.number === prNumber);
    if (frame) return { stack, frame };
  }
  return null;
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/web/src/lib/api/stack.ts
git commit -m "feat(web): stack parsing and grouping logic"
```

---

## Task 5: Primitive display components

**Files:**
- Create: `apps/web/src/lib/components/PRStatusBadge.svelte`
- Create: `apps/web/src/lib/components/CIBadge.svelte`
- Create: `apps/web/src/lib/components/ReviewDecision.svelte`

- [ ] **Step 1: Create PRStatusBadge.svelte**

```svelte
<!-- apps/web/src/lib/components/PRStatusBadge.svelte -->
<script lang="ts">
  export let state: 'open' | 'closed' | 'merged';
  export let draft = false;

  $: label = draft ? 'draft' : state;
  $: classes = {
    open: 'bg-green-100 text-green-800 border-green-200',
    draft: 'bg-gray-100 text-gray-600 border-gray-200',
    merged: 'bg-purple-100 text-purple-800 border-purple-200',
    closed: 'bg-red-100 text-red-700 border-red-200'
  }[label];
</script>

<span class="inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium {classes}">
  {label}
</span>
```

- [ ] **Step 2: Create CIBadge.svelte**

```svelte
<!-- apps/web/src/lib/components/CIBadge.svelte -->
<script lang="ts">
  import type { GithubCheckRun } from '$lib/api/github';

  export let checks: GithubCheckRun[] = [];

  $: overall = (() => {
    if (checks.length === 0) return 'none';
    if (checks.some((c) => c.status !== 'completed')) return 'pending';
    if (checks.some((c) => c.conclusion === 'failure' || c.conclusion === 'timed_out'))
      return 'failure';
    return 'success';
  })();
</script>

{#if overall === 'none'}
  <span class="text-xs text-gray-400">no CI</span>
{:else if overall === 'pending'}
  <span class="inline-flex items-center gap-1 text-xs text-yellow-600">
    <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-yellow-400"></span>
    pending
  </span>
{:else if overall === 'success'}
  <span class="inline-flex items-center gap-1 text-xs text-green-600">
    <span>✓</span> CI
  </span>
{:else}
  <span class="inline-flex items-center gap-1 text-xs text-red-600">
    <span>✗</span> CI
  </span>
{/if}
```

- [ ] **Step 3: Create ReviewDecision.svelte**

```svelte
<!-- apps/web/src/lib/components/ReviewDecision.svelte -->
<script lang="ts">
  import type { GithubReview } from '$lib/api/github';

  export let reviews: GithubReview[] = [];

  // Keep only the latest review per reviewer
  $: latestByUser = (() => {
    const map = new Map<string, GithubReview>();
    for (const r of reviews) {
      if (r.state !== 'PENDING') map.set(r.user.login, r);
    }
    return [...map.values()];
  })();

  $: approved = latestByUser.filter((r) => r.state === 'APPROVED');
  $: changesRequested = latestByUser.filter((r) => r.state === 'CHANGES_REQUESTED');
</script>

<div class="flex flex-wrap gap-1">
  {#each approved as r}
    <span class="inline-flex items-center gap-1 rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-800">
      <img src={r.user.avatar_url} alt={r.user.login} class="h-3.5 w-3.5 rounded-full" />
      {r.user.login} ✓
    </span>
  {/each}
  {#each changesRequested as r}
    <span class="inline-flex items-center gap-1 rounded-full bg-red-100 px-2 py-0.5 text-xs text-red-800">
      <img src={r.user.avatar_url} alt={r.user.login} class="h-3.5 w-3.5 rounded-full" />
      {r.user.login} ✗
    </span>
  {/each}
  {#if latestByUser.length === 0}
    <span class="text-xs text-gray-400">no reviews</span>
  {/if}
</div>
```

- [ ] **Step 4: Commit**

```bash
git add apps/web/src/lib/components/PRStatusBadge.svelte apps/web/src/lib/components/CIBadge.svelte apps/web/src/lib/components/ReviewDecision.svelte
git commit -m "feat(web): PRStatusBadge, CIBadge, ReviewDecision components"
```

---

## Task 6: StackChain and FrameRow components

**Files:**
- Create: `apps/web/src/lib/components/FrameRow.svelte`
- Create: `apps/web/src/lib/components/StackChain.svelte`

- [ ] **Step 1: Create FrameRow.svelte**

```svelte
<!-- apps/web/src/lib/components/FrameRow.svelte -->
<script lang="ts">
  import type { GithubCheckRun, GithubReview } from '$lib/api/github';
  import type { StackFrame } from '$lib/api/stack';
  import PRStatusBadge from './PRStatusBadge.svelte';
  import CIBadge from './CIBadge.svelte';

  export let frame: StackFrame;
  export let checks: GithubCheckRun[] = [];
  export let reviews: GithubReview[] = [];
  export let active = false; // true when this is the PR currently being viewed

  $: pr = frame.pr;
  $: state = pr.merged_at ? 'merged' : (pr.state as 'open' | 'closed');
</script>

<a
  href="/pr/{pr.number}"
  class="group flex items-center gap-3 rounded-lg px-3 py-2 transition-colors hover:bg-gray-50
    {active ? 'bg-blue-50 ring-1 ring-blue-200' : ''}"
>
  <!-- Branch + PR number -->
  <div class="min-w-0 flex-1">
    <span class="block truncate font-mono text-sm font-medium text-gray-900 group-hover:text-blue-700">
      {pr.head.ref}
    </span>
    <span class="text-xs text-gray-500">PR #{pr.number} · {pr.title}</span>
  </div>

  <!-- Badges -->
  <div class="flex shrink-0 items-center gap-2">
    <CIBadge {checks} />
    <PRStatusBadge {state} draft={pr.draft} />
    {#if reviews.some((r) => r.state === 'APPROVED')}
      <span class="text-xs text-green-600" title="Approved">✓</span>
    {/if}
  </div>
</a>
```

- [ ] **Step 2: Create StackChain.svelte**

```svelte
<!-- apps/web/src/lib/components/StackChain.svelte -->
<script lang="ts">
  import type { GithubCheckRun, GithubReview } from '$lib/api/github';
  import type { Stack } from '$lib/api/stack';
  import FrameRow from './FrameRow.svelte';

  export let stack: Stack;
  export let checksByPR: Map<number, GithubCheckRun[]> = new Map();
  export let reviewsByPR: Map<number, GithubReview[]> = new Map();
  export let activePRNumber: number | null = null;

  $: trunk = stack.frames[0]?.pr.base.ref ?? 'main';
</script>

<div class="rounded-xl border border-gray-200 bg-white shadow-sm">
  <!-- Stack header -->
  <div class="border-b border-gray-100 px-4 py-3">
    <span class="text-xs font-semibold uppercase tracking-wide text-gray-500">stack</span>
    <span class="ml-1 text-sm font-medium text-gray-800">
      {stack.frames[0]?.pr.head.ref.split('/').slice(0, 2).join('/') ?? stack.stackId}
    </span>
    <span class="ml-2 text-xs text-gray-400">{stack.frames.length} frames</span>
  </div>

  <div class="px-3 py-2">
    <!-- Trunk row -->
    <div class="flex items-center gap-2 px-3 py-1.5">
      <span class="font-mono text-sm text-gray-500">● {trunk}</span>
    </div>

    <!-- Frames bottom → top -->
    {#each stack.frames as frame (frame.pr.number)}
      <!-- Connector line -->
      <div class="ml-4 h-4 w-px bg-gray-200"></div>
      <div class="flex items-start gap-1">
        <span class="mt-3 shrink-0 text-gray-400">◉</span>
        <div class="flex-1">
          <FrameRow
            {frame}
            checks={checksByPR.get(frame.pr.number) ?? []}
            reviews={reviewsByPR.get(frame.pr.number) ?? []}
            active={activePRNumber === frame.pr.number}
          />
        </div>
      </div>
    {/each}
  </div>
</div>
```

- [ ] **Step 3: Commit**

```bash
git add apps/web/src/lib/components/FrameRow.svelte apps/web/src/lib/components/StackChain.svelte
git commit -m "feat(web): FrameRow and StackChain components"
```

---

## Task 7: TokenForm component and Settings page

**Files:**
- Create: `apps/web/src/lib/components/TokenForm.svelte`
- Create: `apps/web/src/routes/settings/+page.svelte`

- [ ] **Step 1: Create TokenForm.svelte**

```svelte
<!-- apps/web/src/lib/components/TokenForm.svelte -->
<script lang="ts">
  import { settings } from '$lib/stores/settings';
  import { github } from '$lib/api/github';

  let token = $settings.token;
  let repo = $settings.repo;
  let repoStatus: 'idle' | 'checking' | 'ok' | 'error' = 'idle';
  let repoError = '';
  let saved = false;

  async function handleSubmit() {
    repoStatus = 'checking';
    repoError = '';

    const result = await github.getRepo(token, repo);
    if (!result.ok) {
      repoStatus = 'error';
      repoError =
        result.error.kind === 'unauthorized'
          ? 'Token invalid or lacks repo access.'
          : result.error.kind === 'not_found'
            ? 'Repository not found.'
            : 'Could not reach GitHub. Check your connection.';
      return;
    }

    settings.save({ token, repo: result.data.full_name });
    repoStatus = 'ok';
    saved = true;
    setTimeout(() => (saved = false), 3000);
  }

  function handleClear() {
    settings.clear();
    token = '';
    repo = '';
    repoStatus = 'idle';
  }

  $: maskedToken = token.length > 8 ? token.slice(0, 4) + '••••' + token.slice(-4) : '';
</script>

<form on:submit|preventDefault={handleSubmit} class="space-y-6">
  <div>
    <label for="token" class="block text-sm font-medium text-gray-700">
      GitHub personal access token
    </label>
    <p class="mt-0.5 text-xs text-gray-500">
      Needs <code>repo</code> scope.
      <a
        href="https://github.com/settings/tokens/new?scopes=repo&description=giff-dashboard"
        target="_blank"
        rel="noopener"
        class="text-blue-600 underline"
      >Create one ↗</a>
    </p>
    <input
      id="token"
      type="password"
      bind:value={token}
      placeholder={maskedToken || 'ghp_...'}
      class="mt-1.5 w-full rounded-lg border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <div>
    <label for="repo" class="block text-sm font-medium text-gray-700">Repository</label>
    <input
      id="repo"
      type="text"
      bind:value={repo}
      placeholder="owner/repo"
      class="mt-1.5 w-full rounded-lg border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500
        {repoStatus === 'error' ? 'border-red-400' : ''}
        {repoStatus === 'ok' ? 'border-green-400' : ''}"
    />
    {#if repoStatus === 'error'}
      <p class="mt-1 text-xs text-red-600">{repoError}</p>
    {/if}
    {#if repoStatus === 'ok'}
      <p class="mt-1 text-xs text-green-600">Connected ✓</p>
    {/if}
  </div>

  <div class="flex items-center gap-3">
    <button
      type="submit"
      disabled={!token || !repo || repoStatus === 'checking'}
      class="rounded-lg bg-gray-900 px-4 py-2 text-sm font-medium text-white hover:bg-gray-700 disabled:opacity-50"
    >
      {repoStatus === 'checking' ? 'Checking…' : saved ? 'Saved ✓' : 'Save'}
    </button>
    <button
      type="button"
      on:click={handleClear}
      class="rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
    >
      Clear saved data
    </button>
  </div>
</form>
```

- [ ] **Step 2: Create settings page**

```svelte
<!-- apps/web/src/routes/settings/+page.svelte -->
<script lang="ts">
  import TokenForm from '$lib/components/TokenForm.svelte';
</script>

<svelte:head><title>Settings — giff</title></svelte:head>

<div class="mx-auto max-w-lg px-4 py-12">
  <h1 class="mb-1 text-2xl font-semibold text-gray-900">Settings</h1>
  <p class="mb-8 text-sm text-gray-500">
    Your token and repo are stored only in this browser.
  </p>
  <TokenForm />
</div>
```

- [ ] **Step 3: Commit**

```bash
git add apps/web/src/lib/components/TokenForm.svelte apps/web/src/routes/settings/+page.svelte
git commit -m "feat(web): TokenForm component and settings page"
```

---

## Task 8: Root layout with nav and auth guard

**Files:**
- Create: `apps/web/src/routes/+layout.svelte`

- [ ] **Step 1: Create the layout**

```svelte
<!-- apps/web/src/routes/+layout.svelte -->
<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { settings } from '$lib/stores/settings';
  import type { ApiError } from '$lib/api/github';

  export let apiError: ApiError | null = null;

  $: isSettings = $page.url.pathname === '/settings';

  // Redirect to settings if no token/repo configured
  $: if (browser && !isSettings && (!$settings.token || !$settings.repo)) {
    goto('/settings');
  }

  $: errorBanner = (() => {
    if (!apiError) return null;
    if (apiError.kind === 'unauthorized') return 'Token invalid or expired — update it in Settings.';
    if (apiError.kind === 'rate_limited') {
      const t = apiError.resetAt.toLocaleTimeString();
      return `GitHub rate limit reached. Resets at ${t}.`;
    }
    return null;
  })();
</script>

<div class="min-h-screen bg-gray-50">
  <!-- Nav -->
  <nav class="border-b border-gray-200 bg-white">
    <div class="mx-auto flex max-w-6xl items-center justify-between px-4 py-3">
      <a href="/" class="text-base font-semibold tracking-tight text-gray-900">giff</a>
      <div class="flex items-center gap-4">
        {#if $settings.repo}
          <span class="font-mono text-xs text-gray-500">{$settings.repo}</span>
        {/if}
        <a
          href="/settings"
          class="rounded-md px-3 py-1.5 text-sm text-gray-600 hover:bg-gray-100
            {isSettings ? 'bg-gray-100 font-medium' : ''}"
        >
          Settings
        </a>
      </div>
    </div>
  </nav>

  <!-- API error banner -->
  {#if errorBanner}
    <div class="border-b border-yellow-200 bg-yellow-50 px-4 py-2 text-center text-sm text-yellow-800">
      {errorBanner}
      <a href="/settings" class="ml-2 underline">Go to Settings</a>
    </div>
  {/if}

  <main class="mx-auto max-w-6xl px-4 py-6">
    <slot />
  </main>
</div>
```

- [ ] **Step 2: Commit**

```bash
git add apps/web/src/routes/+layout.svelte
git commit -m "feat(web): root layout with nav and settings redirect guard"
```

---

## Task 9: Dashboard page (`/`)

**Files:**
- Create: `apps/web/src/routes/+page.svelte`

- [ ] **Step 1: Create the dashboard page**

```svelte
<!-- apps/web/src/routes/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { settings } from '$lib/stores/settings';
  import { github } from '$lib/api/github';
  import { groupIntoStacks } from '$lib/api/stack';
  import type { GithubCheckRun, GithubReview, GithubPR, ApiError } from '$lib/api/github';
  import type { Stack } from '$lib/api/stack';
  import StackChain from '$lib/components/StackChain.svelte';
  import PRStatusBadge from '$lib/components/PRStatusBadge.svelte';

  let stacks: Stack[] = [];
  let ungrouped: GithubPR[] = [];
  let checksByPR = new Map<number, GithubCheckRun[]>();
  let reviewsByPR = new Map<number, GithubReview[]>();
  let loading = true;
  let error: ApiError | null = null;

  onMount(async () => {
    const { token, repo } = $settings;
    if (!token || !repo) return;

    const prsResult = await github.listPRs(token, repo);
    if (!prsResult.ok) { error = prsResult.error; loading = false; return; }

    const grouped = groupIntoStacks(prsResult.data);
    stacks = grouped.stacks;
    ungrouped = grouped.ungrouped;
    loading = false;

    // Fetch reviews and CI for all PRs in the background
    const allPRs = prsResult.data;
    await Promise.all(
      allPRs.map(async (pr) => {
        const [reviewsResult, checksResult] = await Promise.all([
          github.listReviews(token, repo, pr.number),
          github.listCheckRuns(token, repo, pr.head.sha)
        ]);
        if (reviewsResult.ok) reviewsByPR.set(pr.number, reviewsResult.data);
        if (checksResult.ok) checksByPR.set(pr.number, checksResult.data.check_runs);
        // Trigger reactivity
        checksByPR = new Map(checksByPR);
        reviewsByPR = new Map(reviewsByPR);
      })
    );
  });
</script>

<svelte:head><title>Dashboard — giff</title></svelte:head>

{#if loading}
  <div class="flex items-center justify-center py-20 text-sm text-gray-400">Loading stacks…</div>
{:else if error}
  <div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
    {error.kind === 'unauthorized'
      ? 'Token invalid — update it in Settings.'
      : error.kind === 'not_found'
        ? 'Repository not found.'
        : 'Failed to load PRs.'}
  </div>
{:else if stacks.length === 0 && ungrouped.length === 0}
  <div class="py-20 text-center text-sm text-gray-400">
    No open PRs found in <span class="font-mono">{$settings.repo}</span>.
  </div>
{:else}
  <div class="space-y-4">
    {#each stacks as stack (stack.stackId)}
      <StackChain {stack} {checksByPR} {reviewsByPR} />
    {/each}

    {#if ungrouped.length > 0}
      <div class="rounded-xl border border-gray-200 bg-white shadow-sm">
        <div class="border-b border-gray-100 px-4 py-3">
          <span class="text-xs font-semibold uppercase tracking-wide text-gray-500">
            Ungrouped PRs
          </span>
        </div>
        <ul class="divide-y divide-gray-100 px-3 py-2">
          {#each ungrouped as pr (pr.number)}
            <li class="py-2">
              <a href="/pr/{pr.number}" class="flex items-center gap-3 hover:text-blue-700">
                <span class="font-mono text-sm text-gray-700">#{pr.number}</span>
                <span class="flex-1 text-sm">{pr.title}</span>
                <PRStatusBadge
                  state={pr.merged_at ? 'merged' : (pr.state as 'open' | 'closed')}
                  draft={pr.draft}
                />
              </a>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>
{/if}
```

- [ ] **Step 2: Commit**

```bash
git add apps/web/src/routes/+page.svelte
git commit -m "feat(web): dashboard page with stack grouping"
```

---

## Task 10: Conversation and InlineComment components

**Files:**
- Create: `apps/web/src/lib/components/InlineComment.svelte`
- Create: `apps/web/src/lib/components/ConversationThread.svelte`

- [ ] **Step 1: Create InlineComment.svelte**

```svelte
<!-- apps/web/src/lib/components/InlineComment.svelte -->
<script lang="ts">
  import type { GithubReviewComment } from '$lib/api/github';

  export let comment: GithubReviewComment;
</script>

<div class="my-1 rounded-md border border-blue-100 bg-blue-50 px-3 py-2 text-xs">
  <div class="mb-1 flex items-center gap-2">
    <img src={comment.user.avatar_url} alt={comment.user.login} class="h-4 w-4 rounded-full" />
    <span class="font-medium text-gray-700">{comment.user.login}</span>
    <span class="text-gray-400">{new Date(comment.created_at).toLocaleDateString()}</span>
  </div>
  <p class="whitespace-pre-wrap text-gray-700">{comment.body}</p>
</div>
```

- [ ] **Step 2: Create ConversationThread.svelte**

```svelte
<!-- apps/web/src/lib/components/ConversationThread.svelte -->
<script lang="ts">
  import type { GithubComment, GithubReview } from '$lib/api/github';

  export let comments: GithubComment[] = [];
  export let reviews: GithubReview[] = [];

  type TimelineItem =
    | { kind: 'comment'; item: GithubComment }
    | { kind: 'review'; item: GithubReview };

  $: timeline = [
    ...comments.map((c): TimelineItem => ({ kind: 'comment', item: c })),
    ...reviews
      .filter((r) => r.body.trim().length > 0)
      .map((r): TimelineItem => ({ kind: 'review', item: r }))
  ].sort((a, b) => {
    const dateA = a.kind === 'comment' ? a.item.created_at : (a.item as GithubReview).submitted_at;
    const dateB = b.kind === 'comment' ? b.item.created_at : (b.item as GithubReview).submitted_at;
    return new Date(dateA).getTime() - new Date(dateB).getTime();
  });

  $: stateLabel = (state: GithubReview['state']) =>
    ({
      APPROVED: { text: 'approved', classes: 'bg-green-100 text-green-800' },
      CHANGES_REQUESTED: { text: 'requested changes', classes: 'bg-red-100 text-red-800' },
      COMMENTED: { text: 'reviewed', classes: 'bg-gray-100 text-gray-700' },
      DISMISSED: { text: 'dismissed', classes: 'bg-gray-100 text-gray-500' },
      PENDING: { text: 'pending', classes: 'bg-gray-100 text-gray-400' }
    })[state] ?? { text: state, classes: 'bg-gray-100 text-gray-500' };
</script>

{#if timeline.length === 0}
  <p class="py-8 text-center text-sm text-gray-400">No conversation yet.</p>
{:else}
  <div class="space-y-4">
    {#each timeline as entry (entry.kind === 'comment' ? `c${entry.item.id}` : `r${entry.item.id}`)}
      {@const item = entry.item}
      <div class="flex gap-3">
        <img
          src={item.user.avatar_url}
          alt={item.user.login}
          class="mt-0.5 h-7 w-7 shrink-0 rounded-full"
        />
        <div class="flex-1">
          <div class="mb-1 flex items-center gap-2">
            <span class="text-sm font-medium text-gray-900">{item.user.login}</span>
            {#if entry.kind === 'review'}
              {@const label = stateLabel((entry.item as GithubReview).state)}
              <span class="rounded-full px-2 py-0.5 text-xs {label.classes}">{label.text}</span>
            {/if}
            <span class="text-xs text-gray-400">
              {new Date(
                entry.kind === 'comment'
                  ? (item as GithubComment).created_at
                  : (item as GithubReview).submitted_at
              ).toLocaleString()}
            </span>
          </div>
          <div class="rounded-lg border border-gray-200 bg-white px-4 py-3 text-sm text-gray-700">
            <p class="whitespace-pre-wrap">{item.body}</p>
          </div>
        </div>
      </div>
    {/each}
  </div>
{/if}
```

- [ ] **Step 3: Commit**

```bash
git add apps/web/src/lib/components/InlineComment.svelte apps/web/src/lib/components/ConversationThread.svelte
git commit -m "feat(web): InlineComment and ConversationThread components"
```

---

## Task 11: DiffView component with shiki highlighting

**Files:**
- Create: `apps/web/src/lib/components/DiffView.svelte`

- [ ] **Step 1: Create DiffView.svelte**

```svelte
<!-- apps/web/src/lib/components/DiffView.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { GithubFile, GithubReviewComment } from '$lib/api/github';
  import InlineComment from './InlineComment.svelte';

  export let files: GithubFile[] = [];
  export let reviewComments: GithubReviewComment[] = [];

  // Map: filename → collapsed state
  let collapsed = new Map<string, boolean>();

  $: {
    for (const f of files) {
      if (!collapsed.has(f.filename)) {
        collapsed.set(f.filename, f.changes > 200);
      }
    }
    collapsed = new Map(collapsed);
  }

  function toggle(filename: string) {
    collapsed.set(filename, !collapsed.get(filename));
    collapsed = new Map(collapsed);
  }

  // Group review comments by filename + line
  $: commentsByFileLine = (() => {
    const map = new Map<string, GithubReviewComment[]>();
    for (const c of reviewComments) {
      const key = `${c.path}:${c.line ?? c.original_line ?? 0}`;
      const arr = map.get(key) ?? [];
      arr.push(c);
      map.set(key, arr);
    }
    return map;
  })();

  function parseHunk(patch: string): Array<{
    type: 'context' | 'add' | 'remove' | 'hunk';
    content: string;
    lineOld: number | null;
    lineNew: number | null;
  }> {
    const lines = patch.split('\n');
    let oldLine = 0;
    let newLine = 0;
    return lines.map((line) => {
      if (line.startsWith('@@')) {
        const m = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)/);
        if (m) { oldLine = parseInt(m[1]); newLine = parseInt(m[2]); }
        return { type: 'hunk' as const, content: line, lineOld: null, lineNew: null };
      }
      if (line.startsWith('+')) {
        const l = { type: 'add' as const, content: line.slice(1), lineOld: null, lineNew: newLine };
        newLine++;
        return l;
      }
      if (line.startsWith('-')) {
        const l = { type: 'remove' as const, content: line.slice(1), lineOld: oldLine, lineNew: null };
        oldLine++;
        return l;
      }
      const l = { type: 'context' as const, content: line.startsWith(' ') ? line.slice(1) : line, lineOld: oldLine, lineNew: newLine };
      oldLine++; newLine++;
      return l;
    });
  }

  // File list anchor refs
  function anchor(filename: string) {
    return 'file-' + filename.replace(/[^a-zA-Z0-9]/g, '-');
  }
</script>

<!-- File list summary -->
<div class="mb-4 rounded-lg border border-gray-200 bg-white">
  <div class="border-b border-gray-100 px-4 py-2 text-xs font-medium uppercase tracking-wide text-gray-500">
    {files.length} files changed
  </div>
  <ul class="divide-y divide-gray-50">
    {#each files as file (file.filename)}
      <li>
        <a
          href="#{anchor(file.filename)}"
          class="flex items-center gap-3 px-4 py-2 text-xs hover:bg-gray-50"
        >
          <span class="font-mono text-gray-700 truncate flex-1">{file.filename}</span>
          <span class="text-green-600">+{file.additions}</span>
          <span class="text-red-500">-{file.deletions}</span>
        </a>
      </li>
    {/each}
  </ul>
</div>

<!-- Diffs -->
{#each files as file (file.filename)}
  <div id={anchor(file.filename)} class="mb-4 overflow-hidden rounded-lg border border-gray-200 bg-white">
    <!-- File header -->
    <button
      on:click={() => toggle(file.filename)}
      class="flex w-full items-center gap-3 border-b border-gray-100 bg-gray-50 px-4 py-2 text-left hover:bg-gray-100"
    >
      <span class="text-xs text-gray-400">{collapsed.get(file.filename) ? '▶' : '▼'}</span>
      <span class="flex-1 font-mono text-xs text-gray-800">{file.filename}</span>
      <span class="text-xs">
        <span class="text-green-600">+{file.additions}</span>
        <span class="text-gray-400 mx-1">/</span>
        <span class="text-red-500">-{file.deletions}</span>
      </span>
      {#if file.status !== 'modified'}
        <span class="rounded bg-gray-200 px-1.5 py-0.5 text-xs text-gray-600">{file.status}</span>
      {/if}
    </button>

    {#if !collapsed.get(file.filename)}
      {#if file.patch}
        {@const hunks = parseHunk(file.patch)}
        <div class="overflow-x-auto">
          <table class="w-full text-xs">
            <tbody>
              {#each hunks as row, i (i)}
                {#if row.type === 'hunk'}
                  <tr class="bg-blue-50">
                    <td colspan="3" class="px-3 py-1 font-mono text-blue-600">{row.content}</td>
                  </tr>
                {:else}
                  <tr class="
                    {row.type === 'add' ? 'bg-green-50' : ''}
                    {row.type === 'remove' ? 'bg-red-50' : ''}
                    hover:brightness-95
                  ">
                    <td class="w-10 select-none border-r border-gray-100 px-2 py-0.5 text-right font-mono text-gray-300">
                      {row.lineOld ?? ''}
                    </td>
                    <td class="w-10 select-none border-r border-gray-100 px-2 py-0.5 text-right font-mono text-gray-300">
                      {row.lineNew ?? ''}
                    </td>
                    <td class="px-3 py-0.5 font-mono whitespace-pre">
                      <span class="{row.type === 'add' ? 'text-green-700' : ''}{row.type === 'remove' ? 'text-red-700' : ''}">
                        {row.type === 'add' ? '+' : row.type === 'remove' ? '-' : ' '}{row.content}
                      </span>
                    </td>
                  </tr>
                  <!-- Inline review comments for this line -->
                  {#each commentsByFileLine.get(`${file.filename}:${row.lineNew ?? row.lineOld ?? 0}`) ?? [] as comment (comment.id)}
                    <tr>
                      <td colspan="3" class="px-3 py-1">
                        <InlineComment {comment} />
                      </td>
                    </tr>
                  {/each}
                {/if}
              {/each}
            </tbody>
          </table>
        </div>
      {:else}
        <p class="px-4 py-3 text-xs text-gray-400 italic">Binary file — no diff available.</p>
      {/if}
    {/if}
  </div>
{/each}
```

- [ ] **Step 2: Commit**

```bash
git add apps/web/src/lib/components/DiffView.svelte
git commit -m "feat(web): DiffView with unified diff rendering and inline review comments"
```

---

## Task 12: Full PR page (`/pr/[number]`)

**Files:**
- Create: `apps/web/src/routes/pr/[number]/+page.svelte`

- [ ] **Step 1: Create the PR page**

```svelte
<!-- apps/web/src/routes/pr/[number]/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { settings } from '$lib/stores/settings';
  import { github } from '$lib/api/github';
  import { groupIntoStacks, findFrameInStacks } from '$lib/api/stack';
  import type {
    GithubPR, GithubReview, GithubComment,
    GithubReviewComment, GithubFile, GithubCommit, ApiError
  } from '$lib/api/github';
  import type { Stack } from '$lib/api/stack';
  import PRStatusBadge from '$lib/components/PRStatusBadge.svelte';
  import ReviewDecision from '$lib/components/ReviewDecision.svelte';
  import ConversationThread from '$lib/components/ConversationThread.svelte';
  import DiffView from '$lib/components/DiffView.svelte';
  import StackChain from '$lib/components/StackChain.svelte';

  $: prNumber = parseInt($page.params.number);

  let pr: GithubPR | null = null;
  let reviews: GithubReview[] = [];
  let comments: GithubComment[] = [];
  let reviewComments: GithubReviewComment[] = [];
  let files: GithubFile[] = [];
  let commits: GithubCommit[] = [];
  let stack: Stack | null = null;
  let loading = true;
  let error: ApiError | null = null;
  let activeTab: 'conversation' | 'files' | 'commits' = 'conversation';

  onMount(async () => {
    const { token, repo } = $settings;
    if (!token || !repo) return;

    // Fetch everything in parallel
    const [prResult, reviewsResult, commentsResult, reviewCommentsResult, filesResult, commitsResult, allPRsResult] =
      await Promise.all([
        github.getPR(token, repo, prNumber),
        github.listReviews(token, repo, prNumber),
        github.listIssueComments(token, repo, prNumber),
        github.listReviewComments(token, repo, prNumber),
        github.listFiles(token, repo, prNumber),
        github.listCommits(token, repo, prNumber),
        github.listPRs(token, repo)
      ]);

    if (!prResult.ok) { error = prResult.error; loading = false; return; }

    pr = prResult.data;
    if (reviewsResult.ok) reviews = reviewsResult.data;
    if (commentsResult.ok) comments = commentsResult.data;
    if (reviewCommentsResult.ok) reviewComments = reviewCommentsResult.data;
    if (filesResult.ok) files = filesResult.data;
    if (commitsResult.ok) commits = commitsResult.data;

    // Find stack context
    if (allPRsResult.ok) {
      const { stacks } = groupIntoStacks(allPRsResult.data);
      const found = findFrameInStacks(stacks, prNumber);
      if (found) stack = found.stack;
    }

    loading = false;
  });

  $: prState = pr ? (pr.merged_at ? 'merged' : (pr.state as 'open' | 'closed')) : 'open';
</script>

<svelte:head>
  <title>{pr ? `PR #${prNumber}: ${pr.title}` : `PR #${prNumber}`} — giff</title>
</svelte:head>

{#if loading}
  <div class="flex items-center justify-center py-20 text-sm text-gray-400">Loading PR…</div>
{:else if error || !pr}
  <div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
    Failed to load PR #{prNumber}.
  </div>
{:else}
  <div class="grid grid-cols-1 gap-6 lg:grid-cols-[220px_1fr_240px]">

    <!-- Left: stack context -->
    <aside class="order-3 lg:order-1">
      {#if stack}
        <div class="sticky top-6">
          <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-gray-400">Stack</p>
          <StackChain {stack} activePRNumber={prNumber} />
        </div>
      {:else}
        <div class="rounded-lg border border-dashed border-gray-200 px-4 py-3 text-xs text-gray-400">
          Not part of a giff stack.
        </div>
      {/if}
    </aside>

    <!-- Main: PR content -->
    <div class="order-1 lg:order-2 min-w-0">
      <!-- PR header -->
      <div class="mb-4">
        <div class="flex flex-wrap items-start gap-3">
          <PRStatusBadge state={prState} draft={pr.draft} />
          <h1 class="text-xl font-semibold text-gray-900">{pr.title}</h1>
        </div>
        <p class="mt-1 text-sm text-gray-500">
          <span class="font-mono">{pr.base.ref} ← {pr.head.ref}</span>
          · opened by <span class="font-medium">{pr.user.login}</span>
          · {new Date(pr.created_at).toLocaleDateString()}
        </p>
      </div>

      <!-- Tabs -->
      <div class="mb-4 flex gap-1 border-b border-gray-200">
        {#each [['conversation', 'Conversation'], ['files', `Files (${files.length})`], ['commits', `Commits (${commits.length})`]] as [id, label]}
          <button
            on:click={() => (activeTab = id as typeof activeTab)}
            class="rounded-t-md px-4 py-2 text-sm transition-colors
              {activeTab === id
                ? 'border-b-2 border-gray-900 font-medium text-gray-900'
                : 'text-gray-500 hover:text-gray-700'}"
          >
            {label}
          </button>
        {/each}
      </div>

      {#if activeTab === 'conversation'}
        <ConversationThread {comments} {reviews} />

      {:else if activeTab === 'files'}
        <DiffView {files} {reviewComments} />

      {:else if activeTab === 'commits'}
        <div class="space-y-2">
          {#each commits as commit (commit.sha)}
            <div class="flex items-start gap-3 rounded-lg border border-gray-200 bg-white px-4 py-3">
              {#if commit.author}
                <img
                  src={commit.author.avatar_url}
                  alt={commit.author.login}
                  class="mt-0.5 h-6 w-6 shrink-0 rounded-full"
                />
              {/if}
              <div class="min-w-0 flex-1">
                <p class="text-sm text-gray-900">{commit.commit.message.split('\n')[0]}</p>
                <p class="mt-0.5 font-mono text-xs text-gray-400">
                  {commit.sha.slice(0, 7)}
                  · {commit.commit.author.name}
                  · {new Date(commit.commit.author.date).toLocaleDateString()}
                </p>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Right: metadata -->
    <aside class="order-2 lg:order-3 space-y-5">
      <!-- Reviewers -->
      <div>
        <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-gray-400">Reviews</p>
        <ReviewDecision {reviews} />
      </div>

      <!-- Requested reviewers -->
      {#if pr.requested_reviewers.length > 0}
        <div>
          <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-gray-400">
            Awaiting review
          </p>
          <div class="flex flex-wrap gap-1.5">
            {#each pr.requested_reviewers as r (r.login)}
              <span class="inline-flex items-center gap-1 rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-700">
                <img src={r.avatar_url} alt={r.login} class="h-3.5 w-3.5 rounded-full" />
                {r.login}
              </span>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Labels -->
      {#if pr.labels.length > 0}
        <div>
          <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-gray-400">Labels</p>
          <div class="flex flex-wrap gap-1">
            {#each pr.labels as label (label.name)}
              <span
                class="rounded-full border px-2 py-0.5 text-xs"
                style="background: #{label.color}22; border-color: #{label.color}; color: #{label.color}"
              >
                {label.name}
              </span>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Milestone -->
      {#if pr.milestone}
        <div>
          <p class="mb-1 text-xs font-semibold uppercase tracking-wide text-gray-400">Milestone</p>
          <span class="text-sm text-gray-700">{pr.milestone.title}</span>
        </div>
      {/if}

      <!-- GitHub link -->
      <a
        href={pr.html_url}
        target="_blank"
        rel="noopener"
        class="inline-flex items-center gap-1.5 rounded-lg border border-gray-300 px-3 py-2 text-xs font-medium text-gray-700 hover:bg-gray-50"
      >
        View on GitHub ↗
      </a>
    </aside>
  </div>
{/if}
```

- [ ] **Step 2: Commit**

```bash
git add apps/web/src/routes/pr/\[number\]/+page.svelte
git commit -m "feat(web): full PR page with conversation, diff, and commits tabs"
```

---

## Task 13: Wire up and verify the app builds

**Files:**
- Modify: `apps/web/src/routes/+layout.svelte` (add `ssr = false`)
- Create: `apps/web/src/routes/+layout.ts`

- [ ] **Step 1: Disable SSR globally (required for localStorage + adapter-static)**

```ts
// apps/web/src/routes/+layout.ts
export const ssr = false;
export const prerender = false;
```

- [ ] **Step 2: Run type check**

```bash
cd apps/web && npm run check
```

Expected: no type errors. Fix any that appear.

- [ ] **Step 3: Run dev server and verify all pages load**

```bash
cd apps/web && npm run dev
```

Visit:
- `http://localhost:5173/` → should redirect to `/settings`
- `http://localhost:5173/settings` → should show the token form
- Enter a real token + repo → save → redirected to dashboard
- `http://localhost:5173/` → should show stacks or "No open PRs"
- Click a PR → `/pr/123` → tabs load correctly

- [ ] **Step 4: Build for production**

```bash
cd apps/web && npm run build
```

Expected: `build/` directory created, no errors.

- [ ] **Step 5: Final commit**

```bash
git add apps/web/
git commit -m "feat(web): disable SSR for static adapter, production build verified"
```

---

## Self-Review

**Spec coverage:**
- ✅ `/` dashboard with stack grouping and ungrouped PRs
- ✅ `/pr/[number]` with conversation, files changed (diff + inline comments), commits tabs
- ✅ Left sidebar with stack context, right sidebar with metadata
- ✅ `/settings` with token input, repo validation, clear button
- ✅ Settings redirect guard in layout
- ✅ Error states: 401, rate limit, 404, no giff block
- ✅ `PRStatusBadge`, `CIBadge`, `FrameRow`, `StackChain`, `ReviewDecision`, `ConversationThread`, `InlineComment`, `DiffView`, `TokenForm` — all components present
- ✅ `adapter-static` + SSR disabled
- ✅ `$settings` store backed by localStorage

**No placeholders present.** All code shown inline.

**Type consistency:** `GithubPR`, `GithubReview`, `GithubComment`, `GithubReviewComment`, `GithubFile`, `GithubCommit`, `GithubCheckRun`, `ApiError`, `Stack`, `StackFrame`, `RemoteStackMeta` defined in Tasks 3–4 and used consistently in Tasks 5–12.
