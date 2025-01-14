use clap::ValueEnum;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Default, Debug, Clone, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        let s = s.to_lowercase();
        match s.as_str() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    fn from_env() -> Self {
        let level = std::env::var("FORGE_LOG_LEVEL")
            .or_else(|_| std::env::var("RUST_LOG"))
            .unwrap_or_else(|_| "info".to_string());
        Self::from_str(&level).unwrap_or_default()
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
        }
    }
}

pub fn init_logger() {
    let level = LogLevel::from_env();
    let filter = EnvFilter::from_default_env().add_directive(LevelFilter::from(level).into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time() // Remove timestamp from log output
        .with_target(false) // Remove package name/target from log output
        .init();
}
