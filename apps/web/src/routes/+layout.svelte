<script lang="ts">
  import '../app.css';
  import { settings } from '$lib/stores/settings';
  import { apiError } from '$lib/stores/error';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import ErrorBanner from '$lib/components/ErrorBanner.svelte';
  import WindowControls from '$lib/components/WindowControls.svelte';

  let ready = false;
  onMount(() => {
    ready = true;
  });

  // No saved token/repo → push the user to the settings panel.
  $: if (
    ready &&
    (!$settings.token || !$settings.repo) &&
    $page.url.searchParams.get('settings') !== '1'
  ) {
    goto('/?settings=1', { replaceState: true });
  }

  // 401 from GitHub → settings panel.
  $: if ($apiError && $apiError.status === 401 && $page.url.searchParams.get('settings') !== '1') {
    goto('/?settings=1');
  }
</script>

<!-- Top drag strip — invisible, lives above all content. Lets the user drag the
     window from anywhere along the top edge regardless of which page is open.
     `data-tauri-drag-region` is a Tauri attribute; in a regular browser it's a no-op. -->
<div
  data-tauri-drag-region
  class="fixed inset-x-0 top-0 z-40 h-7"
  aria-hidden="true"
></div>

<WindowControls />

<div class="min-h-screen bg-background text-foreground">
  <ErrorBanner />
  <slot />
</div>
