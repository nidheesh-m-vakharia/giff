<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import Mermaid from '$lib/components/Mermaid.svelte';
  import Note from '$lib/components/Note.svelte';
</script>

<svelte:head>
  <title>Commands · giff</title>
</svelte:head>

<h1>Commands</h1>

<p class="lead text-muted-foreground">
  Every CLI command, what it does, and what it does to the stack. Each section shows a before/after
  diagram so you can predict the effect.
</p>

<h2 id="init">giff init</h2>

<p>Creates the global config file at <code>~/.config/giff/config.toml</code> if it doesn't exist. Idempotent.</p>

<CodeBlock lang="bash">giff init</CodeBlock>

<p>The skeleton it writes:</p>

<CodeBlock lang="toml">{`[github]
token = ""
base_url = "https://api.github.com"

[defaults]
trunk = "main"
draft_prs = true
pr_template = ""`}</CodeBlock>

<p>Edit the file or set <code>GITHUB_TOKEN</code> in your shell. Env wins.</p>

<h2 id="new">giff new &lt;branch&gt;</h2>

<p>Creates a new git branch from the current commit, checks it out, and registers it as a frame in the current stack (or starts a new stack if you're on the trunk).</p>

<CodeBlock lang="bash">giff new feat/auth-tokens</CodeBlock>

<Mermaid
  caption="Effect of `giff new feat/auth-tokens` while on `feat/auth-base`."
  code={`flowchart LR
  subgraph BEFORE[ before ]
    direction BT
    main1(["main"])
    base1["<b>feat/auth-base</b><br/><span style='color:#86868b;font-size:11px'>← here</span>"]:::brand
    main1 --> base1
  end
  subgraph AFTER[ after ]
    direction BT
    main2(["main"])
    base2["feat/auth-base"]
    tokens["<b>feat/auth-tokens</b><br/><span style='color:#86868b;font-size:11px'>← here · 0 commits</span>"]:::brand
    main2 --> base2 --> tokens
  end
  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<p>The new frame inherits its parent from whatever you were on. If you were on <code>main</code>, the frame becomes a stack root.</p>

<h2 id="publish">giff publish &lt;message&gt;</h2>

<p>The recommended way to add a frame: creates the branch <em>and</em> commits the staged changes in one step. The message is used both as the commit message and (slugified) as the branch name.</p>

<CodeBlock lang="bash">{`git add .
giff publish "feat: add token signing"
# → branch: feat/token-signing       (conventional prefix becomes a path segment)
# → commit: "feat: add token signing"`}</CodeBlock>

<p>Override the auto-derived branch:</p>

<CodeBlock lang="bash">giff publish "Some long descriptive message" -b feat/short-name</CodeBlock>

<p>Auto-stage all tracked changes (like <code>git commit -a</code>):</p>

<CodeBlock lang="bash">giff publish -a "Quick patch"</CodeBlock>

<Note>
  If nothing is staged (and you didn't pass <code>-a</code>), <code>publish</code> fails <em>before</em>
  creating the branch. You won't end up on a fresh empty branch by accident.
</Note>

<h2 id="commit">giff commit -m "msg"</h2>

<p>The first commit on a frame. Refuses if the frame already has a commit.</p>

<CodeBlock lang="bash">{`giff commit -m "scaffold auth"          # creates the one commit on this frame
giff commit --amend -m "scaffold auth (revised)"   # always allowed — preserves the invariant
giff commit --amend                                  # amend without changing the message
giff commit -a -m "auto-stage tracked changes"`}</CodeBlock>

<p>If you try a second commit, you get:</p>

<CodeBlock>{`error: frame \`feat/auth-base\` already has 1 commit(s) ahead of \`main\` —
       one commit per frame is enforced.
       Options:
         • Start a new frame on top:   giff new <branch-name>
         • Amend the existing commit:  giff commit --amend [-m "<message>"]`}</CodeBlock>

<h2 id="checkout">giff checkout &lt;target&gt;</h2>

<p>Move to a frame by branch name or by 1-based position.</p>

<CodeBlock lang="bash">{`giff checkout feat/auth-tokens   # by name
giff checkout 2                  # by position — only valid in linear stacks`}</CodeBlock>

<p>Position-based checkout refuses on tree-shaped stacks (positions are ambiguous when there are siblings); use names there.</p>

<h2 id="next-prev">giff next / giff prev</h2>

<p>Move one frame up (<code>next</code>) or one frame down (<code>prev</code>) along the current branch's chain.</p>

<CodeBlock lang="bash">{`giff next     # checkout the child frame
giff prev     # checkout the parent frame`}</CodeBlock>

<p>If the current frame has multiple children, <code>giff next</code> opens an interactive picker:</p>

<CodeBlock>{`╭ frame \`feat/root\` has 2 children — pick one ──────────╮
│ ▸ feat/branch-a #43                                    │
│   feat/branch-b #44                                    │
╰────────────────────────────────────────────────────────╯
↑↓ move  Enter checkout  Esc cancel`}</CodeBlock>

<h2 id="status">giff status</h2>

<p>Where am I.</p>

<CodeBlock lang="bash">giff status</CodeBlock>

<CodeBlock>{`branch: feat/auth-tokens
stack:  auth-refactor (3 frames, linear)
path:   main → feat/auth-base → feat/auth-tokens
depth:  1 (children: 0)
PR:     #43`}</CodeBlock>

<h2 id="dashboard">giff dashboard</h2>

<p>Open the web dashboard in your default browser. Starts an embedded HTTP server on a localhost port and serves the SvelteKit app baked into the <code>giff</code> binary.</p>

<CodeBlock lang="bash">giff dashboard</CodeBlock>

<CodeBlock>{`giff dashboard listening on:
  → http://local.giffstack.com:51743   (preferred — branded URL via DNS to 127.0.0.1)
  → http://localhost:51743             (fallback if your DNS blocks the above)
opening browser…
press Ctrl-C to stop`}</CodeBlock>

<p>Same UI as <a href="https://giffstack.com" target="_blank" rel="noopener">giffstack.com</a>, your token in <code>localStorage</code>, no data leaves your machine. Ctrl-C in the terminal when you're done.</p>

<h2 id="log">giff log</h2>

<p>Tree view of the current repo's stacks. By default hides frames whose PR is closed or merged.</p>

<CodeBlock lang="bash">{`giff log              # only frames with open or no PR
giff log --all        # include closed/merged`}</CodeBlock>

<CodeBlock>{`stack: auth-refactor (trunk: main)
● main
│
◉ feat/auth-base       [PR #42 [open]]
│
◉ feat/auth-tokens     [PR #43 [open]]   ← you are here
│
◉ feat/auth-middleware [PR #44 [open]]`}</CodeBlock>

<p>For trees, the connectors fork:</p>

<CodeBlock>{`stack: y-stack (trunk: main)
● main
│
◉ feat/root            [PR #1 [open]]
│
├─ ◉ feat/left         [PR #2 [open]]
│
└─ ◉ feat/right        [PR #3 [open]]`}</CodeBlock>

<h2 id="push">giff push</h2>

<p>Opens or updates a PR for every frame in the current stack.</p>

<CodeBlock lang="bash">giff push</CodeBlock>

<p>What happens, in order:</p>

<ol>
  <li>Validates the stack (no cycles, no orphans, no duplicate branches).</li>
  <li>Pushes <em>every branch</em> to <code>origin</code> in a single <code>git push --force-with-lease</code> with multiple refspecs — one SSH handshake for the whole stack.</li>
  <li>For each frame in topological order, in parallel (8 workers): creates a new PR if there's no <code>pr_number</code> yet, otherwise updates the existing PR's body and base. Each PR's <code>base</code> is its parent's branch (or the trunk for roots).</li>
  <li>Embeds the <code>giff</code> JSON metadata block at the end of every PR description.</li>
  <li>Writes new <code>pr_number</code>s back to <code>.git/stacked.toml</code>.</li>
</ol>

<Note>
  If half the API calls fail, the successful ones are still saved — re-running <code>giff push</code>
  will retry the failures.
</Note>

<h2 id="sync">giff sync</h2>

<p>Pulls trunk, reconciles merged PRs, rebases the whole stack onto fresh trunk.</p>

<CodeBlock lang="bash">giff sync</CodeBlock>

<p>The full flow:</p>

<ol>
  <li><strong>Reconcile.</strong> Asks GitHub for the status of every tracked PR (in parallel). For each merged frame: removes it from the local store and retargets every child PR's base on GitHub (walking past consecutive merged ancestors).</li>
  <li><strong>Pull trunk.</strong> <code>git fetch origin &lt;trunk&gt;</code> + <code>git rebase origin/&lt;trunk&gt; &lt;trunk&gt;</code>.</li>
  <li><strong>Restack.</strong> Each remaining frame is rebased onto its parent in topological order.</li>
</ol>

<Mermaid
  caption="`giff sync` after the bottom PR was merged via GitHub web UI."
  code={`flowchart LR
  subgraph BEFORE[ before sync ]
    direction BT
    m1(["main<br/>(old)"])
    b1["<b>feat/auth-base</b><br/><span style='color:#86868b;font-size:11px'>PR #42 · MERGED on github</span>"]:::merged
    t1["feat/auth-tokens<br/><span style='color:#86868b;font-size:11px'>PR #43 → base</span>"]
    mw1["feat/auth-mw<br/><span style='color:#86868b;font-size:11px'>PR #44 → tokens</span>"]
    m1 --> b1 --> t1 --> mw1
  end
  subgraph AFTER[ after sync ]
    direction BT
    m2(["main<br/><span style='color:#86868b;font-size:11px'>now includes auth-base</span>"])
    t2["feat/auth-tokens<br/><span style='color:#86868b;font-size:11px'>PR #43 → main (retargeted)</span>"]:::brand
    mw2["feat/auth-mw<br/><span style='color:#86868b;font-size:11px'>PR #44 → tokens</span>"]
    m2 --> t2 --> mw2
  end
  classDef merged fill:#e5e5ea,stroke:#86868b,color:#86868b;
  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<h3>Conflicts</h3>

<p>If a frame conflicts during the rebase, sync stops:</p>

<CodeBlock>{`[2/3] Rebasing feat/auth-tokens onto feat/auth-base...
  conflict in feat/auth-tokens

Resolve the conflicts, stage your changes, then run:
  git rebase --continue
  giff sync --continue`}</CodeBlock>

<p>State is saved in <code>.git/giff_sync_resume.json</code>. Resolve normally with git, then resume with:</p>

<CodeBlock lang="bash">giff sync --continue</CodeBlock>

<h2 id="stack-reorder">giff stack reorder</h2>

<p>Interactive TUI for re-arranging frames in a linear stack. <em>Linear stacks only</em> — for trees, restructure with <code>drop</code> / <code>squash</code>.</p>

<CodeBlock lang="bash">giff stack reorder</CodeBlock>

<p>↑↓ to move the highlighted frame, Enter to apply, Esc to cancel. Run <code>giff push</code> afterward to update PRs.</p>

<h2 id="stack-squash">giff stack squash &lt;branch&gt;</h2>

<p>Merges a frame's commit into its parent's commit. Preserves the one-commit-per-frame invariant by <em>amending</em> the parent's commit, not adding on top.</p>

<CodeBlock lang="bash">giff stack squash feat/auth-tokens</CodeBlock>

<Mermaid
  caption="Squash: feat/auth-tokens' changes get folded into feat/auth-base's commit."
  code={`flowchart LR
  subgraph BEFORE[ before ]
    direction BT
    m1(["main"])
    b1["feat/auth-base<br/><span style='color:#86868b;font-size:11px'>1 commit</span>"]
    t1["<b>feat/auth-tokens</b><br/><span style='color:#86868b;font-size:11px'>squashing</span>"]:::brand
    mw1["feat/auth-mw<br/><span style='color:#86868b;font-size:11px'>1 commit</span>"]
    m1 --> b1 --> t1 --> mw1
  end
  subgraph AFTER[ after ]
    direction BT
    m2(["main"])
    b2["<b>feat/auth-base</b><br/><span style='color:#86868b;font-size:11px'>1 commit · contains both diffs</span>"]:::brand
    mw2["feat/auth-mw<br/><span style='color:#86868b;font-size:11px'>re-parented to base</span>"]
    m2 --> b2 --> mw2
  end
  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<p>Refuses if the parent has no commits to squash into, or if the child has no commits to squash. Children of the squashed frame are re-parented to the squash target.</p>

<h2 id="stack-drop">giff stack drop &lt;branch&gt;</h2>

<p>Removes a frame from the stack. Children are re-parented to the dropped frame's parent (so a Y-shape becomes two roots when you drop the root).</p>

<CodeBlock lang="bash">giff stack drop feat/auth-tokens</CodeBlock>

<Mermaid
  caption="Dropping the middle frame of a chain: above is re-parented down."
  code={`flowchart LR
  subgraph BEFORE[ before ]
    direction BT
    m1(["main"])
    b1["feat/auth-base"]
    t1["<b>feat/auth-tokens</b><br/><span style='color:#86868b;font-size:11px'>dropping</span>"]:::brand
    mw1["feat/auth-mw"]
    m1 --> b1 --> t1 --> mw1
  end
  subgraph AFTER[ after ]
    direction BT
    m2(["main"])
    b2["feat/auth-base"]
    mw2["<b>feat/auth-mw</b><br/><span style='color:#86868b;font-size:11px'>re-parented to base</span>"]:::brand
    m2 --> b2 --> mw2
  end
  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<Note>
  The git branch isn't deleted — only the metadata is removed. Clean up the branch yourself with
  <code>git branch -D &lt;branch&gt;</code> when you're sure.
</Note>

<h2 id="stack-land">giff stack land [--method merge|squash|rebase]</h2>

<p>Merges the bottom (root) PR via the GitHub API and promotes the rest of the stack down.</p>

<CodeBlock lang="bash">{`giff stack land                  # default: merge commit
giff stack land --method squash  # squash-merge
giff stack land --method rebase  # rebase-merge`}</CodeBlock>

<p>Steps:</p>

<ol>
  <li>Refuses if the stack has multiple roots (no unique "bottom" to land).</li>
  <li>Calls GitHub's merge API on the root PR with the chosen method.</li>
  <li>Removes the root frame from the local store.</li>
  <li>For every direct child of the landed frame: sets <code>parent = None</code> and updates its PR's <code>base</code> to the trunk on GitHub.</li>
</ol>

<Note>
  Auto-merge gating (approving reviews etc.) is delegated to your repo's branch protection rules.
  If those don't require reviews, this will land unreviewed code.
</Note>

<h2 id="parent-branch">giff parent-branch (internal)</h2>

<p>Hidden subcommand. Prints the parent branch of the current frame. Used internally by the
pre-commit hook. You shouldn't need to run it directly.</p>

<CodeBlock lang="bash">giff parent-branch    # → "main" or whatever the parent's branch is</CodeBlock>
