<script lang="ts">
  import '../app.css';
  import { REPO_URL } from '$lib/utils';
  import { page } from '$app/stores';
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';

  $: isHome = $page.url.pathname === '/';
  $: onDocs = $page.url.pathname.startsWith('/docs');
</script>

<div class="min-h-screen flex flex-col">
  {#if !isHome}
    <header class="sticky top-0 z-30 backdrop-blur bg-background/80">
      <div class="mx-auto flex h-14 items-center gap-6 px-6 max-w-6xl">
        <a href="/" class="flex items-center gap-2.5 font-semibold tracking-tight">
          <img src="/logo.svg" alt="giff stack" class="h-6 w-6 rounded-md shadow-sm" />
          <span>giff stack</span>
        </a>
        <nav class="ml-auto flex items-center gap-5 text-sm">
          <a
            href="/docs"
            class="transition-colors {onDocs
              ? 'text-foreground font-medium'
              : 'text-muted-foreground hover:text-foreground'}"
          >
            Docs
          </a>
          <a
            href={REPO_URL}
            target="_blank"
            rel="noopener"
            class="inline-flex items-center gap-1.5 text-muted-foreground hover:text-foreground transition-colors"
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
            <span class="hidden sm:inline">GitHub</span>
          </a>
          <ThemeToggle />
        </nav>
      </div>
    </header>
  {:else}
    <div class="absolute right-4 top-4 z-30">
      <ThemeToggle />
    </div>
  {/if}

  <slot />

  {#if !isHome}
    <footer class="mt-20">
      <div
        class="mx-auto max-w-6xl px-6 py-8 text-xs text-muted-foreground/70 flex items-center justify-between gap-4"
      >
        <span>
          <span class="font-mono">giff stack</span> · open source · MIT
        </span>
        <a href={REPO_URL} target="_blank" rel="noopener" class="hover:text-foreground">
          Source
        </a>
      </div>
    </footer>
  {/if}
</div>
