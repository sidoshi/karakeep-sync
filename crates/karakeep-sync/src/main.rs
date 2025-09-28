use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};

mod hn;
mod karakeep;
mod settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = settings::get_settings();
    let mut scheduler = JobScheduler::new().await?;

    let hn_job = Job::new_async(&settings.hn.schedule, move |_uuid, _l| {
        let settings = settings::get_settings();
        Box::pin(async move {
            let _ = hn::sync_hn_upvoted_posts(&settings.hn.auth).await;
        })
    })?;

    scheduler.add(hn_job).await?;
    scheduler.start().await?;

    signal::ctrl_c().await?;
    scheduler.shutdown().await?;

    Ok(())
}
