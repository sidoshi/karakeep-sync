use reqwest::{Client, Url};

pub struct KarakeepClient {
    url: String,
    client: Client,
}

pub struct BookmarkCreate {
    pub title: String,
    pub url: String,
}

impl KarakeepClient {
    pub fn new(url: &str, auth_token: &str) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", auth_token)).unwrap(),
        );
        let client = Client::builder().default_headers(headers).build().unwrap();

        Self {
            url: url.into(),
            client,
        }
    }

    pub async fn create_bookmark(&self, title: &str, url: &str) -> anyhow::Result<String> {
        let api_url = format!("{}/api/v1/bookmarks", self.url);
        let params = serde_json::json!({
            "type": "link",
            "title": title,
            "url": url,
        });

        let resp = self
            .client
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

    pub async fn check_exists_bookmark(
        &self,
        bookmark_url: &str,
    ) -> anyhow::Result<Option<String>> {
        let url = format!("{}/api/v1/bookmarks/search", self.url);

        let resp = self
            .client
            .get(&url)
            .query(&[
                ("q", bookmark_url),
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

        let url_to_check = bookmark_url.parse();
        if url_to_check.is_err() {
            return Ok(None);
        }
        let url_to_check: Url = url_to_check.unwrap();
        let bookmark_url = bookmarks[0]
            .get("content")
            .and_then(|c| c.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .parse();
        if bookmark_url.is_err() {
            return Ok(None);
        }
        let bookmark_url: Url = bookmark_url.unwrap();

        if bookmark_url == url_to_check {
            let id = bookmarks[0]
                .get("id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string());
            return Ok(id);
        }

        Ok(None)
    }

    pub async fn ensure_list_exists(&self, list_name: &str) -> anyhow::Result<String> {
        let url = format!("{}/api/v1/lists", self.url);

        // First, check if the list already exists
        let resp = self
            .client
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

        let resp = self
            .client
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

    pub async fn ensure_bookmark_in_list(
        &self,
        bookmark_id: &str,
        list_id: &str,
    ) -> anyhow::Result<()> {
        let url = format!(
            "{}/api/v1/lists/{}/bookmarks/{}",
            self.url, list_id, bookmark_id
        );
        let _resp = self.client.put(&url).send().await?;

        Ok(())
    }

    pub async fn upsert_bookmark_to_list(
        &self,
        bookmark: &BookmarkCreate,
        list_id: &str,
    ) -> anyhow::Result<bool> {
        // Check if bookmark exists by URL
        tracing::debug!("checking if bookmark exists: {}", &bookmark.url);
        let exists = self.check_exists_bookmark(&bookmark.url).await?;
        let to_create = exists.is_none();
        tracing::debug!("bookmark exists: {}", !to_create);

        let bookmark_id: String;
        // If it doesn't exist, create it
        if to_create {
            tracing::info!("creating bookmark: {} - {}", &bookmark.title, &bookmark.url);
            bookmark_id = self.create_bookmark(&bookmark.title, &bookmark.url).await?;
        } else {
            bookmark_id = exists.unwrap();
        }

        tracing::debug!("adding bookmark: {} to list: {}", &bookmark_id, list_id);
        // Either way, make sure that the bookmark is in the specified list
        self.ensure_bookmark_in_list(&bookmark_id, list_id).await?;
        // Return true if created, false if already existed
        Ok(to_create)
    }
}
