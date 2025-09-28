use std::sync::Arc;

use futures::stream::{self};
use reqwest::{Client, Url, cookie::Jar};

const HN_DOMAIN: &str = "news.ycombinator.com";
const HN_BASE_URL: &str = "https://news.ycombinator.com";

fn get_hn_client(hn_auth: &str) -> anyhow::Result<Arc<Client>> {
    let cookie = format!("user={hn_auth}; Domain={HN_DOMAIN}");
    let url = HN_BASE_URL.parse::<Url>()?;

    let jar = Jar::default();
    jar.add_cookie_str(&cookie, &url);

    let client = reqwest::Client::builder()
        .cookie_provider(jar.into())
        .build()?;

    Ok(Arc::new(client))
}

fn get_upvoted_submissions_from_document(document: &scraper::Html) -> Vec<HNPost> {
    let title_selector = scraper::Selector::parse("tr.athing td.title span.titleline > a")
        .expect("Failed to parse selector");

    document
        .select(&title_selector)
        .map(|el| {
            let url = el.value().attr("href").unwrap_or("").to_string();

            // if the URL is relative, make it absolute
            let url = if url.starts_with("item?") {
                format!("{}/{}", HN_BASE_URL, url)
            } else {
                url
            };

            HNPost {
                title: el.text().collect::<String>(),
                url,
            }
        })
        .collect::<Vec<_>>()
}

fn get_more_link(document: &scraper::Html) -> Option<String> {
    let more_selector = scraper::Selector::parse("a.morelink").expect("Failed to parse selector");
    document
        .select(&more_selector)
        .next()
        .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
}

#[derive(Debug, Clone)]
pub struct HNPost {
    pub title: String,
    pub url: String,
}

pub fn stream_pages(
    hn_auth: &str,
    start_path: String,
) -> anyhow::Result<impl futures::Stream<Item = Vec<HNPost>>> {
    stream_pages_with_base_url(hn_auth, start_path, HN_BASE_URL)
}

