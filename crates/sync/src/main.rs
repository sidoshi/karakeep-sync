use std::sync::Arc;

use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};

use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod karakeep;
mod plugin;
mod settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("tower=off".parse().unwrap())
                .add_directive("hyper=off".parse().unwrap())
                .add_directive("reqwest=off".parse().unwrap())
                .add_directive("karakeep_sync=trace".parse().unwrap())
                .add_directive("karakeep_client=trace".parse().unwrap()),
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut scheduler = JobScheduler::new().await?;

    let plugins = plugin::get_plugins()
        .into_iter()
        .map(Arc::new)
        .collect::<Vec<_>>();

    for plugin in plugins {
        let list_name = plugin.list_name();

        if !plugin.is_activated() {
            tracing::info!("plugin for list '{}' is not activated, skipping", list_name);
            continue;
        }

        if plugin.run_immediate() {
            let plugin = plugin.clone();
            let job =
                Job::new_one_shot_async(std::time::Duration::from_millis(10), move |_uuid, _l| {
                    tracing::info!("starting immediate sync job for list: {}", list_name);
                    let p = plugin.clone();
                    Box::pin(async move {
                        let _ = p.sync().await;
                    })
                })?;
            scheduler.add(job).await?;
        }

        tracing::info!(
            "scheduling recurring job for list: {}, period: {}",
            list_name,
            plugin.recurring_schedule()
        );

        let schedule = plugin.recurring_schedule().to_string();
        let job = Job::new_async(&schedule, move |_uuid, _l| {
            tracing::info!("starting HN sync daily job");
            let p = plugin.clone();
            Box::pin(async move {
                let _ = p.sync().await;
            })
        })?;
        scheduler.add(job).await?;
    }

    scheduler.start().await?;
    signal::ctrl_c().await?;
    scheduler.shutdown().await?;

    Ok(())
}
