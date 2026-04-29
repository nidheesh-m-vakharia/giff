<script lang="ts">
  export let value: string = '';

  import { setContext } from 'svelte';
  import { writable } from 'svelte/store';

  const activeTab = writable(value);
  setContext('tabs', { activeTab });

  $: activeTab.set(value);
</script>

<div class="w-full">
  <slot />
</div>
