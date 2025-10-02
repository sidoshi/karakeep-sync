use crate::settings;
use async_trait::async_trait;
use futures::{Stream, stream};
use karakeep_client::BookmarkCreate;
use reqwest::Url;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct GithubStars {}

fn parse_next_link(link_header: &str) -> Option<String> {
    link_header
        .split(',')
        .find_map(|link| {
            let parts: Vec<&str> = link.split(';').collect();
            if parts.len() == 2 && parts[1].trim() == r#"rel="next""# {
                let url: Url = parts[0]
                    .trim()
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .parse()
                    .ok()?;

                let query = url.query()?;
                Some(format!("?{query}"))
            } else {
                None
            }
        })
        .map(|s| s.to_string())
}

#[async_trait]
impl super::Plugin for GithubStars {
    fn list_name(&self) -> &'static str {
        "GitHub Starred"
    }

    async fn to_bookmark_stream(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Vec<BookmarkCreate>> + Send>>> {
        let stream = stream::unfold(Some("?page=2".to_string()), move |params| async move {
            let params = params?;

            let settings = &settings::get_settings();
            let token = settings
                .github
                .token
                .as_ref()
                .expect("GitHub token must be set for GitHub Stars plugin");

            tracing::info!("fetching GitHub stars with params: {}, {token}", params);

            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("Authorization", format!("Bearer {token}").parse().unwrap());
            headers.insert("User-Agent", "karakeep-sync/1.0".parse().unwrap());
            headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());

            let client = reqwest::Client::new();
            let url = format!("https://api.github.com/user/starred{params}");
            let resp = client.get(url).headers(headers).send().await.ok()?;

            let next_page = resp.headers().get("Link").and_then(|link_header| {
                let link_str = link_header.to_str().ok()?;
                parse_next_link(link_str)
            });

            let resp = resp.json::<Vec<serde_json::Value>>().await.ok()?;
            let bookmarks: Vec<BookmarkCreate> = resp
                .into_iter()
                .map(|item| BookmarkCreate {
                    url: item["html_url"].as_str().unwrap_or("").to_string(),
                    title: item["full_name"].as_str().unwrap_or("").to_string(),
                    // GitHub does not provide timestamp for when the repo was starred
                    created_at: None,
                })
                .collect();

            Some((bookmarks, next_page))
        });
        Ok(Box::pin(stream))
    }

    fn is_activated(&self) -> bool {
        let settings = &settings::get_settings();
        settings.github.token.is_some() && !settings.github.token.as_ref().unwrap().is_empty()
    }

    fn recurring_schedule(&self) -> String {
        let settings = &settings::get_settings();
        settings.github.schedule.clone()
    }
}

#[cfg(test)]
mod test {
    use super::parse_next_link;

    #[test]
    fn test_parse_next_link() {
        let link_header = r#"<https://api.github.com/user/starred?page=2>; rel="next", <https://api.github.com/user/starred?page=34>; rel="last""#;
        let next_link = parse_next_link(link_header);
        assert_eq!(next_link, Some("?page=2".to_string()));

        let link_header_no_next = r#"<https://api.github.com/user/starred?page=34>; rel="last""#;
        let next_link = parse_next_link(link_header_no_next);
        assert_eq!(next_link, None);

        let empty_link_header = "";
        let next_link = parse_next_link(empty_link_header);
        assert_eq!(next_link, None);
    }
}
