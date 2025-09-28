use reqwest::{Client, Url, cookie::Jar};

const HN_BASE_URL: &str = "news.ycombinator.com";

fn get_hn_client(hn_auth: &str) -> anyhow::Result<Client> {
    let cookie = format!("user={hn_auth}; Domain={HN_BASE_URL}");
    let url = format!("https://{HN_BASE_URL}").parse::<Url>()?;

    let jar = Jar::default();
    jar.add_cookie_str(&cookie, &url);

    let client = reqwest::Client::builder()
        .cookie_provider(jar.into())
        .build()?;

    Ok(client)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let hn_auth = dotenvy::var("HN_AUTH")?;
    let client = get_hn_client(&hn_auth)?;

    let body = client
        .get("https://news.ycombinator.com/upvoted?id=sidoshi")
        .send()
        .await?
        .text()
        .await?;

    let document = scraper::Html::parse_document(&body);

    let title_selector = scraper::Selector::parse("tr.athing td.title span.titleline > a")
        .expect("Failed to parse selector");
    dbg!(
        document
            .select(&title_selector)
            .map(|el| (
                el.text().collect::<String>(),
                el.value().attr("href").unwrap_or("")
            ))
            .collect::<Vec<_>>()
    );

    Ok(())
}
