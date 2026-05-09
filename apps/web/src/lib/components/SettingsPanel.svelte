<script lang="ts">
  import TokenForm from './TokenForm.svelte';
</script>

<div class="space-y-8">
  <div class="space-y-1.5">
    <h1 class="text-2xl font-semibold tracking-tight">Settings</h1>
    <p class="text-sm text-muted-foreground">
      Token and repository are stored in <code class="font-mono">localStorage</code> only — they
      never leave your browser.
    </p>
  </div>
  <TokenForm />
</div>
