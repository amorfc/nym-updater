use cmd::AppCmd;
use util::NymConfigFileUtil;

use crate::util::AppLogger;

mod appclient;
mod cmd;
mod constants;
mod util;

const LOG_FILE_PREFIX: &str = "app.log";

#[cmd_lib::main]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    AppLogger::init_logger(LOG_FILE_PREFIX)?;

    let maybe_current_config = NymConfigFileUtil::read_config_file();

    let temp_tag_name = "0.11.0".to_string();

    let wget2 = "wget";

    let result = AppCmd::has_package(wget2)?;

    Ok(())
}
