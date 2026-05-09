<script lang="ts">
  import { cn } from '$lib/utils';
  import { Info, AlertTriangle } from 'lucide-svelte';

  let className = '';
  export { className as class };
  export let kind: 'info' | 'warn' = 'info';
</script>

<aside
  class={cn(
    // Apple-leaning: rounded-2xl, soft fill, no left bar. Color does the kind-signaling.
    'my-6 rounded-2xl px-5 py-4 text-sm',
    kind === 'warn' ? 'bg-brand/5 text-foreground' : 'bg-muted/60 text-foreground',
    className
  )}
>
  <div class="flex items-start gap-3">
    <span class="mt-0.5 shrink-0">
      {#if kind === 'warn'}
        <AlertTriangle class="h-4 w-4 text-brand" />
      {:else}
        <Info class="h-4 w-4 text-muted-foreground" />
      {/if}
    </span>
    <div class="min-w-0 flex-1 leading-relaxed [&>*:first-child]:mt-0 [&>*:last-child]:mb-0">
      <slot />
    </div>
  </div>
</aside>
