mod rotate;

use crate::model::BaseContext;
use anyhow::Result;
use chrono::{Duration, Timelike, Utc};
use log::info;
use tokio::time::{self, Instant as TokioInstant};

fn next_hour() -> TokioInstant {
    let instant = TokioInstant::now();
    let now = Utc::now();

    let hour_from_now = (Utc::now() + Duration::hours(1))
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    let difference = hour_from_now - now;

    instant + difference.to_std().unwrap()
}

pub async fn start(context: BaseContext) -> Result<()> {
    info!("starting jobs loop");

    loop {
        // wait until the next hour
        time::sleep_until(next_hour()).await;

        // run jobs
        let now = Utc::now();
        info!("{} running jobs", now.timestamp());
        tokio::spawn(rotate::execute(context.clone()));
    }
}
