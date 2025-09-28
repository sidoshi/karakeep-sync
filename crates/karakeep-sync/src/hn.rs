use crate::karakeep::{self, BookmarkCreate};
use futures::StreamExt;
use hnscraper::stream_pages;

const HN_UPVOTED_LIST: &str = "HN Upvoted";

fn extract_username_from_auth(hn_auth: &str) -> Option<String> {
    hn_auth.split('&').next().map(|s| s.to_string())
}

pub async fn sync_hn_upvoted_posts(hn_auth: &str) -> anyhow::Result<i32> {
    let username = extract_username_from_auth(hn_auth)
        .ok_or_else(|| anyhow::anyhow!("Failed to extract username from auth token"))?;
    let start_path = format!("upvoted?id={}", username);

    let pages = stream_pages(hn_auth, start_path)?;
    tokio::pin!(pages);

    let mut exists = 0;
    let mut created_count = 0;
    while let Some(page) = pages.next().await {
        tracing::info!("processing a page of upvoted posts (count={})", page.len());
        for post in page {
            let bookmark = BookmarkCreate {
                title: post.title.clone(),
                url: post.url.clone(),
            };
            let created = karakeep::upsert_bookmark_to_list(&bookmark, HN_UPVOTED_LIST).await?;
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
