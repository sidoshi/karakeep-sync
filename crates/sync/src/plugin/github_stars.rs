use crate::settings;
use async_trait::async_trait;
use futures::Stream;
use karakeep_client::BookmarkCreate;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct GithubStars {}

#[async_trait]
impl super::Plugin for GithubStars {
    fn list_name(&self) -> &'static str {
        "GitHub Starred"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        // Implementation for fetching GitHub stars and converting them to BookmarkCreate
        unimplemented!()
    }

    fn is_activated(&self) -> bool {
        let settings = &settings::get_settings();
        settings.github.token.is_some() && !settings.github.token.as_ref().unwrap().is_empty()
    }

    fn recurring_schedule(&self) -> String {
        let settings = &settings::get_settings();
        settings.github.schedule.clone()
    }
}
