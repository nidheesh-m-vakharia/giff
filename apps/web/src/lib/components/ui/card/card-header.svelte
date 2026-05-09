<script lang="ts">
  import { cn } from '$lib/utils';
  let className = '';
  export { className as class };
</script>

<div class={cn('flex flex-col space-y-1.5 p-6', className)}>
  <slot />
</div>
