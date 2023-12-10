use cmd_lib::run_fun;
use regex::Regex;
use tracing::info;

use crate::{
    appclient::{GithubRelease, NymGithubClient},
    cmd::AppCmd,
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

    pub async fn current_asset_state(
        &self,
        asset: &NymReleaseAssets,
    ) -> Result<AssetState, String> {
        let asset_name = asset.name();
        let state = run_fun!(systemctl show -p ActiveState --value $asset_name).map_err(|e| {
            format!(
                "Error while checking if {} exists with {} error",
                asset_name, e
            )
        })?;

        let asset_state = match state.as_str() {
            "active" => AssetState::Running,
            "inactive" => AssetState::Stopped,
            _ => {
                return Err("Mixnode does not exist on systemd".to_string());
            }
        };

        info!("Mixnode exists on sytemd");
        Ok(asset_state)
    }

    pub async fn stop_asset_service(&self, asset: &NymReleaseAssets) -> Result<(), String> {
        let asset_name = asset.name();
        info!("Stopping {}...", asset_name);
        run_fun!(service $asset_name stop)
            .map_err(|e| format!("Error while stopping {} with {} error", asset_name, e))?;
        Ok(())
    }

    pub fn install_latest(&self, asset: &NymReleaseAssets) -> Result<(), String> {
        info!("Installing latest release...");
        let download_url = self.nym_github_client.latest_release_download_url(asset)?;
        info!("Downloading latest release from {}", download_url);
        let path = asset.name();

        run_fun!(wget2 -O $path $download_url)
            .map_err(|e| format!("Error while downloading latest release with {} error", e))?;

        AppCmd::give_ux_permission(path)
            .map_err(|e| format!("Error while chmod {} with {} error", asset.name(), e))?;
        Ok(())
    }

    pub async fn systemd_asset_path(&self, asset: &NymReleaseAssets) -> Result<String, String> {
        let asset_name = asset.name();
        match asset {
            NymReleaseAssets::MixNode => {
                let res = run_fun!(systemctl show -p ExecStart --value $asset_name | grep -o "path=[^;]*" | cut -d= -f2).map_err(|e| format!("Error while getting mixnode systemd path with {} error", e))?;
                Ok(res)
            }
            NymReleaseAssets::Gateway => Err("Gateway not supported yet".to_string()),
        }
    }

    pub async fn asset_build_version(
        &self,
        asset: &NymReleaseAssets,
        bin_path: String,
    ) -> Result<String, String> {
        let res = run_fun!($bin_path --version | grep "Build Version" | awk "{print $3}").map_err(
            |e| {
                format!(
                    "Error while getting {} version with {} error",
                    asset.name(),
                    e
                )
            },
        )?;

        Ok(res)
    }

    pub async fn current_asset_version(&self, asset: &NymReleaseAssets) -> Result<String, String> {
        let asset_name = asset.name();
        let asset_path = self.systemd_asset_path(asset).await?.trim().to_string();
        let res = self.asset_build_version(asset, asset_path).await?;
        info!("Current {} version is {}", asset_name, res);
        Ok(res)
    }

    pub async fn latest_asset_version(&self, asset: &NymReleaseAssets) -> Result<String, String> {
        let asset_name = asset.name();
        let asset_path = "./".to_string() + asset_name;
        self.install_latest(asset)?;
        let res = self.asset_build_version(asset, asset_path).await?;

        info!("Latest {} version is {}", asset_name, res);
        Ok(res)
    }

    async fn update_systemd_file(
        &self,
        asset: &NymReleaseAssets,
        target_exec_path: String,
    ) -> Result<(), String> {
        let asset_name = asset.name();

        let current_systemd_asset_exec_path = self.systemd_asset_path(asset).await?;
        let full_exec_start_line =
            run_fun!(systemctl show -p ExecStart --value $asset_name | grep -o r#"argv\[\]=[^;]*"# | cut -d= -f2)
                .map_err(|e| {
                    format!("Error while getting mixnode systemd path with {} error", e)
                })?;

        let result_str = full_exec_start_line.replace(
            &current_systemd_asset_exec_path,
            //With one space line to separate the path and the args
            format!("{} ", target_exec_path).as_str(),
        );

        let formatted_result = format!("s|^ExecStart=.*|ExecStart={}|", result_str);

        run_fun!(sudo sed -i $formatted_result /etc/systemd/system/nym-mixnode.service)
            .map_err(|e| format!("Error while updating mixnode systemd file with {} error", e))?;

        run_fun!(sudo systemctl daemon-reload)
            .map_err(|e| format!("Error while reloading systemd daemon with {} error", e))?;

        Ok(())
    }

    pub async fn latest_target_asset_path(
        &self,
        asset: &NymReleaseAssets,
    ) -> Result<String, String> {
        let asset_name = asset.name();
        let latest_target_asset_path = AppCmd::realt_path(asset_name).map_err(|e| {
            format!(
                "Error while getting real path of latest assets with {} error",
                e
            )
        })?;

        Ok(latest_target_asset_path)
    }

    pub async fn start_update(&self) -> Result<NymUpdateResult, String> {
        info!("Starting update...");

        let temp_defined_asset = &NymReleaseAssets::MixNode;

        let current_asset_state = self.current_asset_state(temp_defined_asset).await?;
        let current_asset_version = self.current_asset_version(temp_defined_asset).await?;
        let latest_asset_version = self.latest_asset_version(temp_defined_asset).await?;

        let latest_target_asset_path = self.latest_target_asset_path(temp_defined_asset).await?;

        self.update_systemd_file(temp_defined_asset, latest_target_asset_path)
            .await?;

        if current_asset_version == latest_asset_version {
            return Ok(NymUpdateResult::NotNecessary);
        }

        match current_asset_state {
            AssetState::Running => {
                self.stop_asset_service(temp_defined_asset).await?;
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

        match self.start_update().await {
            Ok(res) => res,
            Err(e) => NymUpdateResult::Failure(format!("Failed to start update: {}", e)),
        }
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
