<script lang="ts">
  import {
    getPull,
    listIssueComments,
    listReviews,
    listReviewComments,
    listCommits,
    listFiles
  } from '$lib/api/github';
  import type {
    Commit,
    IssueComment,
    PullFile,
    PullRequest,
    Review,
    ReviewComment
  } from '$lib/api/types';
  import ReviewDecision from './ReviewDecision.svelte';
  import DiffView from './DiffView.svelte';
  import ConversationThread from './ConversationThread.svelte';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Badge } from '$lib/components/ui/badge';
  import { ExternalLink, Eye } from 'lucide-svelte';
  import { timeAgo } from '$lib/utils/time';
  import { cn } from '$lib/utils';

  export let prNumber: number;

  let pr: PullRequest | null = null;
  let comments: IssueComment[] = [];
  let reviews: Review[] = [];
  let reviewComments: ReviewComment[] = [];
  let commits: Commit[] = [];
  let files: PullFile[] = [];
  let loading = true;

  // Local-only "Viewed" toggle per file. Resets between PRs (the {#key prNumber} block in
  // routes/+page.svelte remounts this component on PR change).
  let viewed = new Set<string>();

  $: reviewersDecisions = (() => {
    const latest = new Map<string, Review>();
    for (const r of reviews) {
      const prev = latest.get(r.user.login);
      if (!prev || Date.parse(r.submitted_at ?? '0') > Date.parse(prev.submitted_at ?? '0')) {
        latest.set(r.user.login, r);
      }
    }
    return Array.from(latest.values());
  })();

  $: commentsByFile = (() => {
    const by: Record<string, ReviewComment[]> = {};
    for (const c of reviewComments) {
      (by[c.path] ??= []).push(c);
    }
    return by;
  })();

  $: conversationCount = comments.length + reviews.filter((r) => r.body).length;

  $: void prNumber, load();

  async function load() {
    if (!Number.isFinite(prNumber)) return;
    loading = true;
    pr = null;
    try {
      const [p, ic, rv, rc, cm, fl] = await Promise.all([
        getPull(prNumber),
        listIssueComments(prNumber),
        listReviews(prNumber),
        listReviewComments(prNumber),
        listCommits(prNumber),
        listFiles(prNumber)
      ]);
      pr = p;
      comments = ic;
      reviews = rv;
      reviewComments = rc;
      commits = cm;
      files = fl;
    } finally {
      loading = false;
    }
  }

  function toggleViewed(path: string) {
    if (viewed.has(path)) {
      viewed.delete(path);
    } else {
      viewed.add(path);
    }
    viewed = viewed;
  }

  // Big status pill at the top of the PR. Different palette from GitHub: filled neutral for
  // open, brand red for merged, outlined for draft and closed.
  $: statusPill = (() => {
    if (!pr) return { label: '', cls: '' };
    if (pr.merged) {
      return {
        label: 'Merged',
        cls: 'bg-brand text-brand-fg'
      };
    }
    if (pr.state === 'closed') {
      return {
        label: 'Closed',
        cls: 'border border-foreground/30 text-muted-foreground line-through decoration-foreground/30'
      };
    }
    if (pr.draft) {
      return {
        label: 'Draft',
        cls: 'border border-foreground/20 text-muted-foreground'
      };
    }
    return {
      label: 'Open',
      cls: 'bg-foreground text-background'
    };
  })();
</script>

