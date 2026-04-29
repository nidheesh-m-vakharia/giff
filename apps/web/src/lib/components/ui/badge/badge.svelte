<script lang="ts">
  export let variant: 'default' | 'secondary' | 'destructive' | 'outline' = 'default';

  const variantClasses: Record<string, string> = {
    default: 'bg-primary text-primary-foreground',
    secondary: 'bg-secondary text-secondary-foreground',
    destructive: 'bg-destructive text-destructive-foreground',
    outline: 'border border-border text-foreground',
  };
</script>

<span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold {variantClasses[variant]}">
  <slot />
</span>
