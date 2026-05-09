<script lang="ts">
  import { onMount } from 'svelte';
  import { cn } from '$lib/utils';

  /** Raw mermaid source. Keep node labels short — the renderer wraps awkwardly. */
  export let code: string;
  /** Optional caption rendered below the diagram. */
  export let caption: string | null = null;

  let cls = '';
  export { cls as class };

  let svg = '';
  let error = '';

  onMount(async () => {
    try {
      const mermaid = (await import('mermaid')).default;
      mermaid.initialize({
        startOnLoad: false,
        // `base` lets us drive everything through themeVariables so we match the site's
        // palette exactly. The defaults skew bright; ours stay muted.
        theme: 'base',
        themeVariables: {
          background: 'transparent',
          mainBkg: '#ffffff',
          primaryColor: '#ffffff',
          primaryBorderColor: 'hsl(220, 13%, 86%)',
          primaryTextColor: 'hsl(220, 9%, 12%)',
          secondaryColor: 'hsl(220, 13%, 95%)',
          tertiaryColor: 'hsl(220, 13%, 96.5%)',
          lineColor: 'hsl(220, 8%, 50%)',
          textColor: 'hsl(220, 9%, 12%)',
          fontSize: '14px',
          edgeLabelBackground: '#ffffff',
          clusterBkg: 'hsl(220, 13%, 96.5%)',
          clusterBorder: 'hsl(220, 13%, 86%)',
          nodeBorder: 'hsl(220, 13%, 86%)',
          // Used when callers tag a node with `:::brand` — emphasises a single node in the diagram.
          arrowheadColor: 'hsl(220, 8%, 50%)'
        },
        fontFamily: '"Geist Variable", system-ui, -apple-system, sans-serif',
        flowchart: {
          htmlLabels: true,
          curve: 'basis',
          padding: 18,
          useMaxWidth: true
        }
      });

      const id = 'mmd-' + Math.random().toString(36).slice(2);
      const result = await mermaid.render(id, code);
      svg = result.svg;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });
</script>

<figure class={cn('my-7', cls)}>
  <div class="rounded-2xl bg-muted/40 px-6 py-7 overflow-x-auto">
    <div class="flex justify-center">
      {#if svg}
        <div class="mermaid-host w-full">{@html svg}</div>
      {:else if error}
        <pre class="text-xs text-destructive font-mono">Mermaid render error: {error}</pre>
      {:else}
        <!-- SSR / pre-hydration fallback. Visible until mermaid runs on mount. -->
        <pre class="text-xs text-muted-foreground font-mono whitespace-pre opacity-60">{code}</pre>
      {/if}
    </div>
  </div>
  {#if caption}
    <figcaption class="mt-3 text-xs text-muted-foreground/80 text-center">{caption}</figcaption>
  {/if}
</figure>

<style>
  .mermaid-host :global(svg) {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 0 auto;
  }
  /* Brand-red emphasis class for highlighting a single node — use `:::brand` in mermaid. */
  .mermaid-host :global(.brand > rect),
  .mermaid-host :global(.brand > polygon),
  .mermaid-host :global(.brand > circle),
  .mermaid-host :global(.brand > path) {
    fill: #ff0035 !important;
    stroke: #ff0035 !important;
  }
  .mermaid-host :global(.brand .nodeLabel),
  .mermaid-host :global(.brand .label) {
    color: #ffffff !important;
    fill: #ffffff !important;
  }
</style>
