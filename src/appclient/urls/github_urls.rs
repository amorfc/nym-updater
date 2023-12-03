pub enum GithubApiUrl {
    Releases,
    Repos,
}

impl GithubApiUrl {
    pub fn url(&self) -> &'static str {
        match self {
            GithubApiUrl::Releases => "/releases",
            GithubApiUrl::Repos => "/repos",
        }
    }

    pub fn repo_releases(owner: String, repo: String) -> String {
        format!(
            "{}/{}/{}{}",
            Self::Repos.url(),
            owner,
            repo,
            Self::Releases.url()
        )
    }
}
