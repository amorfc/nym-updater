use prompt::NymUpdateAssetSelectPrompt;
use util::{NymConfigFileUtil, NymReleaseConfig};

mod appclient;
mod constants;
mod prompt;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_config = NymConfigFileUtil::read_config_file()?;
    let selected_options = current_config.and_then(|config| Some(config.as_app_select_options()));

    let prompt = NymUpdateAssetSelectPrompt::new(selected_options);
    let prompt_result = prompt.start();

    let temp_tag_name = "0.11.0".to_string();

    if let Some(result) = prompt_result {
        let config = NymReleaseConfig::from_app_select_options(temp_tag_name, result);
        NymConfigFileUtil::write_config_file(&config)?;
    }

    Ok(())
}
