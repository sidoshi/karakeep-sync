mod hn;
mod karakeep;
mod settings;

#[tokio::main]
async fn main() {
    let settings = settings::Settings::new();
    hn::sync_hn_upvoted_posts(&settings.hn.auth).await.unwrap();
}
