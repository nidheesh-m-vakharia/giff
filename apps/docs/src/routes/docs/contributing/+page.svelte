<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import Mermaid from '$lib/components/Mermaid.svelte';
  import Note from '$lib/components/Note.svelte';
  import { REPO_URL } from '$lib/utils';
</script>

<svelte:head>
  <title>Contributing · giff stack</title>
</svelte:head>

<h1>Contributing</h1>

<p class="lead text-muted-foreground">
  PRs welcome. This page is the orientation: where the code lives, how to build and test, what we
  care about in reviews, and what to skip.
</p>

<h2>Repository layout</h2>

<p>
  Cargo workspace at the root, with a <code>apps/</code> directory for the JS apps and a
  <code>docs/</code> directory for design specs. Five Rust crates, two web apps:
</p>

<Mermaid
  caption="Workspace layout — strict layering, low coupling."
  code={`flowchart LR
  root(("<b>giff/</b>"))
  crates["<b>crates/</b>"]
  apps["<b>apps/</b>"]
  docs["<b>docs/superpowers/</b><br/><span style='color:#86868b;font-size:11px'>specs &amp; plans</span>"]
  compose["<b>docker-compose.yml</b><br/><span style='color:#86868b;font-size:11px'>runs the runner</span>"]
  gaps["<b>GAPS.md</b><br/><span style='color:#86868b;font-size:11px'>known gaps</span>"]
  root --> crates
  root --> apps
  root --> docs
  root --> compose
  root --> gaps

  core["<b>giff-core</b><br/><span style='color:#86868b;font-size:11px'>pure Rust · no I/O</span>"]
  git["<b>giff-git</b><br/><span style='color:#86868b;font-size:11px'>GitBackend · shells out to git</span>"]
  forge["<b>giff-github</b><br/><span style='color:#86868b;font-size:11px'>ForgeBackend · ureq, sync</span>"]
  cli["<b>giffstack (CLI)</b><br/><span style='color:#86868b;font-size:11px'>crate at crates/giff-cli/<br/>ships the giff binary</span>"]:::brand
  runner["<b>giff-runner</b><br/><span style='color:#86868b;font-size:11px'>axum + sqlite + worker</span>"]:::brand
  wasm["<b>giff-wasm</b><br/><span style='color:#86868b;font-size:11px'>future browser binding</span>"]
  crates --> core
  crates --> git
  crates --> forge
  crates --> cli
  crates --> runner
  crates --> wasm

  web["<b>web</b><br/><span style='color:#86868b;font-size:11px'>dashboard · adapter-static</span>"]
  site["<b>docs</b><br/><span style='color:#86868b;font-size:11px'>this site · adapter-vercel</span>"]
  apps --> web
  apps --> site

  classDef brand fill:#ff0035,stroke:#ff0035,color:#ffffff;`}
/>

<h2>Architectural rules</h2>

<ul>
  <li>
    <strong><code>giff-core</code> is I/O-free.</strong> No <code>std::fs</code>,
    <code>std::process</code>, network, no <code>tokio</code>. Anything touching the outside world
    lives in <code>giff-git</code>, <code>giff-github</code>, the CLI, or the runner. Keeps the
    core compilable to WASM without a porting layer.
  </li>
  <li>
    <strong><code>giff-github</code> is sync.</strong> Uses <code>ureq</code>, not async. Async-ifies
    only inside the runner, which wraps sync calls in <code>tokio::task::spawn_blocking</code>.
  </li>
  <li>
    <strong>The CLI is single-binary.</strong> Don't introduce daemons, IPC, or background services
    in the CLI — that's the runner's job.
  </li>
  <li>
    <strong>The web dashboard talks only to GitHub directly.</strong> No backend dependency. Static
    deploy.
  </li>
</ul>

<h2>Build &amp; test</h2>

<CodeBlock lang="bash">{`# clone
git clone ${REPO_URL}.git
cd giff

# build everything
cargo build

# test everything (~120 tests as of this writing)
cargo test

# build a single crate
cargo build -p giffstack
cargo test -p giff-runner

# run a single test
cargo test -p giff-core stack_frame_bottom_has_no_parent

# the CLI in dev mode
cargo run -p giffstack -- log --all`}</CodeBlock>

<p>For the JS apps:</p>

<CodeBlock lang="bash">{`cd apps/web        # or apps/docs
npm install
npm run dev        # local dev server
npm run check      # svelte-check
npm run build      # production build`}</CodeBlock>

<h2>What "done" looks like for a PR</h2>

<p>Roughly, in priority order:</p>

<ol>
  <li><strong>Tests for the change.</strong> New behaviour gets a test that would have failed before. Bugfixes get a regression test.</li>
  <li><strong>CI green.</strong> <code>.github/workflows/ci.yml</code> runs Rust build + test, fmt, clippy, web type-check + build, and docs type-check + build on every PR. PRs don't merge with red CI.</li>
  <li><strong>The whole workspace builds locally too.</strong> <code>cargo build</code> + <code>cargo test</code> from the workspace root.</li>
  <li><strong>No new TypeScript / Svelte errors.</strong> <code>npm run check</code> for whichever apps you touched.</li>
  <li><strong>Style matches the surrounding code.</strong> Rust uses <code>rustfmt</code>; JS uses Prettier (run <code>npm run lint</code>).</li>
  <li><strong>One commit per PR.</strong> Yes, even when developing this tool we eat our own dog food. Use <code>giff publish</code> + <code>giff commit --amend</code> to keep the layer clean.</li>
