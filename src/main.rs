use cmd::AppCmd;
use tracing::info;

use crate::util::AppLogger;

mod appclient;
mod cmd;
mod constants;
mod util;

const LOG_FILE_PREFIX: &str = "app.log";

#[cmd_lib::main]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = AppLogger::init_logger(LOG_FILE_PREFIX)?;
    info!("Starting app");

    let wget2 = "wget";

    let result = AppCmd::has_package(wget2)?;

    info!("result: {}", result);

    Ok(())
}
