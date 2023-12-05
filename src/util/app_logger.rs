use std::env;

use tracing_appender::non_blocking::WorkerGuard;

pub struct AppLogger {}

impl AppLogger {
    pub fn init_logger(file_name: &str) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
        let current_dir = env::current_dir()
            .expect("Failed to get current dir")
            .display()
            .to_string();

        let file_appender = tracing_appender::rolling::hourly(current_dir, file_name);
        let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt().with_writer(file_writer).init();

        Ok(guard)
    }
}
