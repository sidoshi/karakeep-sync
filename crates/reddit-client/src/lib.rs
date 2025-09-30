use reqwest::header;
use serde::Deserialize;

const TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";
const APP_URL: &str = "https://oauth.reddit.com";

pub struct RedditClientRefresher {
    refresh_token: String,
    client: reqwest::Client,

    client_id: String,
    client_secret: String,
}

pub struct RedditClient {
    access_token: String,
    client: reqwest::Client,
    username: String,
}

impl RedditClientRefresher {
    pub fn new(client_id: String, client_secret: String, refresh_token: String) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("karakeep-sync/0.1 by u/doshisid"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            refresh_token,
            client,
            client_id,
            client_secret,
        }
    }

    pub async fn refresh(&self) -> anyhow::Result<RedditClient> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &self.refresh_token),
        ];

        let resp = self
            .client
            .post(TOKEN_URL)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let access_token = resp
            .get("access_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to get access token from response: {:?}", resp)
            })?;

        let resp = self
            .client
            .get(&format!("{}/api/v1/me", APP_URL))
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let username = resp
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to get username from response: {:?}", resp))?;

        Ok(RedditClient {
            access_token: access_token.to_string(),
            client: self.client.clone(),
            username: username.to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct ListingResponse {
    pub data: ListingData,
}

#[derive(Debug, Deserialize)]
pub struct ListingData {
    pub children: Vec<ListingChild>,
    pub after: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct ListingChild {
    pub kind: String,
    pub data: ListingChildData,
}
#[derive(Debug, Deserialize)]
pub struct ListingChildData {
    pub title: Option<String>,
    pub permalink: String,
}

#[derive(Debug)]
pub struct SavedPost {
    pub title: String,
    pub url: String,
}

#[derive(Debug)]
pub struct ListSavedResponse {
    pub posts: Vec<SavedPost>,
    pub after: Option<String>,
}

impl RedditClient {
    pub async fn list_saved(&self, after: Option<&str>) -> anyhow::Result<ListSavedResponse> {
        let mut req = self
            .client
            .get(&format!("{}/user/{}/saved", APP_URL, self.username))
            .bearer_auth(&self.access_token);

        if let Some(after) = after {
            req = req.query(&[("after", after)]);
        }

        let resp = req.send().await?.json::<ListingResponse>().await?;

        let posts = resp
            .data
            .children
            .into_iter()
            .map(|child| {
                let data = child.data;
                SavedPost {
                    title: data
                        .title
                        .unwrap_or_else(|| "(unknown title reddit post)".to_string()),
                    url: format!("https://reddit.com{}", data.permalink),
                }
            })
            .collect::<Vec<_>>();

        Ok({
            ListSavedResponse {
                posts,
                after: resp.data.after,
            }
        })
    }
}
