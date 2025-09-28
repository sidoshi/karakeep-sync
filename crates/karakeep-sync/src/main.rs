use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};

use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod hn;
mod karakeep;
mod settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("tower=off".parse().unwrap())
                .add_directive("hyper=off".parse().unwrap())
                .add_directive("reqwest=off".parse().unwrap())
                .add_directive("karakeep_sync=trace".parse().unwrap()),
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let settings = settings::get_settings();
    let mut scheduler = JobScheduler::new().await?;

    let hn_job_daily = Job::new_async(&settings.hn.schedule, move |_uuid, _l| {
        tracing::info!("starting HN sync daily job");
        let settings = settings::get_settings();
        Box::pin(async move {
            let _ = hn::sync_hn_upvoted_posts(&settings.hn.auth).await;
        })
    })?;

    let hn_job_immediate =
        Job::new_one_shot_async(std::time::Duration::from_millis(10), move |_uuid, _l| {
            tracing::info!("starting HN sync immediate job");
            let settings = settings::get_settings();
            Box::pin(async move {
                let _ = hn::sync_hn_upvoted_posts(&settings.hn.auth).await;
            })
        })?;

    scheduler.add(hn_job_immediate).await?;
    scheduler.add(hn_job_daily).await?;
    scheduler.start().await?;

    signal::ctrl_c().await?;
    scheduler.shutdown().await?;

    Ok(())
}
