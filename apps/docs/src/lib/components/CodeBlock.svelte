<script lang="ts">
  import { cn } from '$lib/utils';

  let className = '';
  export { className as class };
  export let lang: string | null = null;
</script>

<div class={cn('relative my-5 rounded-2xl bg-muted/70 overflow-hidden', className)}>
  {#if lang}
    <div class="absolute top-3 right-4 text-[10px] uppercase tracking-wide text-muted-foreground font-mono pointer-events-none">
      {lang}
    </div>
  {/if}
  <pre class="overflow-x-auto p-5 text-sm leading-[1.7] font-mono text-foreground"><slot /></pre>
</div>
