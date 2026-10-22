use reqwest::{Client, Response, Url};

const SOURCE_PREFIX: &str = "karakeep-sync-source: ";

fn list_source(list: &serde_json::Value) -> Option<&str> {
    list.get("description")?
        .as_str()?
        .lines()
        .find_map(|line| line.strip_prefix(SOURCE_PREFIX))
}

pub struct KarakeepClient {
    url: String,
    client: Client,
}

pub struct BookmarkCreate {
    pub title: String,
    pub url: String,
    pub created_at: Option<String>,
}

async fn parse_response(resp: Response) -> anyhow::Result<serde_json::Value> {
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(anyhow::anyhow!("Karakeep API returned {status}: {body}",));
    }
    serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Failed to parse Karakeep response: {e}\nBody: {body}"))
}

impl KarakeepClient {
    pub fn new(url: &str, auth_token: &str) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {auth_token}")).unwrap(),
        );
        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            url: url.into(),
            client,
        }
    }

    pub async fn create_bookmark(&self, bookmark: &BookmarkCreate) -> anyhow::Result<String> {
        let api_url = format!("{}/api/v1/bookmarks", self.url);
        let mut params = serde_json::json!({
            "type": "link",
            "title": bookmark.title,
            "url": bookmark.url,
        });

        if bookmark.created_at.is_some() {
            params.as_object_mut().unwrap().insert(
                "createdAt".to_string(),
                bookmark.created_at.clone().unwrap().into(),
            );
        }

        let resp = parse_response(self.client.post(&api_url).json(&params).send().await?).await?;

        resp.get("id")
            .and_then(|id| id.as_str())
            .map(|id| id.to_string())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to create bookmark, response did not contain an ID: {resp:?}"
                )
            })
    }

    pub async fn check_exists_bookmark(
        &self,
        bookmark_url: &str,
    ) -> anyhow::Result<Option<String>> {
        let url = format!("{}/api/v1/bookmarks/search", self.url);

        let resp = parse_response(
            self.client
                .get(&url)
                .query(&[
                    ("q", bookmark_url),
                    ("includeContent", "false"),
                    ("limit", "1"),
                ])
                .send()
                .await?,
        )
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
        self.ensure_list_exists_under(list_name, None, None).await
    }

    pub async fn ensure_tracked_child_list_exists(
        &self,
        list_name: &str,
        parent_id: &str,
        source_key: &str,
    ) -> anyhow::Result<String> {
        self.ensure_list_exists_under(list_name, Some(parent_id), Some(source_key))
            .await
    }

    async fn ensure_list_exists_under(
        &self,
        list_name: &str,
        parent_id: Option<&str>,
        source_key: Option<&str>,
    ) -> anyhow::Result<String> {
        let url = format!("{}/api/v1/lists", self.url);

        let resp = parse_response(self.client.get(&url).send().await?).await?;

        let lists = resp.get("lists").and_then(|l| l.as_array()).unwrap();

        let children: Vec<_> = lists
            .iter()
            .filter(|list| list.get("parentId").and_then(|p| p.as_str()) == parent_id)
            .collect();
        let mut matches: Vec<_> = children
            .iter()
            .copied()
            .filter(|list| source_key.is_some_and(|key| list_source(list) == Some(key)))
            .collect();
        if matches.is_empty() {
            matches = children
                .into_iter()
                .filter(|list| {
                    list.get("name").and_then(|n| n.as_str()) == Some(list_name)
                        && (source_key.is_none() || list_source(list).is_none())
                })
                .collect();
        }
        if source_key.is_some() && matches.len() > 1 {
            anyhow::bail!("Multiple Karakeep lists match source {source_key:?}");
        }
        if let Some(list) = matches.first() {
            let id = list
                .get("id")
                .and_then(|id| id.as_str())
                .ok_or_else(|| anyhow::anyhow!("Karakeep list has no ID"))?;
            let mut changes = serde_json::Map::new();
            if list.get("name").and_then(|n| n.as_str()) != Some(list_name) {
                changes.insert("name".into(), list_name.into());
            }
            if let Some(key) = source_key.filter(|_| list_source(list).is_none()) {
                let description = list
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let separator = if description.is_empty() { "" } else { "\n" };
                changes.insert(
                    "description".into(),
                    format!("{description}{separator}{SOURCE_PREFIX}{key}").into(),
                );
            }
            if !changes.is_empty() {
                parse_response(
                    self.client
                        .patch(format!("{url}/{id}"))
                        .json(&changes)
                        .send()
                        .await?,
                )
                .await?;
            }
            return Ok(id.to_string());
        }

        // If not, create it
        let mut params = serde_json::json!({
            "name": list_name,
            "description": "Auto-created list from karakeep-sync",
            "icon": "🚀"
        });
        if let Some(parent_id) = parent_id {
            params["parentId"] = parent_id.into();
        }
        if let Some(key) = source_key {
            params["description"] =
                format!("Auto-created list from karakeep-sync\n{SOURCE_PREFIX}{key}").into();
        }

        let resp = parse_response(self.client.post(&url).json(&params).send().await?).await?;

        resp.get("id")
            .and_then(|id| id.as_str())
            .map(|id| id.to_string())
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to create list, response did not contain an ID: {resp:?}")
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
        self.client.put(&url).send().await?.error_for_status()?;

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

        // If it doesn't exist, create it
        let bookmark_id: String = if to_create {
            tracing::info!("creating bookmark: {} - {}", &bookmark.title, &bookmark.url);
            self.create_bookmark(bookmark).await?
        } else {
            exists.unwrap()
        };

        tracing::debug!("adding bookmark: {} to list: {}", &bookmark_id, list_id);
        // Either way, make sure that the bookmark is in the specified list
        self.ensure_bookmark_in_list(&bookmark_id, list_id).await?;
        // Return true if created, false if already existed
        Ok(to_create)
    }
}
