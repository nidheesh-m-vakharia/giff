<script lang="ts">
  import { cn } from '$lib/utils';

  let className = '';
  export { className as class };
  export let value: string = '';
  export let type: 'text' | 'password' | 'email' | 'number' | 'search' | 'url' = 'text';
  export let placeholder = '';
  export let id: string | undefined = undefined;
  export let name: string | undefined = undefined;
  export let disabled = false;
  export let readonly = false;
  export let autocomplete: string | undefined = undefined;
</script>

{#if type === 'password'}
  <input
    type="password"
    class={cn(
      'flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors',
      'placeholder:text-muted-foreground',
      'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring',
      'disabled:cursor-not-allowed disabled:opacity-50',
      className
    )}
    bind:value
    {placeholder}
    {id}
    {name}
    {disabled}
    {readonly}
    {autocomplete}
    on:input
    on:change
    on:focus
    on:blur
  />
{:else}
  <input
    type="text"
    class={cn(
      'flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors',
      'placeholder:text-muted-foreground',
      'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring',
      'disabled:cursor-not-allowed disabled:opacity-50',
      className
    )}
    bind:value
    {placeholder}
    {id}
    {name}
    {disabled}
    {readonly}
    {autocomplete}
    on:input
    on:change
    on:focus
    on:blur
  />
{/if}
