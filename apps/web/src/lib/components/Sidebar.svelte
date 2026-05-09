<script lang="ts">
  import { onMount } from 'svelte';
  import { settings } from '$lib/stores/settings';
  import { listOpenPulls, listCheckRuns } from '$lib/api/github';
  import { groupIntoStacks, stackName, type StackGrouping } from '$lib/api/stack';
  import type { CheckRun, Repository } from '$lib/api/types';
  import PRStatusBadge from './PRStatusBadge.svelte';
  import RepoSelector from './RepoSelector.svelte';
  import SidebarTreeNode from './SidebarTreeNode.svelte';
  import { page } from '$app/stores';
  import { Settings as SettingsIcon, ChevronDown } from 'lucide-svelte';
  import { cn } from '$lib/utils';
  import ThemeToggle from './ThemeToggle.svelte';

  const REPO_URL = 'https://github.com/nidheesh-m-vakharia/giff';

  let grouping: StackGrouping | null = null;
  let checksByRef: Record<string, CheckRun[] | null> = {};
  let loading = true;

  let switcherOpen = false;
  let pendingRepo = '';

  $: currentPr = Number($page.url.searchParams.get('pr')) || null;
  $: settingsOpen = $page.url.searchParams.get('settings') === '1';

  async function loadStacks() {
    if (!$settings.token || !$settings.repo) {
      grouping = null;
      checksByRef = {};
      loading = false;
      return;
    }
    loading = true;
    try {
      const prs = await listOpenPulls();
      grouping = groupIntoStacks(prs);
      const heads = grouping.stacks.flatMap((s) => s.frames.map((f) => f.pr.head.sha));
      const fetched: Record<string, CheckRun[] | null> = {};
      await Promise.all(
        heads.map(async (sha) => {
          try {
            fetched[sha] = (await listCheckRuns(sha)).check_runs;
          } catch {
            fetched[sha] = null;
          }
        })
      );
      checksByRef = fetched;
    } catch {
      grouping = null;
    } finally {
      loading = false;
    }
  }

  onMount(loadStacks);

  function toggleSwitcher() {
    switcherOpen = !switcherOpen;
    if (switcherOpen) pendingRepo = $settings.repo;
  }

  async function onRepoSelected(e: CustomEvent<Repository>) {
    const repo = e.detail.full_name;
    if (repo === $settings.repo) {
      switcherOpen = false;
      return;
    }
    settings.save({ token: $settings.token, repo });
    switcherOpen = false;
    grouping = null;
    checksByRef = {};
    await loadStacks();
  }

  function rowClass(prNumber: number): string {
    return cn(
      'flex items-center gap-2 py-1 text-sm transition-colors w-full',
      prNumber === currentPr ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'
    );
  }
</script>

