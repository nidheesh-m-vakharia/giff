<script lang="ts">
  import { settings } from '$lib/stores/settings';
  import { validateRepo } from '$lib/api/github';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import RepoSelector from './RepoSelector.svelte';
  import { ExternalLink } from 'lucide-svelte';

  let token = $settings.token;
  let repo = $settings.repo;
  let saving = false;
  let message: { kind: 'ok' | 'err'; text: string } | null = null;

  $: maskedExisting = $settings.token
    ? `${$settings.token.slice(0, 4)}…${$settings.token.slice(-4)}`
    : null;

  async function save() {
    message = null;
    saving = true;
    try {
      if (!token) {
        message = { kind: 'err', text: 'Token is required.' };
        return;
      }
      if (!/^[^/\s]+\/[^/\s]+$/.test(repo)) {
        message = { kind: 'err', text: 'Repository must be in `owner/repo` form.' };
        return;
      }
      const ok = await validateRepo(repo, token);
      if (!ok) {
        message = { kind: 'err', text: 'Repository not found, or token lacks access.' };
        return;
      }
      settings.save({ token, repo });
      message = { kind: 'ok', text: 'Saved.' };
    } finally {
      saving = false;
    }
  }

  function clearAll() {
    settings.clear();
    token = '';
    repo = '';
    message = { kind: 'ok', text: 'Cleared.' };
  }
</script>

<div class="space-y-8 max-w-xl">
  <div class="space-y-2">
    <label for="token" class="block text-sm font-medium">GitHub token</label>
    <Input id="token" type="password" bind:value={token} placeholder="ghp_…" />
    {#if maskedExisting}
      <p class="text-xs text-muted-foreground">
        Currently stored: <code class="font-mono">{maskedExisting}</code>
      </p>
    {/if}
    <p class="text-xs text-muted-foreground leading-relaxed">
      Needs the <code class="font-mono text-[0.7rem] px-1 py-0.5 rounded bg-muted">repo</code> scope.
      <a
        href="https://github.com/settings/tokens/new?scopes=repo&description=giff-web"
        target="_blank"
        rel="noopener"
        class="inline-flex items-center gap-1 underline underline-offset-2 hover:text-foreground"
      >
        Create one <ExternalLink class="h-3 w-3" />
      </a>
    </p>
  </div>

  <div class="space-y-2">
    <label for="repo" class="block text-sm font-medium">Repository</label>
    <RepoSelector bind:value={repo} {token} />
  </div>

  {#if message}
    <p
      class={message.kind === 'ok' ? 'text-sm text-green-600' : 'text-sm text-destructive'}
      role="status"
    >
      {message.text}
    </p>
  {/if}

  <div class="flex gap-2">
    <Button on:click={save} disabled={saving}>{saving ? 'Validating…' : 'Save'}</Button>
    <Button variant="outline" on:click={clearAll}>Clear data</Button>
  </div>
</div>
