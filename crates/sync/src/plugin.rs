mod hn_upvotes;

use crate::karakeep;
use futures::StreamExt;
use karakeep_client::BookmarkCreate;

pub trait Plugin: Clone + Send + Sync {
    const LIST_NAME: &'static str;

    fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<impl futures::Stream<Item = Vec<BookmarkCreate>> + Send>;
    fn is_activated(&self) -> bool;
    fn recurring_schedule(&self) -> &str;

    fn run_immediate(&self) -> bool {
        true
    }

    fn list_name(&self) -> &'static str {
        Self::LIST_NAME
    }

    fn sync(&self) -> impl std::future::Future<Output = anyhow::Result<i32>> + Send {
        async {
            let stream = self.to_bookmark_stream()?;
            tokio::pin!(stream);

            let mut exists = 0;
            let mut created_count = 0;

            let client = karakeep::get_client();
            let list_id = client.ensure_list_exists(self.list_name()).await?;

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
        }
    }
}

pub fn get_plugins() -> Vec<impl Plugin> {
    vec![hn_upvotes::HNUpvoted {}]
}
