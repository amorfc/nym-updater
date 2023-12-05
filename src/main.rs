use cmd::AppCmd;
use util::NymConfigFileUtil;

mod appclient;
mod cmd;
mod constants;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_current_config = NymConfigFileUtil::read_config_file();

    let temp_tag_name = "0.11.0".to_string();

    let wget2 = "wget2";

    let result = AppCmd::has_package(wget2);

    dbg!(&result);

    Ok(())
}
