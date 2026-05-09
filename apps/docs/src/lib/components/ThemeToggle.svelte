<script lang="ts">
  import { Sun, Moon } from 'lucide-svelte';
  import { onMount } from 'svelte';

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
  class="inline-flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
  aria-label={dark ? 'Switch to light mode' : 'Switch to dark mode'}
>
  {#if dark}
    <Sun class="h-4 w-4" />
  {:else}
    <Moon class="h-4 w-4" />
  {/if}
</button>
