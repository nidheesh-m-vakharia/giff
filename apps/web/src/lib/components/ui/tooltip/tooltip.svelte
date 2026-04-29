<script lang="ts">
  export let content: string = '';

  let visible = false;
</script>

<div class="relative inline-block">
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    on:mouseenter={() => (visible = true)}
    on:mouseleave={() => (visible = false)}
    on:focus={() => (visible = true)}
    on:blur={() => (visible = false)}
  >
    <slot />
  </div>
  {#if visible && content}
    <div
      role="tooltip"
      class="absolute bottom-full left-1/2 z-50 mb-1 -translate-x-1/2 rounded-md bg-primary px-3 py-1.5 text-xs text-primary-foreground shadow-md"
    >
      {content}
    </div>
  {/if}
</div>