<aside class="flex w-72 shrink-0 flex-col border-r bg-background">
  <!-- Brand + repo switcher. -->
  <div class="px-4 pt-6 pb-3 space-y-3">
    <a
      href="/"
      class="flex items-center gap-3 font-semibold tracking-tight"
    >
      <img src="/logo.svg" alt="giff stack" class="h-9 w-9 rounded-md shadow-sm" />
      <span class="text-2xl">giff stack</span>
    </a>

    <button
      type="button"
      class={cn(
        'flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-xs transition-colors',
        switcherOpen
          ? 'bg-accent text-foreground'
          : 'text-muted-foreground hover:text-foreground hover:bg-accent'
      )}
      on:click={toggleSwitcher}
      aria-expanded={switcherOpen}
    >
      <span class="font-mono truncate flex-1 text-left">
        {$settings.repo || 'Choose a repository'}
      </span>
      <ChevronDown
        class={cn('h-3.5 w-3.5 transition-transform', switcherOpen && 'rotate-180')}
      />
    </button>

    {#if switcherOpen}
      <div class="pt-1">
        <RepoSelector
          bind:value={pendingRepo}
          token={$settings.token}
          on:select={onRepoSelected}
        />
      </div>
    {/if}
  </div>

  <!-- Stacks -->
  <nav class="flex-1 overflow-y-auto px-4 pb-4">
    {#if loading}
      <p class="text-xs text-muted-foreground py-2">Loading…</p>
    {:else if !$settings.token || !$settings.repo}
      <a
        href="/?settings=1"
        class="block text-xs text-muted-foreground hover:text-foreground py-2"
      >
        Set up token + repo →
      </a>
    {:else if !grouping || (grouping.stacks.length === 0 && grouping.ungrouped.length === 0)}
      <p class="text-xs text-muted-foreground py-2">No open PRs.</p>
    {:else}
      <div class="space-y-8">
        {#each grouping.stacks as stack (stack.id)}
          <section class="space-y-1.5">
            <header class="flex items-baseline gap-2 pb-2 border-b border-foreground/10">
              <span
                class="inline-block w-1 h-3 rounded-full bg-brand"
                aria-hidden="true"
              ></span>
              <h3 class="font-mono text-sm font-medium text-foreground truncate">
                {stackName(stack)}
              </h3>
              <span class="text-[10px] text-muted-foreground ml-auto tabular-nums">
                {stack.frames.length}/{stack.total}
              </span>
            </header>
            {#if stack.is_linear}
              <!-- Linear chains don't benefit from connectors (they'd just stair-step). -->
              {#each stack.frames as frame (frame.pr.number)}
                <a href={`/?pr=${frame.pr.number}`} class={rowClass(frame.pr.number)}>
                  <span class="font-mono text-foreground truncate">{frame.pr.head.ref}</span>
                  <span class="text-xs">#{frame.pr.number}</span>
                  <span class="ml-auto"><PRStatusBadge pr={frame.pr} /></span>
                </a>
              {/each}
            {:else}
              <!-- True trees: render with `├`/`└` connectors and vertical lines for ancestors. -->
              {#each stack.roots as root, i (root.frame.pr.number)}
                <SidebarTreeNode
                  node={root}
                  isLast={i + 1 === stack.roots.length}
                  {currentPr}
                />
              {/each}
            {/if}
          </section>
        {/each}

        {#if grouping.ungrouped.length > 0}
          <section class="space-y-1.5">
            <header class="flex items-baseline gap-2 pb-2 border-b border-foreground/10">
              <span
                class="inline-block w-1 h-3 rounded-full bg-foreground/20"
                aria-hidden="true"
              ></span>
              <h3 class="text-sm font-medium text-muted-foreground">Ungrouped</h3>
              <span class="text-[10px] text-muted-foreground ml-auto tabular-nums">
                {grouping.ungrouped.length}
              </span>
            </header>
            {#each grouping.ungrouped as pr (pr.number)}
              <a href={`/?pr=${pr.number}`} class={rowClass(pr.number)}>
                <span class="font-mono text-foreground truncate">{pr.head.ref}</span>
                <span class="text-xs">#{pr.number}</span>
                <span class="ml-auto"><PRStatusBadge {pr} /></span>
              </a>
            {/each}
          </section>
        {/if}
      </div>
    {/if}
  </nav>

  <!-- Footer: settings + theme + source -->
  <div class="border-t px-4 py-3 flex items-center gap-1 text-sm">
    <a
      href="/?settings=1"
      class={cn(
        'inline-flex items-center gap-1.5 rounded-md px-2 py-1 transition-colors',
        settingsOpen
          ? 'bg-accent text-foreground'
          : 'text-muted-foreground hover:text-foreground hover:bg-accent'
      )}
    >
      <SettingsIcon class="h-4 w-4" />
      Settings
    </a>
    <div class="ml-auto flex items-center">
      <ThemeToggle />
    </div>
    <a
      href={REPO_URL}
      target="_blank"
      rel="noopener"
      class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
      aria-label="Source on GitHub"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="currentColor"
        class="h-4 w-4"
        aria-hidden="true"
      >
        <path
          d="M12 .5C5.65.5.5 5.65.5 12.02c0 5.1 3.29 9.41 7.86 10.93.58.11.79-.25.79-.56 0-.27-.01-1.18-.02-2.14-3.2.7-3.87-1.36-3.87-1.36-.52-1.32-1.27-1.67-1.27-1.67-1.04-.71.08-.7.08-.7 1.15.08 1.76 1.18 1.76 1.18 1.02 1.75 2.69 1.24 3.34.95.1-.74.4-1.24.72-1.53-2.55-.29-5.24-1.28-5.24-5.7 0-1.26.45-2.29 1.18-3.1-.12-.29-.51-1.46.11-3.04 0 0 .96-.31 3.16 1.18a10.95 10.95 0 0 1 5.76 0c2.2-1.49 3.16-1.18 3.16-1.18.62 1.58.23 2.75.11 3.04.74.81 1.18 1.84 1.18 3.1 0 4.43-2.7 5.41-5.27 5.69.41.36.78 1.06.78 2.13 0 1.54-.01 2.78-.01 3.16 0 .31.21.68.8.56 4.56-1.52 7.85-5.83 7.85-10.93C23.5 5.65 18.35.5 12 .5Z"
        />
      </svg>
      Source
    </a>
  </div>
</aside>
