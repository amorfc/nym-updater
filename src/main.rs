use cmd::AppCmd;
use tracing::{error, info};

use crate::util::init_logger;

mod appclient;
mod cmd;
mod constants;
mod util;

const LOG_FILE_DIR: &str = "./logs";
const LOG_FILE_PREFIX: &str = "app.log";

#[cmd_lib::main]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_logger(Some(LOG_FILE_DIR), LOG_FILE_PREFIX)?;

    info!("Starting app");
    drop(_guard);
    let wget2 = "wget";

    // let result = AppCmd::has_package(wget2)?;

    Ok(())
}
