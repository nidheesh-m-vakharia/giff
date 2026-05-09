<script lang="ts">
  import Sidebar from '$lib/components/Sidebar.svelte';
  import PRPanel from '$lib/components/PRPanel.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import { page } from '$app/stores';

  $: prNumber = (() => {
    const v = Number($page.url.searchParams.get('pr'));
    return Number.isFinite(v) && v > 0 ? v : null;
  })();
  $: showSettings = $page.url.searchParams.get('settings') === '1';
</script>

<div class="flex min-h-screen">
  <Sidebar />
  <main class="flex-1 min-w-0 px-8 py-8">
    {#if showSettings}
      <SettingsPanel />
    {:else if prNumber !== null}
      {#key prNumber}
        <PRPanel {prNumber} />
      {/key}
    {:else}
      <div class="flex h-full items-center justify-center">
        <p class="text-sm text-muted-foreground">Select a PR from the sidebar.</p>
      </div>
    {/if}
  </main>
</div>
