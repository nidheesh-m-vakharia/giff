//! `giff dashboard` — open the web dashboard in the user's default browser, served
//! by an embedded HTTP server bound to a localhost port.
//!
//! Why an embedded server instead of a bundled native app:
//!   - Single binary, single install (`cargo install giffstack`). No webview deps,
//!     no per-platform installers.
//!   - The exact same SvelteKit build that ships at giffstack.com runs locally —
//!     zero divergence between hosted and local UX.
//!   - Quitting is just Ctrl-C / closing the tab, no separate process model.
//!
//! URL strategy:
//!   - `local.giffstack.com` is a public DNS A record pointing at 127.0.0.1, so the
//!     browser does the lookup, hits loopback, and lands on our local server. That
//!     gives the user a branded URL bar and isolates cookies/localStorage from any
//!     other tools also using plain `localhost`.
//!   - We always print *both* URLs because some networks (Pi-hole, OpenDNS, certain
//!     routers) strip "public DNS records that resolve to private IPs" as a DNS-
//!     rebinding defense. If the branded URL fails, the user clicks the fallback.
//!
//! Token handling:
//!   - Deliberately minimal in v1. The SvelteKit app reads the token from
//!     localStorage as it does on giffstack.com — first run sends the user to
//!     `?settings=1` to paste it. We could read `~/.config/giff/config.toml` and
//!     inject the token via a `/api/token` route, but that bypasses the SPA's
//!     normal auth flow and adds a backend surface to maintain. Skipping for now.

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener as StdListener};
use tiny_http::{Header, Response, Server};

/// Dashboard static assets, baked into the binary at compile time. The directory is
/// populated by `build.rs` (stub) or by CI from `apps/web/build/` (the real SvelteKit
/// static export) before publishing.
static DASHBOARD: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/dashboard-dist");

/// DNS A record points this hostname at 127.0.0.1. The browser resolves it locally,
/// so traffic never leaves the machine.
const PREFERRED_HOSTNAME: &str = "local.giffstack.com";
/// Stable preferred port — easier to spot in netstat / browser history. We fall back
/// to an OS-assigned port if it's already in use.
const PREFERRED_PORT: u16 = 51743;

pub fn run() -> Result<()> {
    let listener = bind_port().context("binding loopback listener")?;
    let port = listener.local_addr()?.port();

    let branded = format!("http://{}:{}", PREFERRED_HOSTNAME, port);
    let local = format!("http://localhost:{}", port);

    println!("giff dashboard listening on:");
    println!(
        "  → {}   (preferred — branded URL via DNS to 127.0.0.1)",
        branded
    );
    println!(
        "  → {}             (fallback if your DNS blocks the above)",
        local
    );
    println!();
    println!("opening browser…");
    println!("press Ctrl-C to stop");

    // Best-effort browser launch on a side thread. Even `that_detached` does a
    // fork+exec synchronously before returning, and on resource-constrained CI
    // runners that's been measured at >2s — long enough for the kernel to queue
    // the test's connection while tiny_http's accept worker hasn't started yet.
    // Spawning the launch lets the main thread reach `Server::from_listener`
    // immediately, so accepts begin the moment the OS accepts the SYN.
    let branded_for_open = branded.clone();
    std::thread::spawn(move || {
        let _ = open::that_detached(&branded_for_open);
    });

    let server =
        Server::from_listener(listener, None).map_err(|e| anyhow::anyhow!("server: {}", e))?;

    // Single-threaded request loop. Dashboard traffic is one user from one browser —
    // a single thread keeps the implementation small and avoids any concurrency
    // surface for static-file serving.
    for request in server.incoming_requests() {
        if let Err(e) = serve(request) {
            eprintln!("giff dashboard: error serving request: {}", e);
        }
    }
    Ok(())
}

/// Try the preferred port first; fall back to a kernel-assigned ephemeral port if it
/// is already in use. Both bind to 127.0.0.1 only — never expose the dashboard to the
/// network.
fn bind_port() -> Result<StdListener> {
    let loopback = IpAddr::V4(Ipv4Addr::LOCALHOST);
    if let Ok(l) = StdListener::bind(SocketAddr::new(loopback, PREFERRED_PORT)) {
        return Ok(l);
    }
    StdListener::bind(SocketAddr::new(loopback, 0)).context("binding ephemeral port")
}

fn serve(request: tiny_http::Request) -> Result<()> {
    // Strip leading slash and query string. tiny_http's `request.url()` returns the
    // raw path-and-query.
    let url = request.url();
    let path = url.split('?').next().unwrap_or(url).trim_start_matches('/');
    // SPA semantics: empty path → index.html; other unknown paths fall through to
    // index.html so SvelteKit's client router can handle them.
    let path = if path.is_empty() { "index.html" } else { path };

    let (bytes, mime, status): (&[u8], String, u16) = match DASHBOARD.get_file(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .essence_str()
                .to_string();
            (file.contents(), mime, 200)
        }
        None => match DASHBOARD.get_file("index.html") {
            // SPA fallback for client-side routes.
            Some(idx) => (idx.contents(), "text/html; charset=utf-8".into(), 200),
            None => (b"404", "text/plain; charset=utf-8".into(), 404),
        },
    };

    let header = Header::from_bytes(&b"Content-Type"[..], mime.as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid Content-Type header"))?;
    let response = Response::from_data(bytes)
        .with_status_code(status)
        .with_header(header);
    request.respond(response).context("sending response")?;
    Ok(())
}
