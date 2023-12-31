use cmd_lib::run_fun;
use reqwest::Version;
use tracing::info;

use crate::{
    appclient::{GithubRelease, NymGithubClient},
    cmd::AppCmd,
    constants::NymReleaseAssets,
    util::{NymConfigFileUtil, NymReleaseConfig, NymSystemdFileUtil},
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

        info!("Mixnode exists on sytemd with state {:?}", asset_state);
        Ok(asset_state)
    }

    pub async fn stop_asset_service(&self, asset: &NymReleaseAssets) -> Result<(), String> {
        let asset_name = asset.name();
        info!("Stopping {}...", asset_name);
        run_fun!(service $asset_name stop)
            .map_err(|e| format!("Error while stopping {} with {} error", asset_name, e))?;
        Ok(())
    }

    pub async fn install_latest(&self, asset: &NymReleaseAssets) -> Result<String, String> {
        info!("Installing latest release...");
        let download_url = self.nym_github_client.latest_release_download_url(asset)?;
        info!("Downloading latest release from {}", download_url);
        let path_with_latest_tag = self.latest_asset_path(asset).await?;

        run_fun!(wget -O $path_with_latest_tag $download_url)
            .map_err(|e| format!("Error while downloading latest release with {} error", e))?;

        AppCmd::give_ux_permission(&path_with_latest_tag).map_err(|e| {
            format!(
                "Error while chmod {} with {} error",
                path_with_latest_tag, e
            )
        })?;
        Ok(path_with_latest_tag)
    }

    pub async fn systemd_asset_path(&self, asset: &NymReleaseAssets) -> Result<String, String> {
        let asset_name = asset.name();
        match asset {
            NymReleaseAssets::MixNode => {
                let res = run_fun!(systemctl show -p ExecStart --value $asset_name | grep -o "path=[^;]*" | cut -d= -f2).map_err(|e| format!("Error while getting mixnode systemd path with {} error", e))?;
                info!("Mixnode systemd path is {}", res);
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

    pub async fn latest_asset_path(&self, asset: &NymReleaseAssets) -> Result<String, String> {
        let asset_name = asset.name();
        let path_with_latest_tag =
            format!("{}-{}", self.latest_github_release.tag_name, asset_name);

        Ok(path_with_latest_tag)
    }

    pub async fn latest_asset_version(&self, asset: &NymReleaseAssets) -> Result<String, String> {
        info!("Getting latest release version...");
        let asset_name = asset.name();
        let path = self.install_latest(asset).await?;
        let asset_path = "./".to_string() + &path;
        let res = self.asset_build_version(asset, asset_path).await?;

        info!("Latest {} version is {}", asset_name, res);
        Ok(res)
    }

    pub async fn node_id(&self) -> Result<String, String> {
        let id = run_fun!(systemctl cat nym-mixnode | grep id | awk "{print $4}")
            .map_err(|e| format!("Error while getting mixnode id with {} error", e))?;

        Ok(id)
    }

    pub async fn reload_systemd_daemon(&self) -> Result<(), String> {
        info!("Reloading systemd daemon...");
        run_fun!(sudo systemctl daemon-reload)
            .map_err(|e| format!("Error while reloading systemd daemon with {} error", e))?;
        info!("Systemd daemon reloaded");
        Ok(())
    }

    async fn update_systemd_file(
        &self,
        asset: NymReleaseAssets,
        new_exec_path: String,
    ) -> Result<(), String> {
        info!("Updating {} systemd file...", asset.name());

        let systemd_manager = NymSystemdFileUtil::new(asset.clone());

        let bin_path = format!("{}", new_exec_path);
        let version = self.asset_build_version(&asset, bin_path).await?;

        systemd_manager
            .update_exec_start_prop(new_exec_path.clone())
            .await?;

        systemd_manager.update_description_prop(version).await?;

        systemd_manager.systemd_reload()?;
        Ok(())
    }

    pub async fn latest_target_asset_path(
        &self,
        asset: &NymReleaseAssets,
    ) -> Result<String, String> {
        let path_with_latest_tag = self.latest_asset_path(asset).await?;
        let latest_target_asset_path = AppCmd::realt_path(&path_with_latest_tag).map_err(|e| {
            format!(
                "Error while getting real path of latest assets with {} error",
                e
            )
        })?;

        info!(
            "Latest target {} path is {}",
            asset.name(),
            latest_target_asset_path
        );

        Ok(latest_target_asset_path)
    }

    pub async fn start_asset_service(&self, asset: &NymReleaseAssets) -> Result<(), String> {
        let asset_name = asset.name();
        info!("Starting {}...", asset_name);
        run_fun!(systemctl start $asset_name)
            .map_err(|e| format!("Error while restarting {} with {} error", asset_name, e))?;
        let latest_asset_path = self.latest_target_asset_path(asset).await?;

        info!(
            "Successfully {} started release {}",
            asset_name, latest_asset_path
        );
        Ok(())
    }

    pub async fn init_mixnode_node_with_path(&self, path: String) -> Result<(), String> {
        info!("Initing node...");
        let id = self.node_id().await?;

        let ip = run_fun!(curl ipinfo.io | jq -r ".ip")
            .map_err(|e| format!("Error while getting ip address with {} error", e))?;
        info!("Initing node with id {} and ip {}", id, ip);
        let res = run_fun!(sudo $path init --id $id --host $ip)
            .map_err(|e| format!("Error while initing mixnode with {} error", e))?;
        info!("Init result: {}", res);
        Ok(())
    }

    pub async fn start_update(&self) -> Result<NymUpdateResult, String> {
        info!("Starting update...");
        //Be sure that systemd daemon is reloaded to avoid any issues
        self.reload_systemd_daemon().await?;

        let temp_defined_asset = NymReleaseAssets::MixNode;

        let current_asset_state = self.current_asset_state(&temp_defined_asset).await?;
        let current_asset_version = self.current_asset_version(&temp_defined_asset).await?;
        let latest_asset_version = self.latest_asset_version(&temp_defined_asset).await?;
        let latest_target_asset_path = self.latest_target_asset_path(&temp_defined_asset).await?;

        if current_asset_version == latest_asset_version {
            return Ok(NymUpdateResult::NotNecessary);
        }

        match current_asset_state {
            AssetState::Running => {
                self.stop_asset_service(&temp_defined_asset).await?;
            }
            AssetState::Stopped => {
                info!("Mixnode is already stopped");
            }
            AssetState::NotAvailable => {
                return Ok(NymUpdateResult::Failure(
                    "Mixnode does not exist on systemd".to_string(),
                ))
            }
        }

        self.init_mixnode_node_with_path(latest_target_asset_path.clone())
            .await?;
        self.update_systemd_file(temp_defined_asset.clone(), latest_target_asset_path)
            .await?;
        self.start_asset_service(&temp_defined_asset).await?;

        NymConfigFileUtil::update_release_tag(self.latest_github_release.tag_name.clone())
            .map_err(|e| format!("Error while updating release tag with {} error", e))?;

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

#[derive(Debug)]
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
