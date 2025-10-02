use crate::settings;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use hnscraper::stream_pages;
use karakeep_client::BookmarkCreate;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct HNUpvoted {}

fn extract_username_from_auth(hn_auth: &str) -> Option<String> {
    hn_auth.split('&').next().map(|s| s.to_string())
}

#[async_trait]
impl super::Plugin for HNUpvoted {
    fn list_name(&self) -> &'static str {
        "HN Upvoted"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let settings = settings::get_settings();
        let auth = &settings
            .hn
            .auth
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("HN auth token is not set"))?;

        let username = extract_username_from_auth(&auth)
            .ok_or_else(|| anyhow::anyhow!("Failed to extract username from auth token"))?;
        let start_path = format!("upvoted?id={}", username);

        let stream = stream_pages(auth, start_path)?.map(|page| {
            page.into_iter()
                .map(|post| BookmarkCreate {
                    title: post.title,
                    url: post.url,
                    // HN does not provide timestamp for when the post was upvoted
                    created_at: None,
                })
                .collect::<Vec<_>>()
        });

        Ok(Box::pin(stream))
    }

    fn is_activated(&self) -> bool {
        let settings = &settings::get_settings();
        settings.hn.auth.is_some() && !settings.hn.auth.as_ref().unwrap().is_empty()
    }

    fn recurring_schedule(&self) -> String {
        let settings = &settings::get_settings();
        settings.hn.schedule.clone()
    }
}
