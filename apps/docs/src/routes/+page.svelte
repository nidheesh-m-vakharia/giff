<script lang="ts">
  import { ArrowRight } from 'lucide-svelte';
  import { REPO_URL } from '$lib/utils';
</script>

<svelte:head>
  <title>giff — stacked diffs for GitHub</title>
</svelte:head>

<main class="relative flex min-h-screen items-center justify-center px-6 py-20 overflow-hidden">
  <!-- Soft brand-red glow centered behind the logo. Pure decorative; sits below content. -->
  <div
    aria-hidden="true"
    class="pointer-events-none absolute inset-0 -z-10 flex items-center justify-center"
  >
    <div
      class="h-[28rem] w-[28rem] rounded-full bg-brand/15 blur-[120px]"
    ></div>
  </div>

  <div class="w-full max-w-2xl text-center">
    <img
      src="/logo.svg"
      alt="giff"
      class="mx-auto h-24 w-24 rounded-2xl shadow-[0_24px_60px_-20px_rgba(255,0,53,0.35)]"
    />

    <h1
      class="mt-12 text-6xl sm:text-7xl font-semibold leading-[0.95] tracking-[-0.045em]"
    >
      Stacked diffs.<br />
      <span class="text-brand">For real branches.</span>
    </h1>

    <p class="mt-6 text-lg sm:text-xl text-muted-foreground tracking-[-0.01em] max-w-lg mx-auto">
      Open source, self-hostable. CLI, dashboard, and runner — yours to deploy.
    </p>

    <div class="mt-10 flex flex-wrap items-center justify-center gap-3">
      <a
        href="/docs"
        class="inline-flex items-center gap-2 rounded-full bg-foreground text-background px-6 py-3 text-sm font-medium hover:opacity-90 transition-opacity"
      >
        Read the docs
        <ArrowRight class="h-4 w-4" />
      </a>
      <a
        href={REPO_URL}
        target="_blank"
        rel="noopener"
        class="inline-flex items-center gap-2 rounded-full border border-border bg-background/60 px-6 py-3 text-sm font-medium text-foreground hover:bg-muted transition-colors backdrop-blur"
      >
        View on GitHub
      </a>
    </div>
  </div>
</main>
