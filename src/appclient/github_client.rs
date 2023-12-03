use super::{base_client::AppClient, GithubApiUrl, GithubReleasesResponse, RestResponse};

pub struct GithubClient {
    owner: String,
    repo: String,
    client: AppClient,
}

impl GithubClient {
    pub fn new(params: GithubClConstructorParams) -> Self {
        let GithubClConstructorParams {
            base_url,
            owner,
            repo,
        } = params;

        let base_url = base_url.unwrap_or("https://api.github.com".to_string());

        GithubClient {
            owner,
            repo,
            client: AppClient::new(base_url),
        }
    }

    pub async fn latest_repo_release_list(
        &self,
    ) -> Result<RestResponse<GithubReleasesResponse>, String> {
        let url = GithubApiUrl::repo_releases(self.owner.clone(), self.repo.clone());
        let res = self.client.get::<GithubReleasesResponse>(&url).await?;
        Ok(res)
    }
}

pub struct GithubClConstructorParams {
    pub owner: String,
    pub repo: String,
    pub base_url: Option<String>,
}
