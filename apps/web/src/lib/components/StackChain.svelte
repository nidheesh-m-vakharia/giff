<script lang="ts">
  import FrameRow from './FrameRow.svelte';
  import StackNodeView from './StackNodeView.svelte';
  import type { Stack } from '$lib/api/stack';
  import type { CheckRun } from '$lib/api/types';

  export let stack: Stack;
  export let checksByRef: Record<string, CheckRun[] | null> = {};
  export let currentPrNumber: number | null = null;
</script>

{#if stack.is_linear}
  <ul>
    {#each stack.frames as frame (frame.pr.number)}
      <li>
        <FrameRow
          {frame}
          checks={checksByRef[frame.pr.head.sha] ?? null}
          isCurrent={frame.pr.number === currentPrNumber}
        />
      </li>
    {/each}
  </ul>
{:else}
  <ul>
    {#each stack.roots as root (root.frame.pr.number)}
      <StackNodeView node={root} {checksByRef} {currentPrNumber} depth={0} />
    {/each}
  </ul>
{/if}
