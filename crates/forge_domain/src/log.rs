use std::path::PathBuf;

pub fn init_tracing() -> anyhow::Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join("forge");

    let append = tracing_appender::rolling::hourly(dir.clone(), "forge.log");
    let (non_blocking, _) = tracing_appender::non_blocking(append);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
    Ok(dir)
}
