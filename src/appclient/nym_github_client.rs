use std::str::FromStr;

use chrono::{DateTime, Utc};

use super::{
    GithubClConstructorParams, GithubClient, GithubRelease, GithubReleasesResponse, RestResponse,
};

#[derive(Debug)]
pub struct NymGithubClient {
    client: GithubClient,
}

impl NymGithubClient {
    pub fn new() -> Self {
        let nym_params = GithubClConstructorParams {
            owner: "nymtech".to_string(),
            repo: "nym".to_string(),
            base_url: None,
        };

        NymGithubClient {
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

    pub async fn latest_nym_release(&self) -> Result<Option<GithubRelease>, String> {
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

        Ok(latest_nym_binaries_release)
    }
}
