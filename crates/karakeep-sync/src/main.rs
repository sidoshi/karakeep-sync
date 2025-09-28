use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod hn;
mod karakeep;
mod settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let settings = settings::get_settings();
    let mut scheduler = JobScheduler::new().await?;

    let hn_job = Job::new_async(&settings.hn.schedule, move |_uuid, _l| {
        tracing::info!("starting HN sync job");
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
