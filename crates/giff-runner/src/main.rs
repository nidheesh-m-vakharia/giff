use anyhow::{Context, Result};
use clap::Parser;
use giff_runner::{api, config::Config, db::Db, retry, worker};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(name = "giff-runner", about = "Stacked-diffs runner: webhooks + polling + auto-merge")]
struct Cli {
    /// Path to the TOML config file. Falls back to GIFF_RUNNER_CONFIG env var.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config_path = cli
        .config
        .or_else(|| std::env::var("GIFF_RUNNER_CONFIG").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("/etc/giff-runner/config.toml"));

    let token = std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .context("GITHUB_TOKEN env var is required")?;
    let cfg = Arc::new(Config::load_from(&config_path)?.with_token(token));
    tracing::info!(
        config = %config_path.display(),
        listen = %cfg.listen,
        repos = cfg.repos.len(),
        poll_seconds = cfg.poll_seconds,
        "config loaded"
    );

    let db = Arc::new(Db::open(&cfg.db_path()).context("opening sqlite")?);
    tracing::info!(path = %cfg.db_path().display(), "db open");

    // Pre-register configured repos so the API's `/repos` endpoint shows them even before
    // the first poll cycle completes.
    for r in &cfg.repos {
        db.upsert_repo(r.slug.clone()).await?;
    }

    let poll_trigger = worker::spawn(db.clone(), cfg.clone());
    let retry_trigger = retry::spawn(db.clone(), cfg.clone());
    let app = api::build_router(db.clone(), cfg.clone(), poll_trigger, retry_trigger)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(cfg.listen).await?;
    tracing::info!(addr = %cfg.listen, "listening");

    let shutdown = async {
        let ctrl_c = async {
            tokio::signal::ctrl_c().await.ok();
        };
        #[cfg(unix)]
        let term = async {
            use tokio::signal::unix::{signal, SignalKind};
            if let Ok(mut s) = signal(SignalKind::terminate()) {
                s.recv().await;
            }
        };
        #[cfg(not(unix))]
        let term = std::future::pending::<()>();
        tokio::select! {
            _ = ctrl_c => {}
            _ = term => {}
        }
        tracing::info!("shutdown signal received");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .context("axum serve failed")?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,giff_runner=info,axum=info,tower_http=info")))
        .with(fmt::layer().compact())
        .init();
}