</ol>

<h3>Releases</h3>

<p>
  Pushes to <code>main</code> trigger <code>.github/workflows/release.yml</code>, which auto-publishes
  any of <code>giff-core</code>, <code>giff-git</code>, <code>giff-github</code>, <code>giffstack</code>
  whose <code>Cargo.toml</code> version is newer than the version on crates.io. Pushes without a
  version bump are no-ops. The workflow tags the repo <code>v&lt;giffstack-version&gt;</code> and
  creates a GitHub release when <code>giffstack</code> publishes.
</p>

<p>The publish job is scoped to a GitHub environment named <code>crates-io</code>. Setup:</p>

<ol>
  <li>Repo <strong>Settings → Environments → New environment</strong>, name it <code>crates-io</code>.</li>
  <li>Add a secret <code>CARGO_REGISTRY_TOKEN</code> to that environment (token from <a href="https://crates.io/me" target="_blank" rel="noopener">crates.io/me</a>).</li>
  <li>(Optional but recommended) under "Deployment branches" restrict to <code>main</code>, and tick "Required reviewers" with one or more maintainers.</li>
</ol>

<p>
  With reviewers enabled, every push to <code>main</code> that bumps a version pauses the workflow
  with an "Approve deployment" prompt before the secret is injected and <code>cargo publish</code>
  runs — a manual gate against accidental publishes.
</p>

<p>To cut a release: bump the version in the relevant <code>Cargo.toml</code>(s), commit, push to <code>main</code>, approve the deployment.</p>

<h2>What we look for in reviews</h2>

<ul>
  <li>
    <strong>Validation invariants.</strong> Adding a stack mutation? Add a <code>stack.validate()</code>
    call after the mutation. Adding a new error path on an external call? Hook into the retry queue if
    the operation is retry-safe.
  </li>
  <li>
    <strong>Idempotent operations.</strong> The runner re-tries everything, the CLI re-runs after
    crashes. Operations that hit GitHub need to handle "already in target state" cleanly.
  </li>
  <li>
    <strong>Error messages with a fix in them.</strong> "could not derive a branch name from <code>!!!</code>" is
    OK; "ParseError(InvalidInput)" is not. Tell the user what they can do about it.
  </li>
  <li>
    <strong>Don't leak abstractions across crates.</strong> If <code>giffstack</code> needs something
    from <code>giff-core</code> that isn't there, add it to <code>giff-core</code> with a test. Don't
    reach in and re-implement.
  </li>
</ul>

<h2>What we don't want</h2>

<ul>
  <li>
    <strong>Dependencies-as-features.</strong> Adding <code>tokio</code> to <code>giffstack</code>
    "because it'd be nicer with async" is a no. The CLI staying sync keeps binary size and startup
    time predictable.
  </li>
  <li>
    <strong>Premature multi-tenant work.</strong> The runner is single-tenant by design. Don't add
    <code>tenant_id</code> columns or auth scaffolding until Phase 2 lights up.
  </li>
  <li>
    <strong>SaaS-only features in the open-source repo.</strong> If a feature only makes sense
    behind paid SaaS billing, it doesn't belong here.
  </li>
  <li>
    <strong>Refactors without a tied feature.</strong> "Reorganising for clarity" PRs are politely
    declined unless they're paired with new work that benefits from the reorg.
  </li>
</ul>

<h2>Filing issues</h2>

<p>What to include:</p>

<ul>
  <li>The command you ran (full <code>giff ...</code> invocation).</li>
  <li><code>giff --version</code> output.</li>
  <li><code>giff log --all</code> if relevant — shows the stack state.</li>
  <li>What you expected to happen vs what did.</li>
  <li>If it's a runner issue, the relevant lines from <code>docker compose logs</code> and the contents of <code>/data/state.db</code>'s <code>events</code> table around the time of the issue.</li>
</ul>

<Note>
  Issues with concrete reproductions get triaged faster than "doesn't work for me." The smaller and
  more contained the repro, the faster the fix.
</Note>

<h2>Where to start</h2>

<p>Good first PRs:</p>

<ul>
  <li>Pick a P3 from <a href={`${REPO_URL}/blob/main/GAPS.md`} target="_blank" rel="noopener"><code>GAPS.md</code></a> — they're small, scoped, and won't conflict with anything in flight.</li>
  <li>Improve a CLI error message you found unhelpful.</li>
  <li>Add a test for a code path that doesn't have one.</li>
  <li>Fix a typo in these docs.</li>
</ul>

<p>Bigger work — discuss in an issue first so we can sanity-check the direction.</p>
