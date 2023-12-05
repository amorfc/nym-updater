use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::constants::NymReleaseAssets;

use super::{
    GithubClConstructorParams, GithubClient, GithubRelease, GithubReleasesResponse, RestResponse,
};

#[derive(Debug)]
pub struct NymGithubClient {
    owner: String,
    repo: String,
    client: GithubClient,
}

impl NymGithubClient {
    pub fn new() -> Self {
        let repo = "nym".to_string();
        let owner = "nymtech".to_string();

        let nym_params = GithubClConstructorParams {
            owner: owner.clone(),
            repo: repo.clone(),
            base_url: None,
        };

        NymGithubClient {
            owner,
            repo,
            client: GithubClient::new(nym_params),
        }
    }

    async fn latest_nym_release_list(&self) -> Result<GithubReleasesResponse, String> {
        match self.client.latest_repo_release_list().await? {
            RestResponse::Success(res) => Ok(res),
            RestResponse::Error { message } => {
                dbg!(&message);
                Err(format!(
                    "{},{}",
                    message, "Failed to get latest nym release list"
                ))
            }
        }
    }

    pub async fn latest_nym_release(&self) -> Result<GithubRelease, String> {
        let nym_binaries_tag = "nym-binaries";
        let releases_list = self.latest_nym_release_list().await?;
        let latest_nym_binaries_release = releases_list
            .into_iter()
            .filter(|release| release.tag_name.contains(nym_binaries_tag))
            .max_by_key(|release| {
                DateTime::<Utc>::from_str(&release.published_at).unwrap_or_else(|_| {
                    println!("Failed to parse release date: {}", &release.published_at);
                    Utc::now()
                })
            });

        match latest_nym_binaries_release {
            Some(release) => Ok(release),
            None => Err(format!(
                "Failed to find latest nym release with tag: {}",
                nym_binaries_tag
            )),
        }
    }

    pub fn latest_release_download_url(&self, asset: NymReleaseAssets) -> Result<String, String> {
        let download_base_url = format!(
            "https://github.com/{}/{}/releases/latest/download/",
            &self.owner, &self.repo
        );

        match asset {
            NymReleaseAssets::MixNode => Ok(format!("{}{}", download_base_url, asset.name())),
            NymReleaseAssets::Gateway => Err("Gateway is not supported yet".to_string()),
        }
    }
}
