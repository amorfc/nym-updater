pub enum NymReleaseAssets {
    MixNode,
    Gateway,
}

impl NymReleaseAssets {
    pub fn get_all() -> Vec<NymReleaseAssets> {
        vec![NymReleaseAssets::MixNode, NymReleaseAssets::Gateway]
    }

    pub fn get_all_as_string() -> Vec<String> {
        NymReleaseAssets::get_all()
            .into_iter()
            .map(|asset| asset.name().to_string())
            .collect()
    }

    pub fn name(&self) -> &str {
        match self {
            NymReleaseAssets::MixNode => "nym-mixnode",
            NymReleaseAssets::Gateway => "nym-gateway",
        }
    }
}
