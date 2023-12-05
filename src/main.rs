use cmd_lib::run_fun;
use util::NymConfigFileUtil;

mod appclient;
mod constants;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_current_config = NymConfigFileUtil::read_config_file();

    let temp_tag_name = "0.11.0".to_string();

    Ok(())
}
