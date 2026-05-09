<script lang="ts">
  import { listMyRepos } from '$lib/api/github';
  import type { Repository } from '$lib/api/types';
  import { createEventDispatcher, onMount, tick } from 'svelte';
  import { Check, RefreshCw, Lock } from 'lucide-svelte';
  import { cn } from '$lib/utils';

  const dispatch = createEventDispatcher<{ select: Repository }>();

  /** Bound `owner/repo`. */
  export let value = '';
  /** Token used for repo listing — taken from the form, not the saved settings. */
  export let token = '';
  export let id = 'repo';
  export let placeholder = 'owner/repo';

  let repos: Repository[] = [];
  let loading = false;
  let lastFetchedToken = '';
  let error: string | null = null;

  let open = false;
  let highlight = -1;
  let inputEl: HTMLElement | null = null;

  async function fetchRepos() {
    if (!token) {
      repos = [];
      lastFetchedToken = '';
      return;
    }
    loading = true;
    error = null;
    try {
      repos = await listMyRepos(token);
      lastFetchedToken = token;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      repos = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    if (token) fetchRepos();
  });

  // When the token changes (e.g. user just typed a fresh one), invalidate the cached list
  // but don't auto-refetch on every keystroke — wait for the user to click refresh or focus.
  $: if (token !== lastFetchedToken) {
    repos = [];
    error = null;
  }

  $: filtered = (() => {
    const q = value.trim().toLowerCase();
    if (!q) return repos.slice(0, 50);
    return repos
      .filter((r) => r.full_name.toLowerCase().includes(q))
      .slice(0, 50);
  })();

  // Reset highlight whenever the filtered list changes so it's always pointing at a real item.
  $: if (filtered.length === 0) {
    highlight = -1;
  } else if (highlight >= filtered.length) {
    highlight = 0;
  }

  function pick(r: Repository) {
    value = r.full_name;
    open = false;
    dispatch('select', r);
  }

  function onFocus() {
    open = true;
    if (token && repos.length === 0 && !loading && !error) {
      fetchRepos();
    }
  }

  function onBlur() {
    // Tiny delay so a click on a list item resolves before close.
    setTimeout(() => {
      open = false;
    }, 120);
  }

  async function onKey(e: KeyboardEvent) {
    if (!open) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        open = true;
        e.preventDefault();
      }
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (filtered.length > 0) highlight = (highlight + 1) % filtered.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (filtered.length > 0)
        highlight = (highlight - 1 + filtered.length) % filtered.length;
    } else if (e.key === 'Enter') {
      if (highlight >= 0 && filtered[highlight]) {
        e.preventDefault();
        pick(filtered[highlight]);
      }
    } else if (e.key === 'Escape') {
      open = false;
    }
    await tick();
    // Scroll highlighted item into view.
    document
      .querySelector('[data-repo-highlight="true"]')
      ?.scrollIntoView({ block: 'nearest' });
  }
</script>

<div class="relative" bind:this={inputEl}>
  <div class="flex gap-2">
    <input
      {id}
      type="text"
      bind:value
      {placeholder}
      autocomplete="off"
      class={cn(
        'flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors',
        'placeholder:text-muted-foreground',
        'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring'
      )}
      on:focus={onFocus}
      on:blur={onBlur}
      on:keydown={onKey}
      on:input={() => (open = true)}
    />
    <button
      type="button"
      title="Refresh repository list"
      aria-label="Refresh repository list"
      class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-input bg-background text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50"
      on:click={fetchRepos}
      disabled={!token || loading}
    >
      <RefreshCw class={cn('h-4 w-4', loading && 'animate-spin')} />
    </button>
  </div>

  {#if open && (loading || error || filtered.length > 0)}
    <div
      class="absolute z-50 mt-1 w-full rounded-md border bg-popover text-popover-foreground shadow-md max-h-72 overflow-y-auto"
      role="listbox"
    >
      {#if loading}
        <div class="px-3 py-2 text-xs text-muted-foreground">Loading repositories…</div>
      {:else if error}
        <div class="px-3 py-2 text-xs text-destructive">{error}</div>
      {:else}
        {#each filtered as r, i (r.full_name)}
          <button
            type="button"
            data-repo-highlight={i === highlight ? 'true' : 'false'}
            class={cn(
              'flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm',
              i === highlight ? 'bg-accent text-foreground' : 'hover:bg-accent'
            )}
            on:mousedown|preventDefault={() => pick(r)}
            on:mousemove={() => (highlight = i)}
            role="option"
            aria-selected={i === highlight}
          >
            <span class="font-mono truncate flex-1">{r.full_name}</span>
            {#if r.private}
              <Lock class="h-3 w-3 text-muted-foreground" aria-label="private" />
            {/if}
            {#if r.full_name === value}
              <Check class="h-3.5 w-3.5 text-foreground" aria-hidden="true" />
            {/if}
          </button>
        {/each}
        {#if filtered.length === 0 && repos.length > 0}
          <div class="px-3 py-2 text-xs text-muted-foreground">
            No matches for "{value}"
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  {#if !loading && !error && repos.length > 0 && !open}
    <p class="mt-1.5 text-xs text-muted-foreground">
      {repos.length} repos available · click the field to choose
    </p>
  {/if}
  {#if !token && !value}
    <p class="mt-1.5 text-xs text-muted-foreground">Enter a token above to see your repos.</p>
  {/if}
</div>
