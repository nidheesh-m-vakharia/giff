<script lang="ts">
  import type { CheckRun } from '$lib/api/types';

  export let runs: CheckRun[] | null;

  type Summary = 'success' | 'failure' | 'pending' | 'unknown';

  function summarize(rs: CheckRun[] | null): Summary {
    if (!rs || rs.length === 0) return 'unknown';
    let anyPending = false;
    let anyFailure = false;
    for (const r of rs) {
      if (r.status !== 'completed') {
        anyPending = true;
      } else if (r.conclusion === 'failure' || r.conclusion === 'timed_out') {
        anyFailure = true;
      }
    }
    if (anyFailure) return 'failure';
    if (anyPending) return 'pending';
    return 'success';
  }

  $: summary = summarize(runs);
  $: total = runs?.length ?? 0;
</script>

{#if summary === 'success'}
  <span class="inline-flex items-center gap-1.5 text-xs">
    <span class="h-1.5 w-1.5 rounded-full bg-foreground" aria-hidden="true"></span>
    CI
  </span>
{:else if summary === 'failure'}
  <span class="inline-flex items-center gap-1.5 text-xs text-brand">
    <span class="h-1.5 w-1.5 rounded-full bg-brand" aria-hidden="true"></span>
    CI
  </span>
{:else if summary === 'pending'}
  <span class="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
    <span class="h-1.5 w-1.5 rounded-full border border-foreground/40" aria-hidden="true"></span>
    CI
  </span>
{:else if total === 0}
  <!-- intentionally empty: keep the row uncluttered when no CI is configured -->
{/if}
