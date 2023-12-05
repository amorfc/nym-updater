use cmd_lib::run_fun;
use tracing::{error, info};

use crate::{
    appclient::{GithubRelease, NymGithubClient},
    constants::NymReleaseAssets,
    util::{NymConfigFileUtil, NymReleaseConfig},
};

#[derive(Debug)]
pub struct NymUpdater {
    nym_github_client: NymGithubClient,
    latest_github_release: GithubRelease,
    local_release_config: NymReleaseConfig,
}

impl NymUpdater {
    pub async fn init() -> Result<Self, String> {
        let latest_release = NymGithubClient::new().latest_nym_release().await?;
        let current_release = NymConfigFileUtil::read_config_file()?;
        let nym_github_client = NymGithubClient::new();

        Ok(Self {
            latest_github_release: latest_release,
            local_release_config: current_release,
            nym_github_client,
        })
    }

    pub async fn check_asset_state(&self, asset: &NymReleaseAssets) -> Result<AssetState, String> {
        let asset_name = asset.name();
        let state = run_fun!(systemctl show -p ActiveState --value $asset_name).map_err(|e| {
            let err = format!(
                "Error while checking if {} exists with {} error",
                asset_name, e
            );
            error!(err);
            err
        })?;

        let asset_state = match state.as_str() {
            "active" => AssetState::Running,
            "inactive" => AssetState::Stopped,
            _ => {
                error!("Mixnode does not exist on systemd");
                return Err("Mixnode does not exist on systemd".to_string());
            }
        };

        info!("Mixnode exists on sytemd");
        Ok(asset_state)
    }

    pub async fn stop_asset(&self, asset: &NymReleaseAssets) -> Result<(), String> {
        let asset_name = asset.name();
        info!("Stopping {}...", asset_name);
        run_fun!(service $asset_name stop).map_err(|e| {
            let err = format!("Error while stopping {} with {} error", asset_name, e);
            error!(err);
            err
        })?;

        Ok(())
    }

    pub async fn install_latest(&self, asset: &NymReleaseAssets) -> Result<(), String> {
        info!("Installing latest release...");
        let download_url = self.nym_github_client.latest_release_download_url(asset)?;
        let download_res = run_fun!(wget2 -q -O $download_url);
        if download_res.is_err() {
            return Err("Failed to download latest release".to_string());
        }

        Ok(())
    }

    pub async fn start_update(&self) -> Result<NymUpdateResult, String> {
        info!("Starting update...");
        info!("Latest release is {}", self.latest_github_release.tag_name);

        let temp_defined_asset = &NymReleaseAssets::MixNode;
        let asset_state = self.check_asset_state(temp_defined_asset).await?;

        self.install_latest(temp_defined_asset).await?;
        match asset_state {
            AssetState::Running => {
                self.stop_asset(temp_defined_asset).await?;
            }
            AssetState::Stopped => todo!(),
            AssetState::NotAvailable => todo!(),
        }

        Ok(NymUpdateResult::Success)
    }

    pub async fn update_if_needed(&self) -> NymUpdateResult {
        info!("Checking for updates...");
        info!("Latest release is {}", self.latest_github_release.tag_name);
        info!(
            "Current local release is {}",
            self.local_release_config.release_tag
        );

        let is_update_needed =
            self.latest_github_release.tag_name != self.local_release_config.release_tag;

        if !is_update_needed {
            return NymUpdateResult::NotNecessary;
        }

        self.start_update().await.unwrap_or_else(|e| {
            error!("Failed to start update: {}", e);
            return NymUpdateResult::Failure(e);
        });

        NymUpdateResult::Success
    }
}

pub enum AssetState {
    Running,
    Stopped,
    NotAvailable,
}

pub enum NymUpdateResult {
    Success,
    NotNecessary,
    Failure(String),
}
