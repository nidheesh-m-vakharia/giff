// Mirrors crates/giff-core RemoteStackMeta. PR descriptions written by `giff push` carry a
// fenced ```giff JSON block; we extract it here to reconstruct stacks from PRs alone.

import type { PullRequest } from './types';

export interface RemoteStackMeta {
  stack_id: string;
  frame_id: string;
  /** Set on PRs pushed by tree-aware giff. Undefined for pre-tree (legacy) PRs. */
  parent_frame_id?: string | null;
  position: number;
  total: number;
}

const FENCE_RE = /```giff\s*\n([\s\S]*?)\n```/;

export function parseStackMeta(body: string | null): RemoteStackMeta | null {
  if (!body) return null;
  const match = body.match(FENCE_RE);
  if (!match) return null;
  try {
    const parsed = JSON.parse(match[1]) as Partial<RemoteStackMeta>;
    if (
      typeof parsed.stack_id === 'string' &&
      typeof parsed.frame_id === 'string' &&
      typeof parsed.position === 'number' &&
      typeof parsed.total === 'number'
    ) {
      return parsed as RemoteStackMeta;
    }
  } catch {
    /* malformed block — treat as ungrouped */
  }
  return null;
}

export interface StackedPull {
  pr: PullRequest;
  meta: RemoteStackMeta;
}

/**
 * A node in the rendered tree. Children are direct subtree descendants.
 * For linear stacks, every node has 0 or 1 child.
 */
export interface StackNode {
  frame: StackedPull;
  children: StackNode[];
}

export interface Stack {
  id: string;
  total: number;
  /** Topologically ordered (parents before children); for linear stacks this is bottom→top. */
  frames: StackedPull[];
  /** Tree shape — one or more roots. Each root targets the trunk. */
  roots: StackNode[];
  /** True when there's a single root and every node has at most one child. */
  is_linear: boolean;
}

export interface StackGrouping {
  stacks: Stack[];
  ungrouped: PullRequest[];
}

/**
 * Build the tree for a single stack's frames. Uses `parent_frame_id` when present;
 * falls back to linear `position`-ordering for legacy PRs that predate tree support.
 */
function buildTree(frames: StackedPull[]): {
  roots: StackNode[];
  ordered: StackedPull[];
  isLinear: boolean;
} {
  const hasParentField = frames.some((f) => f.meta.parent_frame_id !== undefined);

  if (!hasParentField) {
    // Legacy: position is authoritative. Treat as a linear chain.
    const sorted = [...frames].sort((a, b) => a.meta.position - b.meta.position);
    let roots: StackNode[] = [];
    let cursor: StackNode | null = null;
    for (const f of sorted) {
      const node: StackNode = { frame: f, children: [] };
      if (!cursor) roots = [node];
      else cursor.children = [node];
      cursor = node;
    }
    return { roots, ordered: sorted, isLinear: true };
  }

  const byId = new Map<string, StackedPull>();
  for (const f of frames) byId.set(f.meta.frame_id, f);

  const childrenOf = new Map<string, StackedPull[]>();
  const rootFrames: StackedPull[] = [];
  for (const f of frames) {
    const parentId = f.meta.parent_frame_id ?? null;
    if (parentId == null || !byId.has(parentId)) {
      rootFrames.push(f);
    } else {
      const list = childrenOf.get(parentId) ?? [];
      list.push(f);
      childrenOf.set(parentId, list);
    }
  }

  // Sort siblings deterministically by position so the rendered order is stable.
  const sortByPos = (xs: StackedPull[]) => xs.sort((a, b) => a.meta.position - b.meta.position);
  sortByPos(rootFrames);
  for (const list of childrenOf.values()) sortByPos(list);

  const ordered: StackedPull[] = [];
  const buildNode = (f: StackedPull): StackNode => {
    ordered.push(f);
    const kids = (childrenOf.get(f.meta.frame_id) ?? []).map(buildNode);
    return { frame: f, children: kids };
  };
  const roots = rootFrames.map(buildNode);

  const isLinear =
    roots.length === 1 && Array.from(childrenOf.values()).every((c) => c.length <= 1);

  return { roots, ordered, isLinear };
}

export function groupIntoStacks(prs: PullRequest[]): StackGrouping {
  const buckets = new Map<string, StackedPull[]>();
  const ungrouped: PullRequest[] = [];

  for (const pr of prs) {
    const meta = parseStackMeta(pr.body);
    if (!meta) {
      ungrouped.push(pr);
      continue;
    }
    const list = buckets.get(meta.stack_id) ?? [];
    list.push({ pr, meta });
    buckets.set(meta.stack_id, list);
  }

  const stacks: Stack[] = Array.from(buckets.entries()).map(([id, frames]) => {
    const { roots, ordered, isLinear } = buildTree(frames);
    return {
      id,
      total: frames[0]?.meta.total ?? frames.length,
      frames: ordered,
      roots,
      is_linear: isLinear
    };
  });

  // Most recently updated stacks first.
  stacks.sort((a, b) => {
    const aMax = Math.max(...a.frames.map((f) => Date.parse(f.pr.updated_at)));
    const bMax = Math.max(...b.frames.map((f) => Date.parse(f.pr.updated_at)));
    return bMax - aMax;
  });

  return { stacks, ungrouped };
}

// Friendly name for a stack — borrows from a root frame's branch.
export function stackName(stack: Stack): string {
  const root = stack.roots[0]?.frame ?? stack.frames[0];
  if (!root) return stack.id.slice(0, 8);
  const ref = root.pr.head.ref;
  return ref.replace(/[-/_]?\d+$/, '') || ref;
}
