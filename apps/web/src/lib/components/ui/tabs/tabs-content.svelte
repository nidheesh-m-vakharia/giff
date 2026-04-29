<script lang="ts">
  export let value: string;

  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';

  const { activeTab } = getContext<{ activeTab: Writable<string> }>('tabs');
</script>

{#if $activeTab === value}
  <div role="tabpanel" class="mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
    <slot />
  </div>
{/if}
