<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import Mermaid from '$lib/components/Mermaid.svelte';
  import Note from '$lib/components/Note.svelte';
</script>

<svelte:head>
  <title>Docs · giff</title>
</svelte:head>

<h1>giff documentation</h1>

<p class="lead text-muted-foreground">
  <span class="font-mono">giff</span> is an open-source tool for stacked-diff workflows on GitHub.
  This is the complete reference: concepts, commands, install, contribution, and an honest section
  on what it doesn't do.
</p>

<h2>What you'll find here</h2>

<ul>
  <li><a href="/docs/install">Installation</a> — get the CLI, the web dashboard, and the runner up and running.</li>
  <li><a href="/docs/concepts">Concepts &amp; first principles</a> — frames, stacks, the one-commit-per-frame rule, trees vs linear.</li>
  <li><a href="/docs/commands">Commands</a> — every CLI command with before/after diagrams.</li>
  <li><a href="/docs/why">Why we made this</a> — the motivation and the design choices.</li>
  <li><a href="/docs/limitations">Limitations</a> — who shouldn't use this, and why.</li>
  <li><a href="/docs/contributing">Contributing</a> — the codebase structure, build, and PR process.</li>
</ul>

<h2>Three components</h2>

<p>
  <span class="font-mono">giff</span> has three pieces. They're independently useful — install only
  what you need.
</p>

<Mermaid
  caption="Three components. CLI is mandatory; web and runner are optional adds."
  code={`flowchart TB
  cli["<b>giff CLI</b><br/><span style='color:#86868b;font-size:12px'>Rust · stack management on your machine</span>"]
  web["<b>giff web</b><br/><span style='color:#86868b;font-size:12px'>SvelteKit · read-only dashboard</span>"]
  runner["<b>giff-runner</b><br/><span style='color:#86868b;font-size:12px'>Rust + SQLite · webhooks, polling, auto-merge</span>"]
  github(["<b>GitHub API</b><br/><span style='color:#86868b;font-size:12px'>source of truth</span>"]):::brand
  cli --> github
  web --> github
  runner --> github
  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<h2>The 30-second tour</h2>

<CodeBlock lang="bash">{`# 1. Install (the crate is named giffstack; the binary it ships is giff)
cargo install giffstack

# 2. Configure
giff init
export GITHUB_TOKEN=ghp_xxx   # or paste into ~/.config/giff/config.toml

# 3. Make a stack
git add .
giff publish "feat: scaffold auth"     # frame 1: branch + commit + register
git add .
giff publish "feat: add token signing" # frame 2 stacks on top
giff push                              # opens both PRs

# 4. Inspect
giff log                               # tree view
giff status                            # where am I

# 5. After a review approves the bottom PR
giff sync                              # rebases the rest
giff stack land                        # merges PR #1, retargets PR #2 to main`}</CodeBlock>

<Note>
  Read <a href="/docs/concepts">Concepts &amp; first principles</a> next — the rest of the docs
  assume you know what a "frame" is.
</Note>
