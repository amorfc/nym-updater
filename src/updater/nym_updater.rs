use tracing::info;

use crate::{
    appclient::{GithubRelease, NymGithubClient},
    util::{NymConfigFileUtil, NymReleaseConfig},
};

#[derive(Debug)]
pub struct NymUpdater {
    latest_github_release: GithubRelease,
    local_release_config: NymReleaseConfig,
}

impl NymUpdater {
    pub async fn init() -> Result<Self, String> {
        let nym_github_client = NymGithubClient::new();
        let latest_release = nym_github_client.latest_nym_release().await?;
        let current_release = NymConfigFileUtil::read_config_file()?;

        Ok(Self {
            latest_github_release: latest_release,
            local_release_config: current_release,
        })
    }
    pub async fn start_update() {}

    pub async fn update_if_needed(&self) -> Result<(), String> {
        info!("Checking for updates...");
        info!("Latest release is {}", self.latest_github_release.tag_name);
        info!(
            "Current local release is {}",
            self.local_release_config.release_tag
        );

        let is_update_needed =
            self.latest_github_release.tag_name != self.local_release_config.release_tag;

        if !is_update_needed {
            info!("No update needed");
            return Ok(());
        }

        Ok(())
    }
}
