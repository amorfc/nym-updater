use std::fs;

use serde::{Deserialize, Serialize};

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

    pub fn _write_config_file(config: &NymReleaseConfig) -> Result<(), String> {
        let config_file = serde_json::to_string_pretty(&config).map_err(|e| {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NymAssetUpdateConfig {
    pub name: String,
    pub auto_update: bool,
    pub index: usize,
}
