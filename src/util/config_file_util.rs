use std::fs;

use serde::{Deserialize, Serialize};

use crate::prompt::AppSelectOption;

pub struct NymConfigFileUtil {}

const NYM_CONFIG_FILE_NAME: &str = "auto_update_config.json";
impl NymConfigFileUtil {
    pub fn read_config_file() -> Option<NymReleaseConfig> {
        let config_file = match fs::read_to_string(NYM_CONFIG_FILE_NAME) {
            Ok(file) => file,
            Err(e) => {
                println!(
                    "Error while reading file {} with {} error",
                    NYM_CONFIG_FILE_NAME, e
                );
                return None;
            }
        };

        let current_config = serde_json::from_str::<NymReleaseConfig>(&config_file).ok();

        current_config
    }

    pub fn write_config_file(config: &NymReleaseConfig) -> Result<(), String> {
        let config_file = serde_json::to_string(&config).map_err(|e| {
            format!(
                "Error while serializing config file with {} error",
                e.to_string()
            )
        })?;

        fs::write(NYM_CONFIG_FILE_NAME, config_file).map_err(|e| {
            format!(
                "Error while writing file {} with {} error",
                NYM_CONFIG_FILE_NAME, e
            )
        })?;

        Ok(())
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NymAssetUpdateConfig {
    pub name: String,
    pub auto_update: bool,
    pub index: usize,
}

impl From<AppSelectOption> for NymAssetUpdateConfig {
    fn from(option: AppSelectOption) -> Self {
        NymAssetUpdateConfig {
            name: option.name,
            auto_update: option.checked,
            index: option.index,
        }
    }
}

impl From<NymAssetUpdateConfig> for AppSelectOption {
    fn from(config: NymAssetUpdateConfig) -> Self {
        AppSelectOption {
            name: config.name,
            checked: config.auto_update,
            index: config.index,
        }
    }
}
