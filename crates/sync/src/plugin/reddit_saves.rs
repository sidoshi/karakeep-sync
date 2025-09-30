use futures::Stream;
use karakeep_client::BookmarkCreate;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct RedditSaves {}

impl super::Plugin for RedditSaves {
    fn list_name(&self) -> &'static str {
        "Reddit Saved"
    }

    fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        todo!()
    }

    fn is_activated(&self) -> bool {
        false
    }

    fn recurring_schedule(&self) -> String {
        "".to_string()
    }
}
