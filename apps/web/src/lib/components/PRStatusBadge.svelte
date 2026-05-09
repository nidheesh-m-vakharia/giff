<script lang="ts">
  import type { PullRequest } from '$lib/api/types';

  export let pr: Pick<PullRequest, 'state' | 'draft' | 'merged'>;

  $: label = pr.merged ? 'merged' : pr.draft ? 'draft' : pr.state;
  // Same palette as the big PR header pill: filled foreground for "open", brand red for
  // "merged", outlined neutral for "draft" / "closed". Keeps the sidebar consistent.
  $: dotClass = pr.merged
    ? 'bg-brand'
    : pr.state === 'closed'
      ? 'bg-foreground/30'
      : pr.draft
        ? 'border border-foreground/40'
        : 'bg-foreground';
</script>

<span class="inline-flex items-center gap-1.5 text-xs">
  <span class={`h-1.5 w-1.5 rounded-full ${dotClass}`} aria-hidden="true"></span>
  {label}
</span>
