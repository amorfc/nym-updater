use cmd_lib::run_cmd;

use crate::appclient::NymGithubClient;

mod appclient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let msg = "Hello, world!";

    let latest_release = NymGithubClient::new().latest_nym_release().await?;

    let res = run_cmd!(echo $msg);
    dbg!(&latest_release);

    Ok(())
}
