mod github_lists;
mod github_stars;
mod hn_upvotes;
mod pinboard;
mod reddit_saves;

use crate::karakeep;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use karakeep_client::{BookmarkCreate, KarakeepClient};
use std::pin::Pin;

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    fn list_name(&self) -> &'static str;

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>>;

    fn is_activated(&self) -> bool;
    fn recurring_schedule(&self) -> String;

    fn run_immediate(&self) -> bool {
        true
    }

    async fn sync_additional_lists(&self, _client: &KarakeepClient) -> anyhow::Result<()> {
        Ok(())
    }

    async fn sync(&self) -> anyhow::Result<i32> {
        sync_with_client(self, karakeep::get_client()).await
    }
}

async fn sync_with_client<P: Plugin + ?Sized>(
    plugin: &P,
    client: &KarakeepClient,
) -> anyhow::Result<i32> {
    let mut stream = plugin.to_bookmark_stream().await?;
    let list_name = plugin.list_name();

    let mut exists = 0;
    let mut created_count = 0;

    let list_id = client.ensure_list_exists(list_name).await?;

    'chunks: while let Some(chunk) = stream.next().await {
        tracing::info!(
            "processing chunk for list: {} (count={})",
            list_name,
            chunk.len()
        );
        for bookmark in chunk {
            let created = client.upsert_bookmark_to_list(&bookmark, &list_id).await?;
            if created {
                exists = 0;
                created_count += 1;
            } else {
                exists += 1;
            }

            // if we have 5 consecutive existing posts, we can assume we've caught up
            if exists >= 5 {
                tracing::info!("5 consecutive existing posts found, stopping sync");
                break 'chunks;
            }
        }
    }

    plugin.sync_additional_lists(client).await?;

    tracing::info!(
        "sync complete for list: {} (created={})",
        list_name,
        created_count
    );

    Ok(created_count)
}

pub fn get_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(hn_upvotes::HNUpvoted {}),
        Box::new(reddit_saves::RedditSaves {}),
        Box::new(github_stars::GithubStars {}),
        Box::new(pinboard::PinboardBookmarks {}),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    struct ExistingBookmarks {
        additional_synced: AtomicBool,
    }

    #[async_trait]
    impl Plugin for ExistingBookmarks {
        fn list_name(&self) -> &'static str {
            "GitHub Starred"
        }
        fn is_activated(&self) -> bool {
            true
        }
        fn recurring_schedule(&self) -> String {
            "@daily".into()
        }

        async fn to_bookmark_stream(
            &self,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
            let bookmarks = (0..10)
                .map(|_| BookmarkCreate {
                    title: "owner/repo".into(),
                    url: "https://github.com/owner/repo".into(),
                    created_at: None,
                })
                .collect();
            Ok(Box::pin(futures::stream::iter([bookmarks])))
        }

        async fn sync_additional_lists(&self, _client: &KarakeepClient) -> anyhow::Result<()> {
            self.additional_synced.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn existing_bookmark_cutoff_does_not_skip_additional_lists() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/lists"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"lists": [
                    {"id": "all", "name": "GitHub Starred"}
                ]})),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v1/bookmarks/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"bookmarks": [
                    {"id": "existing", "content": {"url": "https://github.com/owner/repo"}}
                ]}),
            ))
            .expect(5)
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/api/v1/lists/all/bookmarks/existing"))
            .respond_with(ResponseTemplate::new(204))
            .expect(5)
            .mount(&server)
            .await;
        let plugin = ExistingBookmarks {
            additional_synced: AtomicBool::new(false),
        };
        assert_eq!(
            sync_with_client(&plugin, &KarakeepClient::new(&server.uri(), "test-token"))
                .await
                .unwrap(),
            0
        );
        assert!(plugin.additional_synced.load(Ordering::SeqCst));
    }
}