fn stream_pages_with_base_url(
    hn_auth: &str,
    start_path: String,
    base_url: &str,
) -> anyhow::Result<impl futures::Stream<Item = Vec<HNPost>>> {
    let client = get_hn_client(&hn_auth)?;
    let base_url = base_url.to_string();

    let pages = stream::unfold(Some(start_path), move |path| {
        let client = Arc::clone(&client);
        let base_url = base_url.clone();
        async move {
            if path.is_none() {
                return None;
            }

            let url = format!("{}/{}", base_url, path.unwrap());
            let response = client.get(&url).send().await;

            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        return None;
                    }

                    match resp.text().await {
                        Ok(body) => {
                            let document = scraper::Html::parse_document(&body);
                            let submissions = get_upvoted_submissions_from_document(&document);
                            let more_link = get_more_link(&document);
                            Some((submissions, more_link))
                        }
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
    });

    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    // Sample HTML for testing parsing functions
    const SAMPLE_HN_HTML: &str = r#"
        <html>
        <body>
        <table>
            <tr class="athing">
                <td class="title">
                    <span class="titleline">
                        <a href="https://example.com/story1">First Story Title</a>
                    </span>
                </td>
            </tr>
            <tr class="athing">
                <td class="title">
                    <span class="titleline">
                        <a href="https://example.com/story2">Second Story Title</a>
                    </span>
                </td>
            </tr>
        </table>
        <a class="morelink" href="?p=2">More</a>
        </body>
        </html>
    "#;

    const SAMPLE_HN_HTML_NO_MORE: &str = r#"
        <html>
        <body>
        <table>
            <tr class="athing">
                <td class="title">
                    <span class="titleline">
                        <a href="https://example.com/last-story">Last Story</a>
                    </span>
                </td>
            </tr>
        </table>
        </body>
        </html>
    "#;

    #[test]
    fn test_get_upvoted_submissions_from_document() {
        let document = Html::parse_document(SAMPLE_HN_HTML);
        let submissions = get_upvoted_submissions_from_document(&document);

        assert_eq!(submissions.len(), 2);
        assert_eq!(submissions[0].title, "First Story Title");
        assert_eq!(submissions[0].url, "https://example.com/story1");
        assert_eq!(submissions[1].title, "Second Story Title");
        assert_eq!(submissions[1].url, "https://example.com/story2");
    }

    #[test]
    fn test_get_upvoted_submissions_empty_document() {
        let document = Html::parse_document("<html><body></body></html>");
        let submissions = get_upvoted_submissions_from_document(&document);
        assert_eq!(submissions.len(), 0);
    }

    #[test]
    fn test_get_more_link_exists() {
        let document = Html::parse_document(SAMPLE_HN_HTML);
        let more_link = get_more_link(&document);

        assert!(more_link.is_some());
        assert_eq!(more_link.unwrap(), "?p=2");
    }

    #[test]
    fn test_get_more_link_not_exists() {
        let document = Html::parse_document(SAMPLE_HN_HTML_NO_MORE);
        let more_link = get_more_link(&document);

        assert!(more_link.is_none());
    }

    #[test]
    fn test_get_hn_client_valid_auth() {
        let result = get_hn_client("test_user_auth_token");
        assert!(result.is_ok());

        let client = result.unwrap();
        assert!(Arc::strong_count(&client) == 1);
    }

    // Integration test using wiremock for HTTP mocking
    #[tokio::test]
    async fn test_stream_pages_with_mock_server() {
        use futures::StreamExt;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        // Start a mock server
        let mock_server = MockServer::start().await;

        // Mock the first page response
        let first_page_html = r#"
            <html>
            <body>
            <table>
                <tr class="athing">
                    <td class="title">
                        <span class="titleline">
                            <a href="https://example.com/story1">Test Story 1</a>
                        </span>
                    </td>
                </tr>
            </table>
            <a class="morelink" href="?p=2">More</a>
            </body>
            </html>
        "#;

        // Mock the second page response (no more link)
        let second_page_html = r#"
            <html>
            <body>
            <table>
                <tr class="athing">
                    <td class="title">
                        <span class="titleline">
                            <a href="https://example.com/story2">Test Story 2</a>
                        </span>
                    </td>
                </tr>
            </table>
            </body>
            </html>
        "#;

        // Mock first page
        Mock::given(method("GET"))
            .and(path("/upvoted"))
            .respond_with(ResponseTemplate::new(200).set_body_string(first_page_html))
            .mount(&mock_server)
            .await;

        // Mock second page
        Mock::given(method("GET"))
            .and(path("/"))
            .and(query_param("p", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(second_page_html))
            .mount(&mock_server)
            .await;

        // Test the streaming functionality
        let base_uri = mock_server.uri();
        let stream =
            stream_pages_with_base_url("test_auth", "upvoted".to_string(), &base_uri).unwrap();

        let pages: Vec<_> = stream.take(2).collect().await;

        // Verify we got 2 pages
        assert_eq!(pages.len(), 2);

        // Verify first page content
        assert_eq!(pages[0].len(), 1);
        assert_eq!(pages[0][0].title, "Test Story 1");
        assert_eq!(pages[0][0].url, "https://example.com/story1");

        // Verify second page content
        assert_eq!(pages[1].len(), 1);
        assert_eq!(pages[1][0].title, "Test Story 2");
        assert_eq!(pages[1][0].url, "https://example.com/story2");
    }

    #[tokio::test]
    async fn test_stream_pages_single_page() {
        use futures::StreamExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock a single page with no "more" link
        let single_page_html = r#"
            <html>
            <body>
            <table>
                <tr class="athing">
                    <td class="title">
                        <span class="titleline">
                            <a href="https://example.com/single-story">Single Story</a>
                        </span>
                    </td>
                </tr>
            </table>
            </body>
            </html>
        "#;

        Mock::given(method("GET"))
            .and(path("/upvoted"))
            .respond_with(ResponseTemplate::new(200).set_body_string(single_page_html))
            .mount(&mock_server)
            .await;

        let base_uri = mock_server.uri();
        let stream =
            stream_pages_with_base_url("test_auth", "upvoted".to_string(), &base_uri).unwrap();

        let pages: Vec<_> = stream.collect().await;

        // Should only get one page since there's no "more" link
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].len(), 1);
        assert_eq!(pages[0][0].title, "Single Story");
        assert_eq!(pages[0][0].url, "https://example.com/single-story");
    }

    #[tokio::test]
    async fn test_stream_pages_http_error() {
        use futures::StreamExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock a 404 response
        Mock::given(method("GET"))
            .and(path("/upvoted"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let base_uri = mock_server.uri();
        let stream =
            stream_pages_with_base_url("test_auth", "upvoted".to_string(), &base_uri).unwrap();

        let pages: Vec<_> = stream.collect().await;

        // Should get no pages due to HTTP error
        assert_eq!(pages.len(), 0);
    }

    #[tokio::test]
    async fn test_stream_pages_malformed_html() {
        use futures::StreamExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock a response with malformed HTML
        let malformed_html = r#"
            <html>
            <body>
            <div>This doesn't match our selectors</div>
            </body>
            </html>
        "#;

        Mock::given(method("GET"))
            .and(path("/upvoted"))
            .respond_with(ResponseTemplate::new(200).set_body_string(malformed_html))
            .mount(&mock_server)
            .await;

        let base_uri = mock_server.uri();
        let stream =
            stream_pages_with_base_url("test_auth", "upvoted".to_string(), &base_uri).unwrap();

        let pages: Vec<_> = stream.collect().await;

        // Should get one page but with empty submissions
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].len(), 0);
    }
}
