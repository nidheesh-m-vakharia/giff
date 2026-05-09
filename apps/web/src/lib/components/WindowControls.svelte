<script lang="ts">
  // Custom min / maximize / close buttons for the Tauri window.
  //
  // Default: visible whenever we're inside Tauri. We then ATTEMPT to detect macOS via the
  // OS plugin and hide ourselves there (macOS keeps its native traffic lights via
  // titleBarStyle: "Overlay"). Default-visible means a flaky OS-plugin probe can't strand
  // a Windows / Linux user with no way to close the window.

  import { onMount } from 'svelte';
  import { Minus, Square, X } from 'lucide-svelte';
  import { browser } from '$app/environment';
  import { cn } from '$lib/utils';

  let visible = false;
  let win: import('@tauri-apps/api/window').Window | null = null;

  onMount(async () => {
    if (!browser) return;
    if (!('__TAURI_INTERNALS__' in window)) return;

    // We're in Tauri — get a window handle and show controls by default.
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      win = getCurrentWindow();
      visible = true;
    } catch {
      // No way to control the window — don't show buttons that would do nothing.
      return;
    }

    // Try to demote to hidden on macOS (where the OS draws traffic lights). If the probe
    // fails we leave them shown rather than strand the user.
    try {
      const { platform } = await import('@tauri-apps/plugin-os');
      if (platform() === 'macos') visible = false;
    } catch {
      /* leave visible */
    }
  });

  async function minimize() {
    await win?.minimize();
  }
  async function toggleMaximize() {
    await win?.toggleMaximize();
  }
  async function close() {
    await win?.close();
  }

  const btn = cn(
    'inline-flex h-7 w-11 items-center justify-center text-muted-foreground',
    'hover:text-foreground hover:bg-accent transition-colors'
  );
</script>

{#if visible}
  <!-- Right-aligned strip; sits above content via z-50. The drag region in +layout.svelte
       is z-40 so these buttons stay clickable above it. -->
  <div class="fixed top-0 right-0 z-50 flex h-7 select-none">
    <button class={btn} on:click={minimize} aria-label="Minimize" type="button">
      <Minus class="h-4 w-4" />
    </button>
    <button class={btn} on:click={toggleMaximize} aria-label="Toggle maximize" type="button">
      <Square class="h-3 w-3" />
    </button>
    <button
      class={cn(btn, 'hover:bg-brand hover:text-brand-fg')}
      on:click={close}
      aria-label="Close"
      type="button"
    >
      <X class="h-4 w-4" />
    </button>
  </div>
{/if}
