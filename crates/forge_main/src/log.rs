use forge_api::Environment;
use tracing::debug;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{self};

#[tracing::instrument(name = "init_logging", skip(env))]
pub fn init_tracing(env: Environment) -> anyhow::Result<WorkerGuard> {
    let log_path = env.log_path();
    debug!(path = %log_path.display(), "Initializing logging system");

    let append = tracing_appender::rolling::daily(log_path, "forge.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(append);

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("FORGE_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("forge=debug")),
        )
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_thread_ids(false)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(non_blocking)
        .init();

    debug!("Logging system initialized successfully");
    Ok(guard)
}
