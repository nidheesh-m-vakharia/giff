<script lang="ts">
  import type { IssueComment, Review } from '$lib/api/types';

  export let comments: IssueComment[] = [];
  export let reviews: Review[] = [];

  type Entry =
    | { kind: 'comment'; data: IssueComment; at: string }
    | { kind: 'review'; data: Review; at: string };

  $: entries = (() => {
    const xs: Entry[] = [];
    for (const c of comments) xs.push({ kind: 'comment', data: c, at: c.created_at });
    for (const r of reviews) {
      if (!r.body) continue; // skip empty reviews (just an approve click)
      xs.push({ kind: 'review', data: r, at: r.submitted_at ?? '' });
    }
    xs.sort((a, b) => Date.parse(a.at) - Date.parse(b.at));
    return xs;
  })();

  function fmt(at: string): string {
    return new Date(at).toLocaleString();
  }
</script>

{#if entries.length === 0}
  <p class="text-sm text-muted-foreground">No comments or reviews yet.</p>
{:else}
  <div class="space-y-4">
    {#each entries as e (`${e.kind}-${e.data.id}`)}
      <div class="flex gap-3">
        <img
          src={e.data.user.avatar_url}
          alt={e.data.user.login}
          class="h-8 w-8 rounded-full mt-1"
        />
        <div class="flex-1 rounded-md border bg-card">
          <div class="flex items-center gap-2 border-b px-3 py-1.5 bg-muted/40 text-sm">
            <a
              href={e.data.user.html_url}
              target="_blank"
              rel="noopener"
              class="font-medium"
            >
              {e.data.user.login}
            </a>
            <span class="text-muted-foreground">
              {#if e.kind === 'review'}
                {#if e.data.state === 'APPROVED'}
                  approved
                {:else if e.data.state === 'CHANGES_REQUESTED'}
                  requested changes
                {:else}
                  reviewed
                {/if}
              {:else}
                commented
              {/if}
              · {fmt(e.at)}
            </span>
          </div>
          <p class="px-3 py-2 text-sm whitespace-pre-wrap break-words">{e.data.body}</p>
        </div>
      </div>
    {/each}
  </div>
{/if}
