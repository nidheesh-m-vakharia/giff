<script lang="ts">
  import Note from '$lib/components/Note.svelte';
  import { REPO_URL } from '$lib/utils';
</script>

<svelte:head>
  <title>Limitations · giff</title>
</svelte:head>

<h1>Limitations</h1>

<p class="lead text-muted-foreground">
  Things we haven't built, things that don't work yet, and reasons you might pick a different
  tool. The goal here is to save you time — if a deal-breaker is on this page, walk away now
  rather than two weeks in.
</p>

<h2>Who shouldn't use this</h2>

<h3>You don't use GitHub.</h3>
<p>
  <span class="font-mono">giff</span> is GitHub-specific. GitHub Enterprise works (set the
  <code>base_url</code> in config), but GitLab, Bitbucket, Gitea, and friends don't. The forge layer is
  abstracted (<code>ForgeBackend</code> trait) but only one impl exists. Adding GitLab is doable —
  see the <a href="/docs/contributing">contributing page</a>.
</p>

<h3>You don't already work in stacked PRs.</h3>
<p>
  This is a workflow change, not a tool change. <span class="font-mono">giff</span> makes stacked
  diffs <em>easier</em>, but if your team's review process assumes one big PR per feature, dropping
  in a stack tool won't change that. The first cultural conversation is "we're going to start doing
  smaller PRs" — the tool is downstream of that decision.
</p>

<h3>You want a polished, shrink-wrapped product.</h3>
<p>
  <span class="font-mono">giff</span> is alpha software. Bugs exist. Edge cases will surprise you.
  No prebuilt binaries (yet). No published Docker image (yet). If you want the polished version,
  use Graphite — they have ~50 people doing it full-time. We don't.
</p>

<h3>You want jujutsu / Sapling / virtual-branch semantics.</h3>
<p>
  Different abstraction. <span class="font-mono">giff</span> uses real git branches, real merge
  commits, real PRs. If your mental model is "I work on multiple things in one tree and pick what
  to publish," GitButler or jj will fit you better.
</p>

<h2>What's not built</h2>

<p>The full inventory lives in the repo's <code>GAPS.md</code>. Here's the short version, by component.</p>

<h3>The runner</h3>

<ul>
  <li><strong>No auth on the HTTP API.</strong> The <code>/repos</code> / <code>/stacks</code> / <code>/events</code> / <code>/retry-queue</code> endpoints are unauthenticated. Behind a private tunnel (Cloudflare Access, Tailscale ACL, basic-auth reverse proxy) this is fine; on a public URL it's not. Don't expose the runner publicly without auth in front of it.</li>
  <li><strong>Approving-review enforcement is delegated to GitHub branch protection.</strong> If you turn on auto-merge but haven't required reviews on the trunk, the runner will land unreviewed code. Configure branch protection first.</li>
  <li><strong>PAT-based auth, not GitHub Apps.</strong> A long-lived <code>repo</code>-scope token in the runner's env has a high blast radius if compromised. GitHub Apps would be safer; not built.</li>
  <li><strong>Pagination caps at 100 open PRs per repo.</strong> If you have a busier repo, the runner sees only the first page. Fine for most teams; not fine for monorepos.</li>
  <li><strong>No multi-tenant isolation.</strong> One <code>GITHUB_TOKEN</code>, one set of repos. Two teams sharing a runner can read each other's <code>/events</code>.</li>
</ul>

<h3>The CLI</h3>

<ul>
  <li><strong><code>giff stack reorder</code> is linear-only.</strong> Tree-shaped stacks have to be flattened (via <code>squash</code> or <code>drop</code>) before reordering.</li>
  <li><strong>Pre-commit hook auto-installs only after you run any <code>giff</code> command in a repo.</strong> A teammate cloning a stack-managed repo won't have the hook until they run <code>giff status</code> (or any other giff command) once.</li>
  <li><strong>No comprehensive E2E tests against real GitHub.</strong> Lower-level wiremock tests cover the components; integration tests are manual.</li>
</ul>

<h3>The web dashboard</h3>

<ul>
  <li><strong>Plain-text comment rendering.</strong> No GFM, no syntax highlighting in code blocks within comments, no link auto-detection. Click through to GitHub for properly-rendered comments.</li>
  <li><strong>Read-only.</strong> By design — no commenting, approving, or merging from the dashboard. If you want to act, click through to github.com.</li>
  <li><strong>Dark mode is half-built.</strong> CSS variables are in place but there's no toggle.</li>
  <li><strong>Mobile responsive is desktop-first.</strong> Sub-640px screens lay out poorly.</li>
  <li><strong>No tests.</strong> Type checks via <code>svelte-check</code>; no Vitest, no Playwright.</li>
</ul>

<h3>Cross-cutting</h3>

<ul>
  <li><strong>Crate not yet on crates.io.</strong> The CLI package is named <code>giffstack</code> in <code>Cargo.toml</code> and ready to be published, but <code>cargo install giffstack</code> won't resolve until the first <code>cargo publish</code> happens. CI is wired up to auto-publish on push to <code>main</code> once the <code>CARGO_REGISTRY_TOKEN</code> repo secret is set; until then, install from a local checkout (<code>cargo install --path crates/giff-cli</code>).</li>
  <li><strong>No prebuilt standalone binaries or Docker image.</strong> CLI is installable via <code>cargo install giffstack</code> after first publish; the runner still installs via <code>docker compose build</code>. Plan to publish prebuilt binaries and a Docker image once the project stabilises.</li>
  <li><strong>No structured tracing or error reporting.</strong> Logs + the runner's <code>events</code> table is all the observability there is.</li>
</ul>

<Note kind="warn">
  This section is the truth as of today. The full audit is at <a href={`${REPO_URL}/blob/main/GAPS.md`} target="_blank" rel="noopener"><code>GAPS.md</code></a>.
  Open an issue if you hit something that isn't on either list.
</Note>

<h2>Things that might bite you</h2>

<h3>Force-push side effects</h3>
<p>
  <code>giff push</code> uses <code>--force-with-lease</code>. Safe for solo work; if a teammate
  pushes commits to one of your stack branches between your <code>giff push</code> calls, the lease
  will fail and you'll have to fetch + sort it out manually. Don't share branch ownership inside a
  stack with other devs.
</p>

<h3>The runner's auto-merge race</h3>
<p>
  When the runner auto-merges the bottom of a stack, GitHub typically deletes the merged branch
  immediately after. If a child PR's webhook hasn't yet caused a base retarget, that child PR's
  base briefly points at a deleted ref. The retry queue cleans it up on the next attempt — but
  for ~30 seconds the child PR may show as broken on github.com.
</p>

<h3>SSH passphrase prompts</h3>
<p>
  We set <code>GIT_SSH_COMMAND</code> with <code>ControlMaster=auto</code> at the top of every CLI
  command, so within one <code>giff push</code> or <code>giff sync</code> you should be prompted at
  most once. <em>Across</em> commands the multiplexer expires after 2 minutes of inactivity, so
  doing <code>giff publish</code> + <code>giff push</code> + <code>giff sync</code> in slow
  succession could re-prompt. Run <code>ssh-add</code> once at the start of your session for the
  cleanest experience.
</p>

<h3>State drift on shared repos</h3>
<p>
  <code>.git/stacked.toml</code> is per-developer and per-machine. Two devs working on the same stack
  via two clones will each have their own view. They re-converge on <code>giff sync</code> (which
  reads PR descriptions on GitHub to rebuild the truth), but in the meantime they'll disagree.
  Don't try to make it the source of truth for shared state; that's GitHub's job.
</p>
