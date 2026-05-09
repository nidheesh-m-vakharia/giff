<script lang="ts">
  import PRStatusBadge from './PRStatusBadge.svelte';
  import CIBadge from './CIBadge.svelte';
  import type { StackedPull } from '$lib/api/stack';
  import type { CheckRun } from '$lib/api/types';

  export let frame: StackedPull;
  export let checks: CheckRun[] | null = null;
  export let isCurrent = false;
</script>

<a
  href={`/pr/${frame.pr.number}`}
  class="flex items-center gap-3 py-1.5 text-sm transition-colors {isCurrent
    ? 'text-foreground'
    : 'text-muted-foreground hover:text-foreground'}"
>
  <span class="font-mono text-foreground">{frame.pr.head.ref}</span>
  <span class="text-xs">#{frame.pr.number}</span>
  <PRStatusBadge pr={frame.pr} />
  {#if checks !== null}
    <CIBadge runs={checks} />
  {/if}
  <span class="ml-auto truncate max-w-md">{frame.pr.title}</span>
</a>
