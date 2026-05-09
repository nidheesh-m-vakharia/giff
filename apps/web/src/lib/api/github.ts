import { get } from 'svelte/store';
import { settings } from '$lib/stores/settings';
import { setApiError } from '$lib/stores/error';
import { http } from './http';
import type {
  CheckRunsResponse,
  Commit,
  IssueComment,
  PullFile,
  PullRequest,
  Review,
  ReviewComment
} from './types';

const BASE = 'https://api.github.com';

export class GitHubApiError extends Error {
  status: number;
  resetAt: Date | null;
  constructor(message: string, status: number, resetAt: Date | null = null) {
    super(message);
    this.status = status;
    this.resetAt = resetAt;
  }
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const { token } = get(settings);
  const res = await http(`${BASE}${path}`, {
    ...init,
    headers: {
      Accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(init.headers ?? {})
    }
  });

  if (!res.ok) {
    let resetAt: Date | null = null;
    if (res.status === 403) {
      const reset = res.headers.get('x-ratelimit-reset');
      if (reset) resetAt = new Date(Number(reset) * 1000);
    }
    let message = `GitHub ${res.status}`;
    try {
      const body = (await res.json()) as { message?: string };
      if (body?.message) message = body.message;
    } catch {
      /* ignore json parse */
    }
    const err = new GitHubApiError(message, res.status, resetAt);
    setApiError(err);
    throw err;
  }

  return (await res.json()) as T;
}

function repoPath(): string {
  const { repo } = get(settings);
  if (!repo) throw new Error('repo not configured');
  return repo;
}

export async function listOpenPulls(): Promise<PullRequest[]> {
  return request<PullRequest[]>(`/repos/${repoPath()}/pulls?state=open&per_page=100`);
}

export async function getPull(num: number): Promise<PullRequest> {
  return request<PullRequest>(`/repos/${repoPath()}/pulls/${num}`);
}

export async function listIssueComments(num: number): Promise<IssueComment[]> {
  return request<IssueComment[]>(`/repos/${repoPath()}/issues/${num}/comments?per_page=100`);
}

export async function listReviews(num: number): Promise<Review[]> {
  return request<Review[]>(`/repos/${repoPath()}/pulls/${num}/reviews?per_page=100`);
}

export async function listReviewComments(num: number): Promise<ReviewComment[]> {
  return request<ReviewComment[]>(`/repos/${repoPath()}/pulls/${num}/comments?per_page=100`);
}

export async function listCommits(num: number): Promise<Commit[]> {
  return request<Commit[]>(`/repos/${repoPath()}/pulls/${num}/commits?per_page=100`);
}

export async function listFiles(num: number): Promise<PullFile[]> {
  return request<PullFile[]>(`/repos/${repoPath()}/pulls/${num}/files?per_page=100`);
}

export async function listCheckRuns(ref: string): Promise<CheckRunsResponse> {
  return request<CheckRunsResponse>(`/repos/${repoPath()}/commits/${ref}/check-runs`);
}

export async function validateRepo(repo: string, token: string): Promise<boolean> {
  const res = await http(`${BASE}/repos/${repo}`, {
    headers: {
      Accept: 'application/vnd.github+json',
      ...(token ? { Authorization: `Bearer ${token}` } : {})
    }
  });
  return res.ok;
}

/**
 * List repos the given token has access to. Uses an explicit `token` parameter (rather than
 * reading from the settings store) so the Settings page can preview repos for a token the
 * user has typed but not yet saved.
 *
 * Bypasses the global error banner — the caller (RepoSelector) handles its own UI state for
 * "loading", "no token", and "fetch failed" so the banner doesn't fire during keystrokes.
 */
export async function listMyRepos(token: string): Promise<import('./types').Repository[]> {
  const res = await http(
    `${BASE}/user/repos?per_page=100&sort=pushed&affiliation=owner,collaborator,organization_member`,
    {
      headers: {
        Accept: 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
        Authorization: `Bearer ${token}`
      }
    }
  );
  if (!res.ok) {
    throw new Error(`GitHub ${res.status}`);
  }
  return (await res.json()) as import('./types').Repository[];
}
