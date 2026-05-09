<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import Mermaid from '$lib/components/Mermaid.svelte';
  import Note from '$lib/components/Note.svelte';
</script>

<svelte:head>
  <title>Concepts · giff stack</title>
</svelte:head>

<h1>Concepts &amp; first principles</h1>

<p class="lead text-muted-foreground">
  The whole tool is four ideas: <em>frame</em>, <em>stack</em>, <em>trunk</em>, and <em>one commit
  per frame</em>. Everything else falls out of those.
</p>

<h2 id="frame">Frame</h2>

<p>
  A <strong>frame</strong> is a single layer of work. It maps 1:1 to a real git branch. There's no
  virtual-branch magic; if you <code>git branch</code>, you'll see it.
</p>

<p>The rule: <strong>each frame has exactly one commit ahead of its parent.</strong> Why one? Because:</p>

<ul>
  <li>Reviewers see a single conceptual change per frame.</li>
  <li>Rebasing the stack is trivial — replay one commit per frame, not a chain of fixups.</li>
  <li>"Squashing" is a no-op: each frame already <em>is</em> a squashed commit.</li>
</ul>

<p>Enforcement comes from two places:</p>

<ul>
  <li><code>giff commit</code> refuses to add a second commit; suggests <code>giff new</code> or <code>--amend</code>.</li>
  <li>A <code>pre-commit</code> hook (auto-installed) blocks plain <code>git commit</code> from doing the same.</li>
</ul>

<h2 id="stack">Stack</h2>

<p>
  A <strong>stack</strong> is an ordered set of frames where each one is built on top of another.
  In the simplest case it's a chain:
</p>

