use std::sync::OnceLock;

use karakeep_client::KarakeepClient;

static CLIENT: OnceLock<KarakeepClient> = OnceLock::new();
pub fn get_client() -> &'static KarakeepClient {
    CLIENT.get_or_init(|| {
        let settings = &crate::settings::get_settings().karakeep;
        KarakeepClient::new(&settings.url, &settings.auth)
    })
}
