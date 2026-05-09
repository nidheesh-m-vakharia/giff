<script lang="ts">
  import type { StackNode } from '$lib/api/stack';
  import PRStatusBadge from './PRStatusBadge.svelte';
  import { cn } from '$lib/utils';
  import Self from './SidebarTreeNode.svelte';

  /**
   * Recursive tree row for the sidebar. Standard `tree(1)`-style rendering:
   *   - `prefix` describes each ancestor depth: 'vert' draws a vertical line through the row
   *     (because that ancestor still has a sibling below), 'space' is empty (last in line).
   *   - `isLast` controls whether the current row's connector is `├` (more siblings below)
   *     or `└` (last sibling).
   */
  export let node: StackNode;
  export let prefix: ('vert' | 'space')[] = [];
  export let isLast = true;
  export let currentPr: number | null = null;
</script>

<a
  href={`/?pr=${node.frame.pr.number}`}
  class={cn(
    'flex items-stretch text-sm transition-colors',
    node.frame.pr.number === currentPr
      ? 'text-foreground'
      : 'text-muted-foreground hover:text-foreground'
  )}
>
  <!-- Ancestor columns: vertical line where the ancestor still has more siblings below. -->
  {#each prefix as seg, i (i)}
    <span
      class={cn('w-3 shrink-0', seg === 'vert' && 'border-l border-border')}
      aria-hidden="true"
    ></span>
  {/each}

  <!-- Current row's connector: half-line from top, optional half from middle (when more
       siblings below), and a horizontal stub at the midpoint. -->
  <span class="relative w-3 shrink-0" aria-hidden="true">
    <span class="absolute left-0 top-0 h-1/2 border-l border-border"></span>
    {#if !isLast}
      <span class="absolute left-0 top-1/2 h-1/2 border-l border-border"></span>
    {/if}
    <span class="absolute left-0 top-1/2 w-2 border-t border-border"></span>
  </span>

  <span class="flex items-center gap-2 flex-1 min-w-0 pl-1.5 py-1 min-h-[28px]">
    <span class="font-mono text-foreground truncate">{node.frame.pr.head.ref}</span>
    <span class="text-xs">#{node.frame.pr.number}</span>
    <span class="ml-auto"><PRStatusBadge pr={node.frame.pr} /></span>
  </span>
</a>

{#each node.children as child, i (child.frame.pr.number)}
  <Self
    node={child}
    prefix={[...prefix, isLast ? 'space' : 'vert']}
    isLast={i + 1 === node.children.length}
    {currentPr}
  />
{/each}
