use crate::settings;
use async_trait::async_trait;
use futures::{Stream, stream};
use karakeep_client::BookmarkCreate;
use serde::Deserialize;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct PinboardBookmarks {}

#[derive(Debug, Deserialize)]
struct PinboardPost {
    href: String,
    description: String,
    #[allow(dead_code)]
    extended: String,
    #[allow(dead_code)]
    hash: String,
    #[allow(dead_code)]
    time: String,
    #[allow(dead_code)]
    tags: String,
}

#[async_trait]
impl super::Plugin for PinboardBookmarks {
    fn list_name(&self) -> &'static str {
        "Pinboard"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let settings = &settings::get_settings();
        let token = settings
            .pinboard
            .token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Pinboard token is not set"))?;

        tracing::info!("fetching Pinboard bookmarks");

        let client = reqwest::Client::new();
        let url = format!("https://api.pinboard.in/v1/posts/all?auth_token={token}&format=json");
        let resp = client.get(&url).send().await?;

        if !resp.status().is_success() {
            anyhow::bail!("Failed to fetch Pinboard bookmarks: {}", resp.status());
        }

        let posts = resp.json::<Vec<PinboardPost>>().await?;
        let bookmarks: Vec<BookmarkCreate> = posts
            .into_iter()
            .map(|post| BookmarkCreate {
                url: post.href,
                title: post.description,
                created_at: Some(post.time),
            })
            .collect();

        tracing::info!("fetched {} Pinboard bookmarks", bookmarks.len());

        // Return all bookmarks as a single chunk since Pinboard returns everything at once
        let stream = stream::once(async move { bookmarks });
        Ok(Box::pin(stream))
    }

    fn is_activated(&self) -> bool {
        let settings = &settings::get_settings();
        settings.pinboard.token.is_some() && !settings.pinboard.token.as_ref().unwrap().is_empty()
    }

    fn recurring_schedule(&self) -> String {
        let settings = &settings::get_settings();
        settings.pinboard.schedule.clone()
    }
}
