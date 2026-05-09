<script lang="ts">
  import { cn } from '$lib/utils';
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';

  export let value: string;
  let className = '';
  export { className as class };

  const { activeTab } = getContext<{ activeTab: Writable<string> }>('tabs');
</script>

{#if $activeTab === value}
  <div
    role="tabpanel"
    class={cn(
      'mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
      className
    )}
  >
    <slot />
  </div>
{/if}
