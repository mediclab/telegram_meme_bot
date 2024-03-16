use crate::app::utils::Period;
use crate::app::Application;
use crate::bot::statistics::Statistics;
use anyhow::Result;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct Scheduler {
    app: Arc<Application>,
}

impl Scheduler {
    pub fn new(app: Arc<Application>) -> Self {
        Scheduler { app }
    }

    pub async fn handle(&self) -> Result<JobScheduler> {
        let mut scheduler = JobScheduler::new().await?;

        scheduler
            .add(Job::new("00 05 16 * * Fri", {
                let scheduler_app = self.app.clone();
                move |_uuid, _l| {
                    let stats = Statistics::new(scheduler_app.clone());
                    stats.send(&Period::Week);
                }
            })?)
            .await?;

        scheduler
            .add(Job::new("00 05 17 * * *", {
                let scheduler_app = self.app.clone();
                move |_uuid, _l| {
                    let stats = Statistics::new(scheduler_app.clone());
                    stats.send(&Period::Month);
                }
            })?)
            .await?;

        scheduler
            .add(Job::new("00 05 18 * * *", {
                let scheduler_app = self.app.clone();
                move |_uuid, _l| {
                    let stats = Statistics::new(scheduler_app.clone());
                    stats.send(&Period::Year);
                }
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
