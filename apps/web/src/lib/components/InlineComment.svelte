<script lang="ts">
  import type { ReviewComment } from '$lib/api/types';

  export let comment: ReviewComment;

  $: when = new Date(comment.created_at).toLocaleString();
</script>

<div class="border-l-2 border-primary/40 bg-muted/40 px-3 py-2 my-1 text-sm">
  <div class="flex items-center gap-2 mb-1">
    <img
      src={comment.user.avatar_url}
      alt={comment.user.login}
      class="h-5 w-5 rounded-full"
    />
    <a href={comment.user.html_url} target="_blank" rel="noopener" class="font-medium">
      {comment.user.login}
    </a>
    <span class="text-xs text-muted-foreground">{when}</span>
  </div>
  <p class="whitespace-pre-wrap break-words">{comment.body}</p>
</div>
