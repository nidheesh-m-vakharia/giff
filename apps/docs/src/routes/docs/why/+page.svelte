<script lang="ts">
  import Note from '$lib/components/Note.svelte';
  import Mermaid from '$lib/components/Mermaid.svelte';
</script>

<svelte:head>
  <title>Why we made this · giff stack</title>
</svelte:head>

<h1>Why we made this</h1>

<p class="lead text-muted-foreground">
  Stacked diffs aren't a new idea. The reason this project exists is that the tooling around it is
  either closed-source SaaS or fragmented. <span class="font-mono">giff stack</span> is an attempt at
  one open, self-hostable, opinionated implementation.
</p>

<h2>The problem</h2>

<p>
  Big PRs review slowly. A 2,000-line PR is a multi-day game of comment ping-pong: the reviewer
  hits an issue at line 80, asks a clarifying question, waits for an answer, comes back, finds a
  related issue at line 1,400. Meanwhile the author can't merge anything. Meanwhile the trunk
  moves and the rebase pile-up grows.
</p>

<p>
  Stacked diffs solve this by letting you split one logical change into a chain of small,
  reviewable layers — each independently mergeable, each building on the previous. The reviewer
  picks up layer 1 today, layer 2 when ready, layer 3 next week. Five small reviews running in
  parallel beat one big review running in serial.
</p>

<Mermaid
  caption="Big PR (left) vs stack of small PRs (right). Same total work; the stacked version reviews in parallel."
  code={`flowchart LR
  subgraph BIG[ Big PR — serial ]
    direction TB
    big["<b>one PR</b><br/><span style='color:#86868b;font-size:12px'>2,000 LOC<br/>1 review pass</span>"]:::big
  end
  subgraph STACK[ Stacked — parallel ]
    direction BT
    pr1["PR 1<br/><span style='color:#86868b;font-size:11px'>merged</span>"]:::done
    pr2["PR 2<br/><span style='color:#86868b;font-size:11px'>merged</span>"]:::done
    pr3["PR 3<br/><span style='color:#86868b;font-size:11px'>approved</span>"]
    pr4["PR 4<br/><span style='color:#86868b;font-size:11px'>under review</span>"]:::brand
    pr1 --> pr2 --> pr3 --> pr4
  end
  classDef big fill:#ffffff,stroke:hsl(220,13%,86%),color:hsl(220,9%,12%);
  classDef done fill:hsl(220,13%,95%),stroke:hsl(220,13%,86%),color:hsl(220,8%,46%);
  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<h2>Why not just use [existing tool]</h2>

<p>The honest comparison set:</p>

<h3>Graphite</h3>

<p>
  Closest match. Has a free open-source CLI (<code>gt</code>) and a paid hosted SaaS for the dashboard,
  reviewer pad, and merge queue. We borrowed many primitives from their model. The gaps that
  motivated <span class="font-mono">giff stack</span>:
</p>

<ul>
  <li>The dashboard and merge automation are SaaS-only — your token + PR metadata live in their cloud.</li>
  <li>No self-hosted option. If your repo is in a regulated environment that can't send token+repo data to a third party, you're stuck.</li>
  <li>Per-user pricing scales the wrong way for small teams using stacked diffs heavily.</li>
</ul>

<h3>ghstack</h3>

<p>
  Meta's open-source CLI. Closest in spirit; <span class="font-mono">giff stack</span>'s data model
  borrows from it. ghstack is excellent at the CLI layer and stops there — no dashboard, no
  reconciliation service. If the CLI is all you want, ghstack is a fine choice.
</p>

<h3>git-spice / git-branchless</h3>

<p>
  Adjacent projects that overlap on primitives but solve slightly different problems
  (branch-tracking ergonomics; advanced rebase). Worth knowing about; not direct replacements.
</p>

<h3>GitButler</h3>

<p>
  Different abstraction entirely. GitButler does <em>virtual branches</em> — multiple unrelated
  changes simultaneously in one working copy, "publish" them as separate PRs whenever. <span
  class="font-mono">giff</span> does <em>real branches in a stack</em> — one logical change broken
  into reviewable layers. Different problems. Use GitButler if "I'm juggling three independent
  things at once" is your pain; use <span class="font-mono">giff stack</span> if "I have one big feature
  to break up" is.
</p>

<h3>Mergify</h3>

<p>
  PR automation engine — handles auto-merge, merge queues, branch protection rules, etc. Doesn't
  manage stacks. The <span class="font-mono">giff stack</span> runner overlaps with Mergify's auto-merge
  feature for the specific case of "merge the bottom of a stack." For anything more complex,
  Mergify is the better tool.
</p>

<h2>What we believe</h2>

<ul>
  <li>
    <strong>Real branches over virtual ones.</strong> Every frame is a branch you can see in
    <code>git branch</code>. No magic working-copy semantics. Your stack survives <code>giff</code>
    being uninstalled — you just lose the metadata file, not the work.
  </li>
  <li>
    <strong>One commit per frame.</strong> Frames represent logical layers, not commit history. A
    frame with three WIP commits is a frame that hasn't been polished yet; rebases get worse with
    every fixup commit. The pre-commit hook makes the rule machine-enforced rather than
    aspirational.
  </li>
  <li>
    <strong>GitHub is the source of truth.</strong> Local <code>.git/stacked.toml</code> is a cache.
    The runner's SQLite is a cache. The web dashboard reads from GitHub directly. Anything that
    holds local state holds it temporarily and reconciles on demand. Lose the cache, sync, get the
    cache back.
  </li>
  <li>
    <strong>Self-hostable means actually self-hostable.</strong> The runner runs in any Docker host
    you control. Token and metadata never leave your infrastructure. If the project disappears
    tomorrow your install keeps working.
  </li>
  <li>
    <strong>Open source by default.</strong> The CLI, the web dashboard, and the runner are all in
    the same repo, all under the same license. There's no "open core" — the parts you want, you can
    have, run, and modify.
  </li>
  <li>
    <strong>Honest about limits.</strong> See the <a href="/docs/limitations">limitations page</a>.
  </li>
</ul>

<h2>The path</h2>

<p>The roadmap, in priority order:</p>

<ol>
  <li><strong>Phase 1 (today):</strong> CLI + dashboard + self-hosted runner. Single-tenant. Fully open source. Goal: useful for one team.</li>
  <li><strong>Phase 2 (later):</strong> Hosted version of the same runner — single-tenant per customer, paid via subscription. The bar to start: do people deploy Phase 1? Are they emailing about issues?</li>
  <li><strong>Phase 3 (much later):</strong> Multi-tenant SaaS, real auth, billing, org model. Only if Phase 2 validates demand.</li>
</ol>

<Note>
  We deliberately don't have any of this yet. The runner is single-tenant. There's no auth on the
  HTTP API. There's no billing code. We're starting at "is the tool useful at all" and working
  outward from there.
</Note>
