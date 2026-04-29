<script lang="ts">
  export let value: string;

  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';

  const { activeTab } = getContext<{ activeTab: Writable<string> }>('tabs');
  $: isActive = $activeTab === value;
</script>

<button
  type="button"
  role="tab"
  aria-selected={isActive}
  on:click={() => activeTab.set(value)}
  class="inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 {isActive
    ? 'bg-background text-foreground shadow-sm'
    : 'hover:bg-background/50'}"
>
  <slot />
</button>
