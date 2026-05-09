// Shapes of the GitHub REST objects we consume. Trimmed to the fields the UI uses.

export interface User {
  login: string;
  avatar_url: string;
  html_url: string;
}

export interface PullRequest {
  number: number;
  title: string;
  body: string | null;
  state: 'open' | 'closed';
  draft: boolean;
  merged: boolean;
  html_url: string;
  user: User;
  head: { ref: string; sha: string };
  base: { ref: string };
  labels: Array<{ name: string; color: string }>;
  milestone: { title: string } | null;
  requested_reviewers: User[];
  created_at: string;
  updated_at: string;
  merged_at: string | null;
  closed_at: string | null;
}

export interface IssueComment {
  id: number;
  user: User;
  body: string;
  created_at: string;
  html_url: string;
}

export type ReviewState =
  | 'APPROVED'
  | 'CHANGES_REQUESTED'
  | 'COMMENTED'
  | 'DISMISSED'
  | 'PENDING';

export interface Review {
  id: number;
  user: User;
  body: string;
  state: ReviewState;
  submitted_at: string | null;
  html_url: string;
}

export interface ReviewComment {
  id: number;
  user: User;
  body: string;
  path: string;
  line: number | null;
  original_line: number | null;
  side: 'LEFT' | 'RIGHT' | null;
  diff_hunk: string;
  created_at: string;
  html_url: string;
  in_reply_to_id?: number;
}

export interface Commit {
  sha: string;
  html_url: string;
  commit: {
    message: string;
    author: { name: string; email: string; date: string };
  };
  author: User | null;
}

export interface PullFile {
  filename: string;
  status: 'added' | 'removed' | 'modified' | 'renamed' | 'copied' | 'changed' | 'unchanged';
  additions: number;
  deletions: number;
  changes: number;
  patch?: string;
  previous_filename?: string;
}

export interface CheckRun {
  name: string;
  status: 'queued' | 'in_progress' | 'completed';
  conclusion:
    | 'success'
    | 'failure'
    | 'neutral'
    | 'cancelled'
    | 'skipped'
    | 'timed_out'
    | 'action_required'
    | null;
  html_url: string;
}

export interface CheckRunsResponse {
  total_count: number;
  check_runs: CheckRun[];
}

export interface Repository {
  full_name: string;
  description: string | null;
  private: boolean;
  html_url: string;
  pushed_at: string;
}
