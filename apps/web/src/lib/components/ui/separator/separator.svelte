<script lang="ts">
  export let orientation: 'horizontal' | 'vertical' = 'horizontal';
  export let decorative = true;
</script>

<div
  role={decorative ? 'none' : 'separator'}
  aria-orientation={decorative ? undefined : orientation}
  class={orientation === 'horizontal'
    ? 'h-[1px] w-full bg-border'
    : 'h-full w-[1px] bg-border'}
/>
