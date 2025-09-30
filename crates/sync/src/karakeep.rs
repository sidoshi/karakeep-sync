use std::sync::OnceLock;

use karakeep_client::KarakeepClient;

static CLIENT: OnceLock<KarakeepClient> = OnceLock::new();
pub fn get_client() -> &'static KarakeepClient {
    CLIENT.get_or_init(|| {
        let settings = &crate::settings::get_settings().karakeep;
        KarakeepClient::new(&settings.url, &settings.auth)
    })
}

pub struct BookmarkCreate {
    pub title: String,
    pub url: String,
}

pub async fn upsert_bookmark_to_list(
    bookmark: &BookmarkCreate,
    list_id: &str,
) -> anyhow::Result<bool> {
    // Check if bookmark exists by URL
    tracing::debug!("checking if bookmark exists: {}", &bookmark.url);
    let client = get_client();
    let exists = client.check_exists_bookmark(&bookmark.url).await?;
    let to_create = exists.is_none();
    tracing::debug!("bookmark exists: {}", !to_create);

    let bookmark_id: String;
    // If it doesn't exist, create it
    if to_create {
        tracing::info!("creating bookmark: {} - {}", &bookmark.title, &bookmark.url);
        bookmark_id = client
            .create_bookmark(&bookmark.title, &bookmark.url)
            .await?;
    } else {
        bookmark_id = exists.unwrap();
    }

    tracing::debug!("adding bookmark: {} to list: {}", &bookmark_id, list_id);
    // Either way, make sure that the bookmark is in the specified list
    client
        .ensure_bookmark_in_list(&bookmark_id, list_id)
        .await?;
    // Return true if created, false if already existed
    Ok(to_create)
}
