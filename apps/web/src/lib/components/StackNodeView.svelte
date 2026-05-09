<script lang="ts">
  import FrameRow from './FrameRow.svelte';
  import type { StackNode } from '$lib/api/stack';
  import type { CheckRun } from '$lib/api/types';
  import Self from './StackNodeView.svelte';

  export let node: StackNode;
  export let checksByRef: Record<string, CheckRun[] | null> = {};
  export let currentPrNumber: number | null = null;
  export let depth = 0;
</script>

<li style="padding-left: {depth * 1.25}rem">
  <FrameRow
    frame={node.frame}
    checks={checksByRef[node.frame.pr.head.sha] ?? null}
    isCurrent={node.frame.pr.number === currentPrNumber}
  />
</li>
{#each node.children as child (child.frame.pr.number)}
  <Self node={child} {checksByRef} {currentPrNumber} depth={depth + 1} />
{/each}
