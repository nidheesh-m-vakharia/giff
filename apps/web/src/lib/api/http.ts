// One fetch helper for the whole app. In a Tauri webview it routes through
// `@tauri-apps/plugin-http`, which proxies the request through the Rust process — no
// CORS, no `mode: 'no-cors'` shenanigans, and we get to scope which hosts are reachable
// in `tauri.conf.json` / capabilities. In a regular browser (e.g. `npm run dev` opened
// at localhost) it falls back to `globalThis.fetch`, so the dashboard remains
// browser-runnable for development.
//
// The plugin's fetch has the same signature as native fetch, so call sites don't need
// to know which one they're using.

import { browser } from '$app/environment';

let resolvedFetch: Promise<typeof globalThis.fetch> | null = null;

function isTauri(): boolean {
  if (!browser) return false;
  // Tauri 2 marker. Available as soon as the webview boots.
  return typeof (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !==
    'undefined';
}

function resolveFetch(): Promise<typeof globalThis.fetch> {
  if (resolvedFetch) return resolvedFetch;
  resolvedFetch = (async () => {
    if (isTauri()) {
      const mod = await import('@tauri-apps/plugin-http');
      // The plugin exports `fetch` with a fetch-compatible signature.
      return mod.fetch as unknown as typeof globalThis.fetch;
    }
    return globalThis.fetch.bind(globalThis);
  })();
  return resolvedFetch;
}

export async function http(
  input: RequestInfo | URL,
  init?: RequestInit
): Promise<Response> {
  const f = await resolveFetch();
  return f(input, init);
}
