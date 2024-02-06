use crate::app::utils::Period;
use crate::app::Application;
use crate::bot;
use clokwerk::{Interval::Friday, Job, TimeUnits};
use futures::executor::block_on;
use std::sync::Arc;
use std::time::Duration;

pub struct Scheduler {
    timings: SchedulerTimings,
    app: Arc<Application>,
}

pub struct SchedulerTimings {
    week: String,
    month: String,
    year: String,
}

impl Scheduler {
    pub fn new(app: Arc<Application>, week: &str, month: &str, year: &str) -> Self {
        Scheduler {
            timings: SchedulerTimings {
                week: week.to_string(),
                month: month.to_string(),
                year: year.to_string(),
            },
            app: app.clone(),
        }
    }

    pub fn init(&self) -> clokwerk::ScheduleHandle {
        let mut scheduler = clokwerk::Scheduler::with_tz(chrono::Utc);

        scheduler.every(Friday).at(&self.timings.week).once().run({
            let scheduler_app = self.app.clone();
            move || {
                block_on(bot::top::send_top_stats(&scheduler_app, Period::Week))
                    .expect("Can't send meme of week");
            }
        });

        scheduler
            .every(1.day())
            .at(&self.timings.month)
            .once()
            .run({
                let scheduler_app = self.app.clone();
                move || {
                    block_on(bot::top::send_top_stats(&scheduler_app, Period::Month))
                        .expect("Can't send meme of month");
                }
            });

        scheduler.every(1.day()).at(&self.timings.year).once().run({
            let scheduler_app = self.app.clone();
            move || {
                block_on(bot::top::send_top_stats(&scheduler_app, Period::Year))
                    .expect("Can't send meme of year");
            }
        });

        scheduler.watch_thread(Duration::from_millis(100))
    }
}
