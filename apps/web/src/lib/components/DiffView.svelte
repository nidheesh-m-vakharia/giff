<script lang="ts">
  import type { PullFile, ReviewComment } from '$lib/api/types';
  import { parsePatch, shikiLang, type DiffLine } from '$lib/api/diff';
  import { Button } from '$lib/components/ui/button';
  import InlineComment from './InlineComment.svelte';
  import { ChevronDown, ChevronRight } from 'lucide-svelte';
  import { onMount } from 'svelte';

  export let file: PullFile;
  export let comments: ReviewComment[] = [];

  $: hunks = parsePatch(file.patch);
  $: lang = shikiLang(file.filename);

  // Index comments by 1-based line number on the right side (post-image).
  $: commentsByLine = (() => {
    const map = new Map<number, ReviewComment[]>();
    for (const c of comments) {
      const ln = c.line ?? c.original_line;
      if (ln == null) continue;
      const list = map.get(ln) ?? [];
      list.push(c);
      map.set(ln, list);
    }
    return map;
  })();

  const COLLAPSE_THRESHOLD = 200;
  let expanded = file.changes <= COLLAPSE_THRESHOLD;

  // Per-line shiki highlighting. We highlight each line's text individually so we can
  // keep the gutter and per-line backgrounds.
  let highlighter: import('shiki').Highlighter | null = null;

  onMount(async () => {
    if (file.changes > COLLAPSE_THRESHOLD * 5) return; // skip very large files
    const shiki = await import('shiki');
    try {
      highlighter = await shiki.getHighlighter({
        themes: ['github-light'],
        langs: [lang === 'text' ? 'plaintext' : (lang as never)]
      });
    } catch {
      // Unsupported language — fall through to plain text.
      highlighter = null;
    }
  });

  function highlight(line: DiffLine): string {
    if (!highlighter) return escapeHtml(line.text);
    try {
      const html = highlighter.codeToHtml(line.text, {
        lang: lang === 'text' ? 'plaintext' : lang,
        theme: 'github-light'
      });
      // Strip wrapping <pre>/<code> so we can place the styled spans inline.
      const inner = html.replace(/^<pre[^>]*><code[^>]*>/, '').replace(/<\/code><\/pre>\s*$/, '');
      return inner;
    } catch {
      return escapeHtml(line.text);
    }
  }

  function escapeHtml(s: string): string {
    return s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  }
</script>

<div class="border rounded-md overflow-hidden bg-card">
  <button
    type="button"
    class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent"
    on:click={() => (expanded = !expanded)}
  >
    {#if expanded}
      <ChevronDown class="h-4 w-4" />
    {:else}
      <ChevronRight class="h-4 w-4" />
    {/if}
    <span class="font-mono text-sm truncate">{file.filename}</span>
    <span class="ml-auto text-xs">
      <span class="text-green-600">+{file.additions}</span>
      <span class="text-destructive ml-2">-{file.deletions}</span>
    </span>
  </button>

  {#if expanded}
    {#if !file.patch}
      <div class="px-3 py-4 text-sm text-muted-foreground border-t">
        Diff not available (binary or too large).
      </div>
    {:else}
      <div class="border-t font-mono text-xs overflow-x-auto">
        {#each hunks as hunk}
          <div class="bg-muted/60 px-3 py-1 text-muted-foreground">{hunk.header}</div>
          {#each hunk.lines as line}
            <div
              class="grid grid-cols-[3rem_3rem_1fr] gap-0 {line.kind === 'add'
                ? 'bg-green-50'
                : line.kind === 'del'
                  ? 'bg-red-50'
                  : line.kind === 'meta'
                    ? 'bg-muted/30 text-muted-foreground'
                    : ''}"
            >
              <span class="text-right pr-2 text-muted-foreground select-none">
                {line.oldNum ?? ''}
              </span>
              <span class="text-right pr-2 text-muted-foreground select-none">
                {line.newNum ?? ''}
              </span>
              <span class="px-2 whitespace-pre">
                <span class="select-none {line.kind === 'add'
                  ? 'text-green-700'
                  : line.kind === 'del'
                    ? 'text-destructive'
                    : ''}">
                  {line.kind === 'add' ? '+' : line.kind === 'del' ? '-' : ' '}
                </span>{@html highlight(line)}
              </span>
            </div>
            {#if line.newNum != null && commentsByLine.has(line.newNum)}
              <div class="grid grid-cols-[6rem_1fr]">
                <span></span>
                <div class="pr-2">
                  {#each commentsByLine.get(line.newNum) ?? [] as c}
                    <InlineComment comment={c} />
                  {/each}
                </div>
              </div>
            {/if}
          {/each}
        {/each}
      </div>
    {/if}
  {/if}
</div>