<Mermaid
  caption="A linear stack: each frame's PR targets the frame below."
  code={`flowchart BT
  main(["main<br/><span style='color:#86868b;font-size:11px'>trunk</span>"])
  base["<b>feat/auth-base</b><br/><span style='color:#86868b;font-size:12px'>PR #42 → main</span>"]
  tokens["<b>feat/auth-tokens</b><br/><span style='color:#86868b;font-size:12px'>PR #43 → feat/auth-base</span>"]
  mw["<b>feat/auth-middleware</b><br/><span style='color:#86868b;font-size:12px'>PR #44 → feat/auth-tokens</span>"]:::brand
  main --> base --> tokens --> mw
  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<p>
  The <em>root frame</em> targets the trunk. Every other frame targets the frame below it. When a
  reviewer looks at PR #43 on GitHub, they see only the diff between <code>feat/auth-base</code>
  and <code>feat/auth-tokens</code> — one layer at a time.
</p>

<h3>Trees</h3>

<p>
  Stacks aren't always linear. A frame can have multiple children — that makes the stack a
  <strong>tree</strong>, not a chain:
</p>

<Mermaid
  caption="A Y-shaped stack: feat/root has two parallel children."
  code={`flowchart BT
  main(["main<br/><span style='color:#86868b;font-size:11px'>trunk</span>"])
  root["<b>feat/root</b><br/><span style='color:#86868b;font-size:12px'>PR #1 → main</span>"]
  left["<b>feat/branch-a</b><br/><span style='color:#86868b;font-size:12px'>PR #2 → feat/root</span>"]
  right["<b>feat/branch-b</b><br/><span style='color:#86868b;font-size:12px'>PR #3 → feat/root</span>"]
  main --> root
  root --> left
  root --> right`}
/>

<p>Operations stay sensible on trees:</p>

<ul>
  <li><code>giff next</code> on a frame with multiple children opens a TUI picker.</li>
  <li><code>giff log</code> renders the tree with <code>├─</code> / <code>└─</code> connectors.</li>
  <li><code>giff stack land</code> requires exactly one root (otherwise "the bottom" is ambiguous).</li>
  <li><code>giff stack reorder</code> is linear-only; trees use <code>drop</code> / <code>squash</code> instead.</li>
</ul>

<h2 id="trunk">Trunk</h2>

<p>
  The <strong>trunk</strong> is the base branch your stacks land into — usually <code>main</code> or
  <code>master</code>. Default is configured in <code>~/.config/giff/config.toml</code>:
</p>

<CodeBlock lang="toml">{`[defaults]
trunk = "main"`}</CodeBlock>

<p>
  Each stack stores its own trunk in <code>.git/stacked.toml</code>, so different stacks in the same
  repo can target different bases (e.g. a release branch).
</p>

<h2 id="lifecycle">Lifecycle</h2>

<p>The full life of a stacked feature looks like this:</p>

<Mermaid
  caption="The state of a stack across its life."
  code={`flowchart LR
  publish["<b>publish</b><br/><span style='color:#86868b;font-size:12px'>create frame +<br/>commit changes</span>"]
  push["<b>push</b><br/><span style='color:#86868b;font-size:12px'>open / update<br/>PRs on GitHub</span>"]
  sync["<b>sync</b><br/><span style='color:#86868b;font-size:12px'>rebase onto<br/>fresh trunk</span>"]
  land["<b>land</b><br/><span style='color:#86868b;font-size:12px'>merge bottom,<br/>promote rest</span>"]:::brand
  publish --> push --> sync --> land
  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<ol>
  <li><strong>publish</strong> — stage changes, run <code>giff publish "msg"</code>. A frame is born with one commit.</li>
  <li><strong>push</strong> — <code>giff push</code> opens or updates a PR for every frame. Each PR's <code>base</code> is the frame below; the bottom targets trunk.</li>
  <li><strong>sync</strong> — somebody else merges to <code>main</code>. <code>giff sync</code> pulls trunk and rebases the whole stack onto it. If a frame was merged via the GitHub web UI, sync detects it and retargets its children.</li>
  <li><strong>land</strong> — once the bottom PR is approved, <code>giff stack land</code> merges it via the GitHub API and retargets the (now-orphaned) child PRs to trunk.</li>
</ol>

<h2 id="metadata">Metadata: where it lives</h2>

<p>Two files, plus PR descriptions on GitHub.</p>

<h3><code>.git/stacked.toml</code> — local, per-repo</h3>

<p>
  Holds the list of stacks, frames, parent pointers, and PR numbers. Lives <em>inside</em>
  <code>.git/</code>, so git ignores it automatically — never committed, never pushed.
</p>

<CodeBlock lang="toml">{`[[stacks]]
id = "a1b2c3..."
name = "auth-refactor"
trunk = "main"

[[stacks.frames]]
id = "f1..."
branch = "feat/auth-base"
pr_number = 42

[[stacks.frames]]
id = "f2..."
branch = "feat/auth-tokens"
parent = "f1..."
pr_number = 43`}</CodeBlock>

<h3>Embedded JSON in PR descriptions</h3>

<p>
  Every PR <code>giff push</code> creates carries a small fenced JSON block at the end of the
  description:
</p>

<CodeBlock>{`Part 2/3 of stack \`auth-refactor\`.

\`\`\`giff
{"stack_id":"a1b2c3","frame_id":"f2","parent_frame_id":"f1","position":2,"total":3}
\`\`\``}</CodeBlock>

<p>
  This is what lets the <strong>web dashboard</strong> and the <strong>runner</strong> reconstruct
  the stack from GitHub alone — no shared local file needed. Don't delete the block manually;
  <code>giff push</code> will rewrite it.
</p>

<h3>Other state files</h3>

<ul>
  <li><code>.git/giff_sync_resume.json</code> — written when <code>giff sync</code> hits a rebase conflict. Holds the list of frames still to rebase. Deleted on <code>giff sync --continue</code> success.</li>
  <li><code>.git/hooks/pre-commit</code> — auto-installed hook enforcing one commit per frame.</li>
  <li><code>~/.config/giff/config.toml</code> — global config (token, base URL, defaults).</li>
</ul>

<h2 id="reconciliation">Reconciliation</h2>

<p>
  PRs sometimes get merged outside <span class="font-mono">giff</span> — say, by clicking the green
  Merge button on github.com. When that happens the local store is briefly stale: it still thinks
  the merged frame exists. <strong>Reconciliation</strong> is the process of detecting these merges
  and updating local state plus the PR bases on GitHub.
</p>

<p>Two reconcilers, both safe to run concurrently:</p>

<ul>
  <li><strong>CLI:</strong> <code>giff sync</code> queries GitHub for each tracked PR's status, prunes merged frames, retargets children's bases on GitHub, and rebases the rest locally.</li>
  <li><strong>Runner</strong> (optional): a service that runs the same logic continuously via webhooks + polling, plus an explicit retry queue when API calls fail.</li>
</ul>

<p>
  Both ultimately call the same kind of API (<code>get_pr</code>, <code>update_pr</code>) and both
  are idempotent. Running them at the same time is safe — the second one's reconcile is a no-op.
</p>

<h2 id="runner">The runner — optional, but useful</h2>

<p>
  When you don't run <code>giff sync</code> for a few hours, your local store drifts from reality.
  That's fine — sync re-converges. But until you sync, no automation happens. The runner closes
  that gap by running the reconciliation continuously, with two signal sources:
</p>

<ul>
  <li><strong>Webhooks</strong> — the primary path. GitHub fires <code>pull_request</code> /
  <code>pull_request_review</code> events at a URL you register; the runner verifies the HMAC
  signature, refreshes the affected PR snapshot, and reconciles.</li>
  <li><strong>Polling</strong> — every 15 minutes by default, as a safety net for missed deliveries.</li>
</ul>

<p>It can also auto-merge the bottom frame of any single-root stack once GitHub reports it mergeable. See <a href="/docs/install#3-the-runner-optional">install</a> for the full setup.</p>

<Note>
  The runner is <em>not required</em>. <code>giff</code> works fully without it — you just have to
  run <code>giff sync</code> yourself when you want reconciliation to happen. The runner is for
  when you want it to happen automatically.
</Note>
