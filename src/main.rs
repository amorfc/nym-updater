use std::time::Duration;

use tokio::{join, spawn, time::sleep};
use tracing::{error, info};

use crate::{updater::NymUpdater, util::init_logger};

mod appclient;
mod cmd;
mod constants;
mod updater;
mod util;

const LOG_FILE_DIR: &str = "./logs";
const LOG_FILE_PREFIX: &str = "app.log";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_logger(Some(LOG_FILE_DIR), LOG_FILE_PREFIX)?;
    info!("Starting app");
    run_update_cron().await;
    info!("Stopping app");
    Ok(())
}

pub async fn run_update_cron() {
    let updater_task = spawn(async move {
        'cron_loop: loop {
            let updater = match NymUpdater::init().await {
                Ok(res) => res,
                Err(e) => {
                    error!("Failed to init updater: {:?}", e);
                    run_sleep_period().await;
                    continue 'cron_loop;
                }
            };

            join!(async {
                match updater.update_if_needed().await {
                    Ok(_) => info!("update successful"),
                    Err(message) => info!("update failed: {}", message),
                }
            });

            run_sleep_period().await;
        }
    });

    if let Err(e) = updater_task.await {
        error!("Updater task failed: {:?}", e);
    }
}

pub async fn run_sleep_period() {
    let period_duration = Duration::from_secs(10);

    sleep(period_duration).await;
}
