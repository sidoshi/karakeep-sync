mod hn_upvotes;
mod reddit_saves;

use crate::karakeep;
use futures::{Stream, StreamExt};
use karakeep_client::BookmarkCreate;
use std::pin::Pin;

pub trait Plugin: Send + Sync + 'static {
    fn list_name(&self) -> &'static str;

    fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>>;

    fn is_activated(&self) -> bool;
    fn recurring_schedule(&self) -> String;

    fn run_immediate(&self) -> bool {
        true
    }

    fn sync(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = anyhow::Result<i32>> + Send + 'static>> {
        let stream_result = self.to_bookmark_stream();
        let list_name = self.list_name();

        Box::pin(async move {
            let mut stream = stream_result?;

            let mut exists = 0;
            let mut created_count = 0;

            let client = karakeep::get_client();
            let list_id = client.ensure_list_exists(list_name).await?;

            while let Some(chunk) = stream.next().await {
                tracing::info!("processing a page of upvoted posts (count={})", chunk.len());
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
                        return Ok(created_count);
                    }
                }
            }

            Ok(created_count)
        })
    }
}

pub fn get_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(hn_upvotes::HNUpvoted {}),
        Box::new(reddit_saves::RedditSaves {}),
    ]
}
