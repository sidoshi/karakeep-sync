use crate::settings;
use futures::StreamExt;
use hnscraper::stream_pages;
use karakeep_client::BookmarkCreate;

#[derive(Debug, Clone)]
pub struct HNUpvoted {}

fn extract_username_from_auth(hn_auth: &str) -> Option<String> {
    hn_auth.split('&').next().map(|s| s.to_string())
}

impl super::Plugin for HNUpvoted {
    const LIST_NAME: &'static str = "HN Upvoted";

    fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<impl futures::Stream<Item = Vec<BookmarkCreate>> + Send> {
        let settings = &settings::get_settings();
        let auth = &settings.hn.auth;
        let username = extract_username_from_auth(&auth)
            .ok_or_else(|| anyhow::anyhow!("Failed to extract username from auth token"))?;
        let start_path = format!("upvoted?id={}", username);

        let stream = stream_pages(auth, start_path)?.map(|page| {
            page.into_iter()
                .map(|post| BookmarkCreate {
                    title: post.title,
                    url: post.url,
                })
                .collect::<Vec<_>>()
        });

        Ok(stream)
    }

    fn is_activated(&self) -> bool {
        let settings = &settings::get_settings();
        !settings.hn.auth.is_empty() && !settings.hn.schedule.is_empty()
    }

    fn recurring_schedule(&self) -> &str {
        let settings = &settings::get_settings();
        &settings.hn.schedule
    }
}
