use std::env;
pub struct AppLogger {}

impl AppLogger {
    pub fn init_logger(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let current_path = env::current_dir().expect("Failed to get current dir");
        let current_dir = current_path
            .to_str()
            .expect("Failed to convert path to str");

        let file_appender = tracing_appender::rolling::hourly(current_dir, file_name);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt().with_writer(non_blocking).init();
        Ok(())
    }
}
