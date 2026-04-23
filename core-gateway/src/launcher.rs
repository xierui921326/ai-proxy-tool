use anyhow::*;
use std::process::Command;

pub fn launch_with_proxy(app_path: &str, port: u16) -> Result<()> {
    let mut cmd = Command::new(app_path);
    let proxy = format!("http://127.0.0.1:{port}");
    cmd.env("HTTPS_PROXY", &proxy)
        .env("HTTP_PROXY", &proxy)
        .env("ALL_PROXY", &proxy)
        .spawn()
        .context("failed to launch target app with proxy")?;
    Ok(())
}

