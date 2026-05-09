<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import type { ReviewState } from '$lib/api/types';
  import { Check, X, MessageSquare, Clock } from 'lucide-svelte';

  export let state: ReviewState;
</script>

{#if state === 'APPROVED'}
  <Badge class="bg-green-600 hover:bg-green-700 text-white gap-1">
    <Check class="h-3 w-3" /> approved
  </Badge>
{:else if state === 'CHANGES_REQUESTED'}
  <Badge variant="destructive" class="gap-1">
    <X class="h-3 w-3" /> changes requested
  </Badge>
{:else if state === 'COMMENTED'}
  <Badge variant="secondary" class="gap-1">
    <MessageSquare class="h-3 w-3" /> commented
  </Badge>
{:else if state === 'PENDING'}
  <Badge variant="outline" class="gap-1">
    <Clock class="h-3 w-3" /> pending
  </Badge>
{:else}
  <Badge variant="outline">{state.toLowerCase()}</Badge>
{/if}
