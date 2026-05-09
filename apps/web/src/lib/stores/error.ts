import { writable } from 'svelte/store';
import type { GitHubApiError } from '$lib/api/github';

export type ApiErrorState = GitHubApiError | null;

export const apiError = writable<ApiErrorState>(null);

export function setApiError(err: GitHubApiError): void {
  apiError.set(err);
}

export function clearApiError(): void {
  apiError.set(null);
}
