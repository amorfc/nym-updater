use prompt::{AppSelectOption, NymUpdateAssetSelectPrompt};
use serde::{Deserialize, Serialize};
use util::{NymAssetUpdateConfig, NymConfigFileUtil};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NymReleaseConfig {
    pub release_tag: String,
    pub assets: Vec<NymAssetUpdateConfig>,
}

impl NymReleaseConfig {
    pub fn from_app_select_options(release_tag: String, assets: Vec<AppSelectOption>) -> Self {
        NymReleaseConfig {
            release_tag,
            assets: assets.into_iter().map(|asset| asset.into()).collect(),
        }
    }

    pub fn as_app_select_options(&self) -> Vec<AppSelectOption> {
        self.assets
            .iter()
            .map(|asset| asset.clone().into())
            .collect()
    }
}
