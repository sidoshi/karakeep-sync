use futures::Stream;
use karakeep_client::BookmarkCreate;
use std::pin::Pin;

use crate::settings;
use reddit_client::RedditClientRefresher;

#[derive(Debug, Clone)]
pub struct RedditSaves {}

impl super::Plugin for RedditSaves {
    fn list_name(&self) -> &'static str {
        "Reddit Saved"
    }

    fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let settings = &settings::get_settings();
        let client_id = &settings.reddit.client_id;
        let client_secret = &settings.reddit.client_secret;
        let refresh_token = &settings.reddit.refresh_token;

        let _client = RedditClientRefresher::new(
            client_id.into(),
            client_secret.into(),
            refresh_token.into(),
        );

        todo!()
    }

    fn is_activated(&self) -> bool {
        let settings = settings::get_settings();

        !settings.reddit.client_id.is_empty()
            && !settings.reddit.client_secret.is_empty()
            && !settings.reddit.refresh_token.is_empty()
    }

    fn recurring_schedule(&self) -> String {
        let settings = settings::get_settings();
        settings.reddit.schedule.clone()
    }
}
