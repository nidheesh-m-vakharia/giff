<script lang="ts">
  import '../app.css';
  import { settings } from '$lib/stores/settings';
  import { apiError } from '$lib/stores/error';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import ErrorBanner from '$lib/components/ErrorBanner.svelte';

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

<div class="min-h-screen bg-background text-foreground">
  <ErrorBanner />
  <slot />
</div>
