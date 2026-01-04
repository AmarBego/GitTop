use crate::github::proxy_keyring;
use crate::settings::AppSettings;

#[derive(Debug, Clone)]
pub struct NetworkProxyState {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub creds_dirty: bool,
    pub needs_rebuild: bool,
}

impl NetworkProxyState {
    pub fn new(settings: &AppSettings) -> Self {
        let enabled = settings.proxy.enabled;
        let url = settings.proxy.url.clone();

        // Load proxy credentials from keyring if they exist
        let (username, password) = if settings.proxy.has_credentials
            && let Ok(Some((user, pass))) =
                proxy_keyring::load_proxy_credentials(&settings.proxy.url)
        {
            (user, pass)
        } else {
            (String::new(), String::new())
        };

        Self {
            enabled,
            url,
            username,
            password,
            creds_dirty: false,
            needs_rebuild: false,
        }
    }
}
