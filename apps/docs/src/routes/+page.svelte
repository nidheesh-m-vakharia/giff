<script lang="ts">
  import { ArrowRight } from 'lucide-svelte';
</script>

<svelte:head>
  <title>giff stack — stacked diffs for GitHub</title>
</svelte:head>

<main class="flex min-h-screen items-center justify-center px-6 py-20">
  <div class="w-full max-w-xl text-center">
    <img
      src="/logo.svg"
      alt="giff stack"
      class="mx-auto h-20 w-20 rounded-2xl"
    />

    <h1 class="mt-10 text-5xl sm:text-6xl font-semibold tracking-[-0.04em]">
      giff stack
    </h1>

    <p class="mt-4 text-base text-muted-foreground">
      Stacked diffs for GitHub. Open source.
    </p>

    <a
      href="/docs"
      class="mt-10 inline-flex items-center gap-2 rounded-full bg-foreground text-background px-6 py-3 text-sm font-medium hover:opacity-90 transition-opacity"
    >
      Read the docs
      <ArrowRight class="h-4 w-4" />
    </a>
  </div>
</main>
