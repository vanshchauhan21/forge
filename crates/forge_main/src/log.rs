use std::path::PathBuf;

use tracing_subscriber::filter::LevelFilter;

pub fn init_tracing(dir: PathBuf) -> anyhow::Result<PathBuf> {
    let append = tracing_appender::rolling::hourly(dir.clone(), "forge.log");
    let (non_blocking, _) = tracing_appender::non_blocking(append);
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_max_level(LevelFilter::TRACE)
        .with_level(true)
        .with_writer(non_blocking)
        .init();
    Ok(dir)
}
