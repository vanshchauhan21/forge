use forge_domain::Environment;
use tracing_appender::non_blocking::WorkerGuard;

pub fn init_tracing(env: Environment) -> anyhow::Result<WorkerGuard> {
    let append = tracing_appender::rolling::daily(env.log_path().clone(), "forge.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(append);

    tracing_subscriber::fmt()
        .pretty()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("FORGE_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("forge=debug")),
        )
        .with_level(true)
        .with_writer(non_blocking)
        .init();
    Ok(guard)
}
