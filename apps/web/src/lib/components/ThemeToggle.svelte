<script lang="ts">
  import { Sun, Moon } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import { cn } from '$lib/utils';

  let dark = false;

  onMount(() => {
    dark = document.documentElement.classList.contains('dark');
  });

  function toggle() {
    dark = !dark;
    document.documentElement.classList.toggle('dark', dark);
    try {
      localStorage.setItem('theme', dark ? 'dark' : 'light');
    } catch {}
  }
</script>

<button
  type="button"
  on:click={toggle}
  class={cn(
    'inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-sm text-muted-foreground hover:text-foreground hover:bg-accent transition-colors'
  )}
  aria-label={dark ? 'Switch to light mode' : 'Switch to dark mode'}
>
  {#if dark}
    <Sun class="h-4 w-4" />
  {:else}
    <Moon class="h-4 w-4" />
  {/if}
</button>
