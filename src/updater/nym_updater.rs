use cmd_lib::run_fun;
use tracing::{error, info};

use crate::{
    appclient::{GithubRelease, NymGithubClient},
    cmd::AppCmd,
    util::{NymConfigFileUtil, NymReleaseConfig},
};

#[derive(Debug)]
pub struct NymUpdater {
    latest_github_release: GithubRelease,
    local_release_config: NymReleaseConfig,
}

impl NymUpdater {
    pub async fn init() -> Result<Self, String> {
        let latest_release = NymGithubClient::new().latest_nym_release().await?;
        let current_release = NymConfigFileUtil::read_config_file()?;

        Ok(Self {
            latest_github_release: latest_release,
            local_release_config: current_release,
        })
    }

    pub async fn is_mixnode_exists(&self) -> Result<bool, String> {
        let is_active_systemd = run_fun!(systemctl show -p ActiveState --value nym-mixnode)
            .map_err(|e| {
                let err = format!("Error while checking if mixnode exists with {} error", e);
                error!(err);
                err
            })?
            .contains("active");

        info!("Mixnode exists on sytemd: {}", is_active_systemd);
        Ok(true)
    }

    pub async fn start_update(&self) -> Result<NymUpdateResult, String> {
        info!("Starting update...");
        info!("Latest release is {}", self.latest_github_release.tag_name);
        self.is_mixnode_exists().await?;

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

        self.start_update()
            .await
            .unwrap_or_else(|e| return NymUpdateResult::Failure(e));

        NymUpdateResult::Success
    }
}

pub enum NymUpdateResult {
    Success,
    NotNecessary,
    Failure(String),
}
