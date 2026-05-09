<script lang="ts">
  import { apiError, clearApiError } from '$lib/stores/error';
  import { X } from 'lucide-svelte';

  $: err = $apiError;
  $: message = err
    ? err.status === 401
      ? 'Token invalid or expired — update it in Settings.'
      : err.status === 403 && err.resetAt
        ? `Rate-limited by GitHub. Resets at ${err.resetAt.toLocaleTimeString()}.`
        : err.status === 404
          ? 'Repository not found, or token lacks access.'
          : err.message
    : '';
</script>

{#if err}
  <div
    class="flex items-center gap-3 border-b border-destructive/40 bg-destructive/10 px-4 py-2 text-sm text-destructive"
    role="alert"
  >
    <span class="flex-1">{message}</span>
    <button
      type="button"
      class="rounded p-1 hover:bg-destructive/20"
      on:click={clearApiError}
      aria-label="Dismiss error"
    >
      <X class="h-4 w-4" />
    </button>
  </div>
{/if}
