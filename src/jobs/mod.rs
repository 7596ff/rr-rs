use anyhow::Result;
use chrono::{Duration, Timelike, Utc};
use log::info;
use tokio::time::{self, Instant as TokioInstant};

use crate::model::Context;

mod rotate;

fn next_hour() -> TokioInstant {
    let instant = TokioInstant::now();
    let now = Utc::now();

    let hour_from_now = Utc::now() + Duration::hours(1);
    let hour_from_now = hour_from_now.with_minute(0).unwrap().with_second(0).unwrap();
    let difference = hour_from_now - now;

    instant + difference.to_std().unwrap()
}

pub async fn start(context: Context) -> Result<()> {
    info!("starting jobs loop");

    loop {
        // wait until the next hour
        time::delay_until(next_hour()).await;

        // run jobs
        info!("running jobs");
        tokio::spawn(rotate::execute(context.clone()));
    }
}
