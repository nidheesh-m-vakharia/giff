//! Pre-build step for the `giff` binary.
//!
//! Ensures `dashboard-dist/index.html` exists so the `include_dir!` macro in
//! `commands/dashboard.rs` can embed *something* at compile time. CI populates
//! this directory with the real SvelteKit static build before running cargo;
//! everywhere else (fresh clones, `cargo build` without npm) we drop in a stub
//! HTML page that points users at the real build instructions.
//!
//! The directory is gitignored — it gets recreated on every build, and bundled
//! into the published .crate via `package.include` in Cargo.toml.

use std::fs;
use std::path::Path;

const STUB_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>giff dashboard</title>
    <style>
      body { font-family: -apple-system, system-ui, sans-serif; margin: 4rem; color: #222; }
      code { background: #f4f4f4; padding: 0.15em 0.4em; border-radius: 4px; }
    </style>
  </head>
  <body>
    <h1>giff dashboard</h1>
    <p>This binary was built without the SvelteKit dashboard assets.</p>
    <p>If you cloned the repo and ran <code>cargo install --path crates/giff-cli</code>,
       build the dashboard first:</p>
    <pre><code>cd apps/web && npm install && npm run build
cp -r build ../../crates/giff-cli/dashboard-dist
cargo install --path crates/giff-cli --force</code></pre>
    <p>Or install the published crate, which ships a real build:
       <code>cargo install giffstack --force</code>.</p>
  </body>
</html>
"#;

fn main() {
    let dist = Path::new("dashboard-dist");
    if !dist.join("index.html").exists() {
        fs::create_dir_all(dist).expect("creating dashboard-dist/");
        fs::write(dist.join("index.html"), STUB_HTML).expect("writing stub index.html");
    }
    println!("cargo:rerun-if-changed=dashboard-dist");
}
