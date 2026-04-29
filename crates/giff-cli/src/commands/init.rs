use crate::config::GlobalConfig;
use anyhow::Result;

pub fn run() -> Result<()> {
    let cfg = GlobalConfig::default();
    cfg.write()?;
    println!("giff initialized. Edit ~/.config/giff/config.toml to add your GitHub token.");
    Ok(())
}
