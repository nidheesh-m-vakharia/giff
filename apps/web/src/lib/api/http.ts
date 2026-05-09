// Thin alias over `globalThis.fetch`. Kept as a separate export so call sites read
// consistently and we have a single place to swap in retry / instrumentation later.

export function http(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  return globalThis.fetch(input, init);
}
