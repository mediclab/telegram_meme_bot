use crate::app::utils::Period;
use crate::bot::statistics::Statistics;
use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {}
    }

    pub async fn handle(&self) -> Result<JobScheduler> {
        let mut scheduler = JobScheduler::new().await?;

        scheduler
            .add(Job::new_async("00 21 10 * * Fri", |_uuid, _l| {
                Box::pin(async move {
                    let stats = Statistics::new();
                    stats.send(&Period::Week).await;
                })
            })?)
            .await?;

        scheduler
            .add(Job::new_async("00 05 17 * * *", |_uuid, _l| {
                Box::pin(async move {
                    let stats = Statistics::new();
                    stats.send(&Period::Month).await;
                })
            })?)
            .await?;

        scheduler
            .add(Job::new_async("00 05 18 * * *", |_uuid, _l| {
                Box::pin(async move {
                    let stats = Statistics::new();
                    stats.send(&Period::Year).await;
                })
            })?)
            .await?;

        scheduler.shutdown_on_ctrl_c();

        scheduler.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("Scheduler shutdown...");
            })
        }));

        scheduler.start().await?;

        Ok(scheduler)
    }
}
