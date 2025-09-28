mod client {
    use reqwest::{Client, Url};

    use crate::settings;

    fn get_client() -> Client {
        let settings = settings::get_settings();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", settings.karakeep.auth))
                .unwrap(),
        );
        Client::builder().default_headers(headers).build().unwrap()
    }

    pub async fn create_bookmark(title: &str, url: &str) -> anyhow::Result<String> {
        let settings = settings::get_settings();
        let client = get_client();

        let api_url = format!("{}/api/v1/bookmarks", settings.karakeep.url);
        let params = serde_json::json!({
            "type": "link",
            "title": title,
            "url": url,
        });

        let resp = client
            .post(&api_url)
            .json(&params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        resp.get("id")
            .and_then(|id| id.as_str())
            .map(|id| id.to_string())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to create bookmark, response did not contain an ID: {:?}",
                    resp
                )
            })
    }

    pub async fn bookmark_exists(url_to_check: &str) -> anyhow::Result<Option<String>> {
        let settings = settings::get_settings();
        let client = get_client();

        let url = format!("{}/api/v1/bookmarks/search", settings.karakeep.url);

        let resp = client
            .get(&url)
            .query(&[
                ("q", url_to_check),
                ("includeContent", "false"),
                ("limit", "1"),
            ])
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let bookmarks = resp.get("bookmarks").and_then(|b| b.as_array()).unwrap();

        if bookmarks.is_empty() {
            return Ok(None);
        }

        let url_to_check: Url = url_to_check.parse().expect("Failed to parse URL to check");
        let bookmark_url: Url = bookmarks[0]
            .get("content")
            .and_then(|c| c.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .parse()
            .expect("Failed to parse bookmark URL");

        if bookmark_url == url_to_check {
            let id = bookmarks[0]
                .get("id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string());
            return Ok(id);
        }

        Ok(None)
    }

    pub async fn ensure_list_exists(list_name: &str) -> anyhow::Result<String> {
        let settings = settings::get_settings();
        let client = get_client();

        let url = format!("{}/api/v1/lists", settings.karakeep.url);

        // First, check if the list already exists
        let resp = client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let lists = resp.get("lists").and_then(|l| l.as_array()).unwrap();

        for list in lists {
            if list.get("name").and_then(|n| n.as_str()) == Some(list_name) {
                if let Some(id) = list.get("id").and_then(|id| id.as_str()) {
                    return Ok(id.to_string());
                }
            }
        }

        // If not, create it
        let params = serde_json::json!({
            "name": list_name,
            "description": "Auto-created list from karakeep-sync",
            "icon": "🚀"
        });

        let resp = client
            .post(&url)
            .json(&params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        resp.get("id")
            .and_then(|id| id.as_str())
            .map(|id| id.to_string())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to create list, response did not contain an ID: {:?}",
                    resp
                )
            })
    }

    pub async fn ensure_bookmark_in_list(bookmark_id: &str, list: &str) -> anyhow::Result<()> {
        let settings = settings::get_settings();
        let client = get_client();

        let list_id = ensure_list_exists(list).await?;

        let url = format!(
            "{}/api/v1/lists/{}/bookmarks/{}",
            settings.karakeep.url, list_id, bookmark_id
        );
        let _resp = client.put(&url).send().await?;

        Ok(())
    }
}

pub struct BookmarkCreate {
    pub title: String,
    pub url: String,
}

pub async fn upsert_bookmark_to_list(
    bookmark: &BookmarkCreate,
    list: &str,
) -> anyhow::Result<bool> {
    // Check if bookmark exists by URL
    let exists = client::bookmark_exists(&bookmark.url).await?;
    let to_create = exists.is_none();

    let bookmark_id: String;
    // If it doesn't exist, create it
    if to_create {
        bookmark_id = client::create_bookmark(&bookmark.title, &bookmark.url).await?;
    } else {
        bookmark_id = exists.unwrap();
    }

    // Either way, make sure that the bookmark is in the specified list
    client::ensure_bookmark_in_list(&bookmark_id, list).await?;
    // Return true if created, false if already existed
    Ok(true)
}
