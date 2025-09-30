use async_trait::async_trait;
use futures::Stream;
use karakeep_client::BookmarkCreate;
use std::{pin::Pin, sync::Arc};

use crate::settings;
use reddit_client::RedditClientRefresher;

#[derive(Debug, Clone)]
pub struct RedditSaves {}

#[async_trait]
impl super::Plugin for RedditSaves {
    fn list_name(&self) -> &'static str {
        "Reddit Saved"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let settings = settings::get_settings();
        let client_id = settings.reddit.clientid.clone();
        let client_secret = settings.reddit.clientsecret.clone();
        let refresh_token = settings.reddit.refreshtoken.clone();

        let client = RedditClientRefresher::new(client_id, client_secret, refresh_token)
            .refresh()
            .await?;
        let client = Arc::new(client);

        let stream = futures::stream::unfold(None, move |after: Option<String>| {
            let client = client.clone();
            async move {
                let resp = client.list_saved(after.as_deref()).await.ok()?;

                let items = resp
                    .posts
                    .into_iter()
                    .map(|post| BookmarkCreate {
                        title: post.title,
                        url: post.url,
                    })
                    .collect::<Vec<_>>();

                Some((items, resp.after))
            }
        });

        Ok(Box::pin(stream))
    }

    fn is_activated(&self) -> bool {
        let settings = settings::get_settings();

        !settings.reddit.clientid.is_empty()
            && !settings.reddit.clientsecret.is_empty()
            && !settings.reddit.refreshtoken.is_empty()
    }

    fn recurring_schedule(&self) -> String {
        let settings = settings::get_settings();
        settings.reddit.schedule.clone()
    }
}
