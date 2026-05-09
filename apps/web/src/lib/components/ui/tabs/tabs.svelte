<script lang="ts">
  import { cn } from '$lib/utils';
  import { setContext } from 'svelte';
  import { writable } from 'svelte/store';

  export let value: string = '';
  let className = '';
  export { className as class };

  const activeTab = writable(value);
  setContext('tabs', { activeTab });

  $: activeTab.set(value);
</script>

<div class={cn('w-full', className)}>
  <slot />
</div>
