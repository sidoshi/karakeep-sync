use anyhow::Context;
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
        let client_id = settings
            .reddit
            .clientid
            .as_ref()
            .context("Reddit client ID is not set")?
            .clone();
        let client_secret = settings
            .reddit
            .clientsecret
            .as_ref()
            .context("Reddit client secret is not set")?
            .clone();
        let refresh_token = settings
            .reddit
            .refreshtoken
            .as_ref()
            .context("Reddit refresh token is not set")?
            .clone();

        let client = RedditClientRefresher::new(client_id, client_secret, refresh_token)
            .refresh()
            .await?;
        let client = Arc::new(client);

        enum StreamState {
            Init,
            Next(Option<String>),
        }

        let stream = futures::stream::unfold(StreamState::Init, move |state| {
            let client = client.clone();

            async move {
                let after = match &state {
                    StreamState::Init => None,
                    StreamState::Next(after) => after.as_ref().cloned(),
                };
                if after.is_none() && matches!(state, StreamState::Next(_)) {
                    return None;
                }

                let resp = client.list_saved(after.as_deref()).await.ok()?;

                let items = resp
                    .posts
                    .into_iter()
                    .map(|post| BookmarkCreate {
                        title: post.title,
                        url: post.url,
                    })
                    .collect::<Vec<_>>();

                tracing::debug!(
                    "fetched {} saved posts from Reddit, after: {:?}",
                    items.len(),
                    resp.after
                );
                Some((items, StreamState::Next(resp.after)))
            }
        });

        Ok(Box::pin(stream))
    }

    fn is_activated(&self) -> bool {
        let settings = settings::get_settings();

        settings.reddit.clientid.is_some()
            && settings.reddit.clientsecret.is_some()
            && settings.reddit.refreshtoken.is_some()
    }

    fn recurring_schedule(&self) -> String {
        let settings = settings::get_settings();
        settings.reddit.schedule.clone()
    }
}