{#if loading}
  <p class="text-sm text-muted-foreground">Loading PR #{prNumber}…</p>
{:else if !pr}
  <p class="text-sm text-muted-foreground">Could not load PR #{prNumber}.</p>
{:else}
  <div class="grid gap-10 lg:grid-cols-[minmax(0,1fr)_15rem]">
    <div class="space-y-6 min-w-0">
      <header class="space-y-4 pb-5 border-b">
        <h1 class="text-2xl font-semibold tracking-tight flex items-baseline gap-2 flex-wrap">
          <span>{pr.title}</span>
          <span class="text-muted-foreground font-normal">#{pr.number}</span>
          <a
            href={pr.html_url}
            target="_blank"
            rel="noopener"
            class="text-muted-foreground hover:text-foreground"
            aria-label="Open on GitHub"
          >
            <ExternalLink class="h-4 w-4" />
          </a>
        </h1>

        <div class="flex items-center gap-3 flex-wrap">
          <span
            class={cn(
              'inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium',
              statusPill.cls
            )}
          >
            {statusPill.label}
          </span>

          <span class="flex items-center gap-2 text-sm text-muted-foreground">
            <img
              src={pr.user.avatar_url}
              alt={pr.user.login}
              class="h-5 w-5 rounded-full"
            />
            <span>
              <a
                href={pr.user.html_url}
                target="_blank"
                rel="noopener"
                class="font-medium text-foreground hover:underline underline-offset-2"
              >
                {pr.user.login}
              </a>
              {pr.merged ? 'merged' : 'wants to merge'}
              <span class="font-medium text-foreground">{commits.length}</span>
              {commits.length === 1 ? 'commit' : 'commits'}
              into
              <span
                class="inline-block font-mono text-xs px-1.5 py-0.5 rounded border bg-muted/50 text-foreground"
              >
                {pr.base.ref}
              </span>
              from
              <span
                class="inline-block font-mono text-xs px-1.5 py-0.5 rounded border bg-muted/50 text-foreground"
              >
                {pr.head.ref}
              </span>
              · {timeAgo(pr.created_at)}
            </span>
          </span>
        </div>
      </header>

      <Tabs value="conversation" class="w-full">
        <TabsList>
          <TabsTrigger value="conversation">
            Conversation
            {#if conversationCount > 0}
              <span class="ml-1.5 inline-flex items-center justify-center rounded-full bg-muted px-1.5 text-[10px] font-medium text-muted-foreground min-w-5">
                {conversationCount}
              </span>
            {/if}
          </TabsTrigger>
          <TabsTrigger value="files">
            Files
            {#if files.length > 0}
              <span class="ml-1.5 inline-flex items-center justify-center rounded-full bg-muted px-1.5 text-[10px] font-medium text-muted-foreground min-w-5">
                {files.length}
              </span>
            {/if}
          </TabsTrigger>
          <TabsTrigger value="commits">
            Commits
            {#if commits.length > 0}
              <span class="ml-1.5 inline-flex items-center justify-center rounded-full bg-muted px-1.5 text-[10px] font-medium text-muted-foreground min-w-5">
                {commits.length}
              </span>
            {/if}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="conversation" class="space-y-6">
          {#if pr.body && pr.body.trim()}
            <!-- Description rendered as the first comment in the timeline, like GitHub. -->
            <article class="flex gap-3">
              <img
                src={pr.user.avatar_url}
                alt={pr.user.login}
                class="h-8 w-8 rounded-full mt-1"
              />
              <div class="flex-1 min-w-0 rounded-md border bg-card">
                <header class="flex items-center gap-2 border-b px-3 py-1.5 bg-muted/40 text-sm">
                  <a
                    href={pr.user.html_url}
                    target="_blank"
                    rel="noopener"
                    class="font-medium hover:underline underline-offset-2"
                  >
                    {pr.user.login}
                  </a>
                  <span class="text-muted-foreground">
                    opened this · {timeAgo(pr.created_at)}
                  </span>
                </header>
                <p class="px-3 py-3 text-sm whitespace-pre-wrap break-words">{pr.body}</p>
              </div>
            </article>
          {/if}
          <ConversationThread {comments} {reviews} />
        </TabsContent>

        <TabsContent value="files" class="space-y-4">
          {#if files.length === 0}
            <p class="text-sm text-muted-foreground">No file changes.</p>
          {:else}
            <!-- File jump-list with viewed toggle -->
            <div class="rounded-md border bg-card text-sm">
              {#each files as f, i (f.filename)}
                <div
                  class={cn(
                    'flex items-center gap-3 px-3 py-2',
                    i + 1 < files.length && 'border-b'
                  )}
                >
                  <a
                    href={`#file-${encodeURIComponent(f.filename)}`}
                    class="font-mono truncate flex-1 hover:underline underline-offset-2"
                  >
                    {f.filename}
                  </a>
                  <span class="text-xs tabular-nums">
                    <span class="text-foreground">+{f.additions}</span>
                    <span class="text-muted-foreground ml-1">−{f.deletions}</span>
                  </span>
                  <button
                    type="button"
                    on:click={() => toggleViewed(f.filename)}
                    class={cn(
                      'inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs transition-colors',
                      viewed.has(f.filename)
                        ? 'bg-foreground text-background border-foreground'
                        : 'text-muted-foreground hover:text-foreground'
                    )}
                    aria-pressed={viewed.has(f.filename)}
                  >
                    <Eye class="h-3 w-3" />
                    Viewed
                  </button>
                </div>
              {/each}
            </div>
            {#each files as f (f.filename)}
              <div
                id={`file-${encodeURIComponent(f.filename)}`}
                class={cn(viewed.has(f.filename) && 'opacity-50')}
              >
                <DiffView file={f} comments={commentsByFile[f.filename] ?? []} />
              </div>
            {/each}
          {/if}
        </TabsContent>

        <TabsContent value="commits">
          {#if commits.length === 0}
            <p class="text-sm text-muted-foreground">No commits.</p>
          {:else}
            <ul class="rounded-md border bg-card divide-y">
              {#each commits as c (c.sha)}
                <li class="flex items-center gap-3 px-3 py-2 text-sm">
                  {#if c.author?.avatar_url}
                    <img
                      src={c.author.avatar_url}
                      alt={c.author.login}
                      class="h-6 w-6 rounded-full"
                    />
                  {:else}
                    <div
                      class="h-6 w-6 rounded-full bg-muted flex items-center justify-center text-[10px] text-muted-foreground"
                    >
                      {(c.commit.author.name[0] ?? '?').toUpperCase()}
                    </div>
                  {/if}
                  <span class="flex-1 truncate">{c.commit.message.split('\n')[0]}</span>
                  <span class="text-xs text-muted-foreground">
                    {c.author?.login ?? c.commit.author.name} · {timeAgo(c.commit.author.date)}
                  </span>
                  <a
                    href={c.html_url}
                    target="_blank"
                    rel="noopener"
                    class="font-mono text-xs text-muted-foreground hover:text-foreground tabular-nums"
                  >
                    {c.sha.slice(0, 7)}
                  </a>
                </li>
              {/each}
            </ul>
          {/if}
        </TabsContent>
      </Tabs>
    </div>

    <!-- Right: PR metadata -->
    <aside class="space-y-6 text-sm">
      <section class="space-y-2">
        <h2 class="text-xs uppercase tracking-wide text-muted-foreground">Reviewers</h2>
        {#if reviewersDecisions.length === 0 && pr.requested_reviewers.length === 0}
          <p class="text-muted-foreground text-xs">None.</p>
        {/if}
        {#each reviewersDecisions as r}
          <div class="flex items-center gap-2">
            <img src={r.user.avatar_url} alt={r.user.login} class="h-5 w-5 rounded-full" />
            <span class="flex-1 truncate">{r.user.login}</span>
            <ReviewDecision state={r.state} />
          </div>
        {/each}
        {#each pr.requested_reviewers.filter((u) => !reviewersDecisions.some((r) => r.user.login === u.login)) as u}
          <div class="flex items-center gap-2 text-muted-foreground">
            <img src={u.avatar_url} alt={u.login} class="h-5 w-5 rounded-full" />
            <span class="flex-1 truncate">{u.login}</span>
            <ReviewDecision state="PENDING" />
          </div>
        {/each}
      </section>

      {#if pr.labels.length > 0}
        <section class="space-y-2">
          <h2 class="text-xs uppercase tracking-wide text-muted-foreground">Labels</h2>
          <div class="flex flex-wrap gap-1">
            {#each pr.labels as l}
              <Badge variant="outline" style={`border-color: #${l.color}; color: #${l.color};`}>
                {l.name}
              </Badge>
            {/each}
          </div>
        </section>
      {/if}

      {#if pr.milestone}
        <section class="space-y-2">
          <h2 class="text-xs uppercase tracking-wide text-muted-foreground">Milestone</h2>
          <p>{pr.milestone.title}</p>
        </section>
      {/if}
    </aside>
  </div>
{/if}
