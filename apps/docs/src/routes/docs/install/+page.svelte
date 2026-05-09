<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import Note from '$lib/components/Note.svelte';
</script>

<svelte:head>
  <title>Installation · giff</title>
</svelte:head>

<h1>Installation</h1>

<p>
  <span class="font-mono">giff</span> is three independent pieces. Most people only need the CLI.
  Add the web dashboard and the runner if you want them.
</p>

<h2>Prerequisites</h2>

<ul>
  <li><strong>Git</strong> — any reasonably modern version (≥ 2.30 is fine)</li>
  <li><strong>Rust toolchain</strong> — only required for installing the CLI from source. Get it from <a href="https://rustup.rs" target="_blank" rel="noopener">rustup.rs</a>. Rust 1.75 or newer.</li>
  <li><strong>Node.js</strong> — only for running the web dashboard locally. v18+.</li>
  <li><strong>Docker + Docker Compose</strong> — only for the optional runner.</li>
  <li><strong>A GitHub personal access token</strong> with the <code>repo</code> scope.</li>
</ul>

<h2>1. The CLI</h2>

<p>From crates.io (recommended):</p>

<CodeBlock lang="bash">cargo install giffstack</CodeBlock>

<p>
  The crate is named <code>giffstack</code>; the binary it installs is called <code>giff</code>
  (same convention as <code>ripgrep</code> shipping <code>rg</code>). To upgrade later, re-run
  with <code>--force</code>:
</p>

<CodeBlock lang="bash">cargo install giffstack --force</CodeBlock>

<p>Or build from a local checkout (good for hacking on it):</p>

<CodeBlock lang="bash">{`git clone https://github.com/nidheesh-m-vakharia/giff.git
cd giff
cargo install --path crates/giff-cli`}</CodeBlock>

<p>Verify:</p>

<CodeBlock lang="bash">{`giff --version
giff --help`}</CodeBlock>

<h3>First-time setup</h3>

<CodeBlock lang="bash">{`giff init                           # writes ~/.config/giff/config.toml skeleton
export GITHUB_TOKEN=ghp_xxxxxxxxx   # or edit the config and paste it under [github]`}</CodeBlock>

<p>The token needs the <code>repo</code> scope. Generate one at <a href="https://github.com/settings/tokens/new?scopes=repo&description=giff" target="_blank" rel="noopener">github.com/settings/tokens/new?scopes=repo</a>. <code>GITHUB_TOKEN</code> from the env always wins over the value in the config file — useful for keeping the token out of files.</p>

<Note>
  The CLI installs a <code>pre-commit</code> hook in any repo where you run <code>giff new</code>. It enforces "one commit per frame" against direct <code>git commit</code> calls. Disable by deleting <code>.git/hooks/pre-commit</code>.
</Note>

<h3>Uninstall</h3>

<CodeBlock lang="bash">cargo uninstall giffstack</CodeBlock>

<p>That removes the binary. Per-repo state in <code>.git/stacked.toml</code> and the pre-commit hook stay until you delete them manually.</p>

<h2>2. The web dashboard (optional)</h2>

<p>A static SvelteKit SPA. Reads from the GitHub API directly with a token in <code>localStorage</code>. No backend.</p>

<CodeBlock lang="bash">{`cd apps/web
npm install
npm run dev      # http://localhost:5173`}</CodeBlock>

<p>For a static deployment:</p>

<CodeBlock lang="bash">{`npm run build    # output in apps/web/build/`}</CodeBlock>

<p>Serve <code>apps/web/build/</code> from any static host (Netlify, Vercel, S3, your own nginx). The first visit sends you to <code>/?settings=1</code>; paste your token and pick a repo from the dropdown (it lists every repo your token can see).</p>

<h2>3. The runner (optional)</h2>

<p>A Rust service that polls GitHub, ingests webhooks, retargets PR bases when stacks merge, and (when configured per-repo) auto-merges approved bottom frames.</p>

<CodeBlock lang="bash">{`mkdir -p config
cp crates/giff-runner/example-config.toml config/runner.toml
# edit config/runner.toml — set [[repos]] slugs, generate webhook_secrets

echo 'GITHUB_TOKEN=ghp_xxxxxxx' > .env
docker compose up -d
curl http://localhost:8080/healthz   # → ok`}</CodeBlock>

<p>To wire up GitHub webhooks (for instant reactions instead of 15-minute polling), see <a href="/docs/concepts#runner">the runner section in Concepts</a> and the <code>crates/giff-runner/README.md</code>.</p>

<h2>Updating</h2>

<CodeBlock lang="bash">{`# CLI (from crates.io)
cargo install giffstack --force

# CLI (from a local checkout)
cd giff && git pull
cargo install --path crates/giff-cli --force

# Web
cd apps/web && npm install && npm run build

# Runner
docker compose pull       # once a published image exists
docker compose build && docker compose up -d`}</CodeBlock>

<Note kind="warn">
  No published Docker images yet. The runner builds from source.
  See <a href="/docs/limitations">Limitations</a>.
</Note>
