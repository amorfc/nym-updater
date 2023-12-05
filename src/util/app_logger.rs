use std::io;

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    prelude::*,
};

pub fn init_logger(
    file_dir: Option<&str>,
    file_name: &str,
) -> Result<Option<WorkerGuard>, Box<dyn std::error::Error>> {
    let mut guard = None;

    let file_log = file_dir
        .map(|p| tracing_appender::non_blocking(tracing_appender::rolling::daily(p, file_name)))
        .map(|(none_blocking, g)| {
            guard = Some(g);
            fmt::Layer::new()
                .with_ansi(false)
                .with_writer(none_blocking.with_max_level(Level::INFO))
        })
        .unwrap();

    let console_log = fmt::Layer::new()
        .with_ansi(true)
        .with_writer(io::stderr.with_min_level(Level::INFO).or_else(io::stdout));

    let subscriber = tracing_subscriber::registry()
        .with(console_log)
        .with(file_log);

    subscriber.try_init()?;
    Ok(guard)
}
